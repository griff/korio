use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering, fence};

use korio_core::metadata::{Metadata, MetadataProvider};
use korio_core::span::{Current, ExtensionSet, Id};
use korio_core::{Severity, Subscriber};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use stack::SpanStack;
use thread_local::ThreadLocal;

use crate::layer::{LookupSpan, SpanRefData};
use crate::{IdGenerator, id_generator};

mod stack;

pub struct Registry {
    pool: Arc<RwLock<SpanPool>>,
    current_spans: ThreadLocal<RefCell<SpanStack>>,
    id_generator: Arc<dyn IdGenerator>,
}

impl Registry {
    pub fn new() -> Self {
        Self::with_id_generator(id_generator::RandomIdGenerator())
    }

    pub fn with_id_generator<I>(id_generator: I) -> Self
    where
        I: IdGenerator + 'static,
    {
        Self {
            pool: Default::default(),
            current_spans: Default::default(),
            id_generator: Arc::new(id_generator),
        }
    }

    fn write_pool(&self) -> RwLockWriteGuard<'_, SpanPool> {
        self.pool.write()
    }

    fn read_pool(&self) -> RwLockReadGuard<'_, SpanPool> {
        self.pool.read()
    }

    fn get(&self, id: &Id) -> Option<Arc<ActiveSpan>> {
        self.read_pool().active_spans.get(id).cloned()
    }

    /// Returns a guard which tracks how many `Layer`s have
    /// processed an `on_close` notification via the `CLOSE_COUNT` thread-local.
    /// For additional details, see [`CloseGuard`].
    ///
    pub(crate) fn start_close(&self, id: Id) -> CloseGuard<'_> {
        CLOSE_COUNT.with(|count| {
            let c = count.get();
            count.set(c + 1);
        });
        CloseGuard {
            id,
            registry: self,
            is_closing: false,
        }
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Subscriber for Registry {
    fn upstream_span(&self, id: &Id) {
        self.write_pool()
            .active_spans
            .insert(id.clone(), Arc::new(ActiveSpan::new_upstream(id.clone())));
    }

    fn new_span(&self, span_data: &dyn korio_core::span::NewSpanData) -> Id {
        // resolve parent
        let parent = if span_data.is_root() {
            None
        } else if span_data.is_contextual() {
            self.current_span().id().cloned()
        } else {
            span_data.parent()
        };

        // create id
        let trace_id = parent
            .as_ref()
            .map(|parent_id| parent_id.trace_id)
            .unwrap_or_else(|| self.id_generator.generate_trace_id());
        let span_id = self.id_generator.generate_span_id();
        let id = Id { trace_id, span_id };

        self.write_pool().active_spans.insert(
            id.clone(),
            Arc::new(ActiveSpan::new(id.clone(), parent, span_data)),
        );

        id
    }

    fn downstream_span(&self, span_data: &dyn korio_core::span::SpanData) {
        let id = span_data.id().clone();
        let parent = span_data.parent().clone();

        self.write_pool().active_spans.insert(
            id.clone(),
            Arc::new(ActiveSpan::new(id.clone(), parent, span_data)),
        );
    }

    fn enter(&self, id: &Id) {
        self.current_spans
            .get_or_default()
            .borrow_mut()
            .push(id.clone());
    }

    fn exit(&self, id: &Id) {
        if let Some(spans) = self.current_spans.get() {
            spans.borrow_mut().pop(id);
        }
    }

    fn clone_span(&self, id: &Id) -> Id {
        let span = self
                .get(id)
                .unwrap_or_else(|| panic!(
                    "tried to clone {:?}, but no span exists with that ID\n\
                    This may be caused by consuming a parent span (`parent: span`) rather than borrowing it (`parent: &span`).",
                    id,
                ));
        // Like `std::sync::Arc`, adds to the ref count (on clone) don't require
        // a strong ordering; if we call` clone_span`, the reference count must
        // always at least 1. The only synchronization necessary is between
        // calls to `try_close`: we have to ensure that all threads have
        // dropped their refs to the span before the span is closed.
        let refs = span.ref_count.fetch_add(1, Ordering::Relaxed);
        assert_ne!(
            refs, 0,
            "tried to clone a span ({:?}) that already closed",
            id
        );
        id.clone()
    }

    fn try_close(&self, id: Id) -> bool {
        let span = match self.get(&id) {
            Some(span) => span,
            None if std::thread::panicking() => return false,
            None => panic!("tried to drop a ref to {:?}, but no such span exists!", id),
        };

        let refs = span.ref_count.fetch_sub(1, Ordering::Release);
        if !std::thread::panicking() {
            assert!(refs < usize::MAX, "reference count overflow!");
        }
        if refs > 1 {
            return false;
        }

        // Synchronize if we are actually removing the span (stolen
        // from std::Arc); this ensures that all other `try_close` calls on
        // other threads happen-before we actually remove the span.
        fence(Ordering::Acquire);
        true
    }

    fn current_span(&self) -> Current {
        self.current_spans
            .get()
            .and_then(|spans| {
                let spans = spans.borrow();
                let id = spans.current()?;
                let span = self.get(id)?;
                Some(Current::new(id.clone(), span))
            })
            .unwrap_or_else(Current::none)
    }

    fn on_register_dispatch(&self, dispatch: &korio_core::dispatcher::Dispatch) {
        let _ = dispatch;
    }

    unsafe fn downcast_raw(&self, id: std::any::TypeId) -> Option<*const ()> {
        if id == std::any::TypeId::of::<Self>() {
            Some(self as *const Self as *const ())
        } else {
            None
        }
    }

    #[inline]
    fn start(&self, _id: &Id, _start_time: std::time::SystemTime) {}

    #[inline]
    fn end(&self, _id: &Id, _end_time: std::time::SystemTime) {}

    #[inline]
    fn record(&self, _id: &Id, _record: &dyn korio_core::span::SpanRecordData) {}

    #[inline]
    fn record_follows_from(
        &self,
        _id: &Id,
        _follows_from: &dyn korio_core::span::SpanFollowsFromData,
    ) {
    }

    #[inline]
    fn new_event(&self, event: &dyn korio_core::event::NewEventData) {
        eprintln!("Registry new event {} {}", event.target(), event.name());
    }

    #[inline]
    fn downstream_event(&self, _event: &dyn korio_core::event::EventData) {}
}

#[derive(Debug, Default)]
struct SpanPool {
    active_spans: HashMap<Id, Arc<ActiveSpan>>,
}

#[derive(Debug)]
struct ActiveSpan {
    id: Id,
    parent_id: Option<Id>,
    ref_count: AtomicUsize,
    metadata: Option<OwnedMetadata>,
    extensions: ExtensionSet,
}

impl ActiveSpan {
    fn new(id: Id, parent_id: Option<Id>, metadata: &dyn Metadata) -> Self {
        Self {
            id,
            parent_id,
            ref_count: AtomicUsize::new(1),
            metadata: Some(OwnedMetadata::from_data(metadata)),
            extensions: Default::default(),
        }
    }

    fn new_upstream(id: Id) -> Self {
        Self {
            id,
            parent_id: None,
            ref_count: AtomicUsize::new(1),
            metadata: None,
            extensions: Default::default(),
        }
    }
}

impl MetadataProvider for ActiveSpan {
    fn metadata(&self) -> Option<&dyn Metadata> {
        self.metadata.as_ref().map(|m| m as &dyn Metadata)
    }
}

impl SpanRefData for ActiveSpan {
    fn id(&self) -> Id {
        self.id.clone()
    }

    fn parent(&self) -> Option<Id> {
        self.parent_id.clone()
    }

    fn metadata(&self) -> Option<&dyn Metadata> {
        self.metadata.as_ref().map(|m| m as &dyn Metadata)
    }

    fn extensions(&self) -> korio_core::span::Extensions<'_> {
        self.extensions.read()
    }

    fn extensions_mut(&self) -> korio_core::span::ExtensionsMut<'_> {
        self.extensions.write()
    }
}

#[derive(Debug)]
pub struct OwnedMetadata {
    /// Target of the span or event.
    pub target: String,

    /// Name of the span or event.
    pub name: String,

    /// The original severity string from the source
    pub severity_text: String,

    /// The corresponding severity value, normalized
    pub severity_number: Severity,
}

impl OwnedMetadata {
    pub fn from_data(data: &dyn Metadata) -> Self {
        let target = data.target().to_string();
        let name = data.name().to_string();
        let severity_text = data.severity_text().to_string();
        let severity_number = data.severity_number();
        Self {
            target,
            name,
            severity_text,
            severity_number,
        }
    }
}

impl Metadata for OwnedMetadata {
    fn name(&self) -> &str {
        &self.name
    }

    fn target(&self) -> &str {
        &self.target
    }

    fn severity_number(&self) -> Severity {
        self.severity_number
    }

    fn severity_text(&self) -> &str {
        &self.severity_text
    }
}

impl LookupSpan for Registry {
    fn span_data(&self, id: &Id) -> Option<Arc<dyn SpanRefData>> {
        self.get(id).map(|data| data as Arc<dyn SpanRefData>)
    }
}

/// A guard that tracks how many [`Registry`]-backed `Layer`s have
/// processed an `on_close` event.
///
/// This is needed to enable a [`Registry`]-backed Layer to access span
/// data after the `Layer` has recieved the `on_close` callback.
///
/// Once all `Layer`s have processed this event, the [`Registry`] knows
/// that is able to safely remove the span tracked by `id`. `CloseGuard`
/// accomplishes this through a two-step process:
/// 1. Whenever a [`Registry`]-backed `Layer::on_close` method is
///    called, `Registry::start_close` is closed.
///    `Registry::start_close` increments a thread-local `CLOSE_COUNT`
///    by 1 and returns a `CloseGuard`.
/// 2. The `CloseGuard` is dropped at the end of `Layer::on_close`. On
///    drop, `CloseGuard` checks thread-local `CLOSE_COUNT`. If
///    `CLOSE_COUNT` is 0, the `CloseGuard` removes the span with the
///    `id` from the registry, as all `Layers` that might have seen the
///    `on_close` notification have processed it. If `CLOSE_COUNT` is
///    greater than 0, `CloseGuard` decrements the counter by one and
///    _does not_ remove the span from the [`Registry`].
///
pub(crate) struct CloseGuard<'r> {
    id: Id,
    registry: &'r Registry,
    is_closing: bool,
}

// === impl CloseGuard ===

impl CloseGuard<'_> {
    pub(crate) fn set_closing(&mut self) {
        self.is_closing = true;
    }
}

impl Drop for CloseGuard<'_> {
    fn drop(&mut self) {
        // If this returns with an error, we are already panicking. At
        // this point, there's nothing we can really do to recover
        // except by avoiding a double-panic.
        let _ = CLOSE_COUNT.try_with(|count| {
            let c = count.get();
            // Decrement the count to indicate that _this_ guard's
            // `on_close` callback has completed.
            //
            // Note that we *must* do this before we actually remove the span
            // from the registry, since dropping the `DataInner` may trigger a
            // new close, if this span is the last reference to a parent span.
            count.set(c - 1);

            // If the current close count is 1, this stack frame is the last
            // `on_close` call. If the span is closing, it's okay to remove the
            // span.
            if c == 1 && self.is_closing {
                let mut pool = self.registry.write_pool();
                pool.active_spans.remove(&self.id);
            }
        });
    }
}

thread_local! {
    /// `CLOSE_COUNT` is the thread-local counter used by `CloseGuard` to
    /// track how many layers have processed the close.
    /// For additional details, see [`CloseGuard`].
    ///
    static CLOSE_COUNT: Cell<usize> = const { Cell::new(0) };
}
