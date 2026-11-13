//! Dispatching to a [`Subscriber`]

use std::any::Any;
use std::cell::{Cell, Ref, RefCell};
use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock, Weak};
use std::time::SystemTime;

use crate::Subscriber;
use crate::event::{EventData, NewEventData};
use crate::span::{Current, Id, NewSpanData, SpanData, SpanFollowsFromData, SpanRecordData};
use crate::subscriber::NoSubscriber;

static GLOBAL_DISPATCH: OnceLock<Dispatch> = OnceLock::new();
static NONE: &Dispatch = &Dispatch::none();
static NO_SUBSCRIBER: NoSubscriber = NoSubscriber::new();
static SCOPED_COUNT: AtomicUsize = AtomicUsize::new(0);

std::thread_local! {
    static CURRENT_STATE: State = const {
        State {
            default: RefCell::new(None),
            can_enter: Cell::new(true),
        }
    };
}

/// The dispatch state of a thread.
struct State {
    /// This thread's current default dispatcher.
    default: RefCell<Option<Dispatch>>,
    /// Whether or not we can currently begin dispatching a trace event.
    ///
    /// This is set to `false` when functions such as `enter`, `exit`, `event`,
    /// and `new_span` are called on this thread's default dispatcher, to
    /// prevent further trace events triggered inside those functions from
    /// creating an infinite recursion. When we finish handling a dispatch, this
    /// is set back to `true`.
    can_enter: Cell<bool>,
}

/// While this guard is active, additional calls to subscriber functions on
/// the default dispatcher will not be able to access the dispatch context.
/// Dropping the guard will allow the dispatch context to be re-entered.
struct Entered<'a>(&'a State);

/// A trait that implements `Sync`, but not `Send`. Used with PhantomData, this
/// can mark a struct as `!Send`.
trait NotSend: Sync {}

/// A guard that resets the current default dispatcher to the prior
/// default dispatcher when dropped.
#[derive(Debug)]
pub struct DefaultGuard(
    Option<Dispatch>,
    /// ```compile_fail
    /// use tracing_core::dispatch::*;
    /// trait AssertSend: Send {}
    ///
    /// impl AssertSend for DefaultGuard {}
    /// ```
    PhantomData<dyn NotSend>,
);

/// Sets this dispatch as the default for the duration of a closure.
///
/// The default dispatcher is used when creating a new [span] or
/// [`Event`].
///
/// <pre class="ignore" style="white-space:normal;font:inherit;">
///     <strong>Note</strong>: This function required the Rust standard library.
///     <code>no_std</code> users should use <a href="fn.set_global_default.html">
///     <code>set_global_default</code></a> instead.
/// </pre>
///
/// [span]: super::span
/// [`Subscriber`]: super::subscriber::Subscriber
/// [`Event`]: super::event::Event
pub fn with_default<T>(dispatcher: &Dispatch, f: impl FnOnce() -> T) -> T {
    // When this guard is dropped, the default dispatcher will be reset to the
    // prior default. Using this (rather than simply resetting after calling
    // `f`) ensures that we always reset to the prior dispatcher even if `f`
    // panics.
    let _guard = set_default(dispatcher);
    f()
}

///Sets this dispatch as the default for the duration of the closure if no other dispatch is set.
pub fn with_optional_default<T>(dispatcher: &Dispatch, f: impl FnOnce() -> T) -> T {
    if is_default_unset() {
        with_default(dispatcher, f)
    } else {
        (f)()
    }
}

/// Sets this dispatch as the current thread's default dispatch for the duration
/// of the lifetime of the returned [`DefaultGuard`].
///
/// <div class="example-wrap" style="display:inline-block">
/// <pre class="ignore" style="white-space:normal;font:inherit;">
///
/// **Note**: Because this sets the dispatch for the current thread only, the returned
/// [`DefaultGuard`] does not implement [`Send`]. If the guard was sent to another thread and
/// dropped there, we'd try to restore the dispatch value for the wrong thread. Thus,
/// [`DefaultGuard`] should not be sent between threads.
///
/// </pre></div>
#[must_use = "Dropping the guard unregisters the dispatcher."]
pub fn set_default(dispatcher: &Dispatch) -> DefaultGuard {
    // When this guard is dropped, the default dispatcher will be reset to the
    // prior default. Using this ensures that we always reset to the prior
    // dispatcher even if the thread calling this function panics.
    State::set_default(dispatcher.clone())
}

/// Sets this dispatch as the global default for the duration of the entire program.
/// Will be used as a fallback if no thread-local dispatch has been set in a thread
/// (using `with_default`.)
///
/// Can only be set once; subsequent attempts to set the global default will fail.
/// Returns `Err` if the global default has already been set.
///
/// <div class="example-wrap" style="display:inline-block"><pre class="compile_fail" style="white-space:normal;font:inherit;">
///     <strong>Warning</strong>: In general, libraries should <em>not</em> call
///     <code>set_global_default()</code>! Doing so will cause conflicts when
///     executables that depend on the library try to set the default later.
/// </pre></div>
///
/// [span]: super::span
/// [`Subscriber`]: super::subscriber::Subscriber
/// [`Event`]: super::event::Event
pub fn set_global_default(dispatch: Dispatch) -> Result<(), SetGlobalDefaultError> {
    GLOBAL_DISPATCH
        .set(dispatch)
        .map_err(|_| SetGlobalDefaultError { _no_construct: () })
}

/// Return true if the global default has been explicitly set.
///
/// This means that [`set_global_default`] has been called.
pub fn is_global_default_set() -> bool {
    GLOBAL_DISPATCH.get().is_some()
}

/// Return true if the thread's dispatcher has been explicitly set.
///
/// This means that [`set_default`] or [`with_default`] has been called.
pub fn is_default_set() -> bool {
    CURRENT_STATE
        .try_with(|state| state.default.borrow().is_some())
        .unwrap_or_default()
}

/// Return true if no dispatcher has been explicitly set.
pub fn is_default_unset() -> bool {
    !is_default_set() && !is_global_default_set()
}

/// Executes a closure with a reference to this thread's current [dispatcher].
///
/// Note that calls to `get_default` should not be nested; if this function is
/// called while inside of another `get_default`, that closure will be provided
/// with `Dispatch::none` rather than the previously set dispatcher.
///
/// [dispatcher]: super::dispatcher::Dispatch
pub fn get_default<T, F>(mut f: F) -> T
where
    F: FnMut(&Dispatch) -> T,
{
    if SCOPED_COUNT.load(Ordering::Acquire) == 0 {
        // fast path if no scoped dispatcher has been set; just use the global
        // default.
        return f(get_global());
    }

    CURRENT_STATE
        .try_with(|state| {
            if let Some(entered) = state.enter() {
                return f(&entered.current());
            }

            f(NONE)
        })
        .unwrap_or_else(|_| f(NONE))
}

/// Executes a closure with a reference to this thread's current [dispatcher].
///
/// Note that calls to `get_default` should not be nested; if this function is
/// called while inside of another `get_default`, that closure will be provided
/// with `Dispatch::none` rather than the previously set dispatcher.
///
/// [dispatcher]: super::dispatcher::Dispatch
#[doc(hidden)]
#[inline(never)]
pub fn get_current<T>(f: impl FnOnce(&Dispatch) -> T) -> Option<T> {
    if SCOPED_COUNT.load(Ordering::Acquire) == 0 {
        // fast path if no scoped dispatcher has been set; just use the global
        // default.
        return Some(f(get_global()));
    }

    CURRENT_STATE
        .try_with(|state| {
            let entered = state.enter()?;
            Some(f(&entered.current()))
        })
        .ok()?
}

#[inline]
fn get_global() -> &'static Dispatch {
    GLOBAL_DISPATCH.get().unwrap_or(NONE)
}

/// A dispather for a subscriber if any.
#[derive(Clone)]
pub struct Dispatch {
    subscriber: Option<Arc<dyn Subscriber + Send + Sync>>,
}

impl Dispatch {
    /// Return a `Dispatch` that has no subscriber and ignores all spans.
    pub const fn none() -> Self {
        Self { subscriber: None }
    }

    /// Return a `Dispatch` that forwards to the provided [`Subscriber`].
    pub fn new<S>(subscriber: S) -> Self
    where
        S: Subscriber + Send + Sync,
    {
        let this = Self {
            subscriber: Some(Arc::new(subscriber)),
        };
        this.subscriber().on_register_dispatch(&this);
        this
    }

    fn subscriber(&self) -> &(dyn Subscriber + Send + Sync) {
        self.subscriber.as_deref().unwrap_or(&NO_SUBSCRIBER)
    }

    /// Register a remote upstream span and context.
    pub fn upstream_span(&self, span: &Id) {
        self.subscriber().upstream_span(span);
    }

    /// Record a newly created span.
    #[inline]
    pub fn new_span(&self, span: &dyn NewSpanData) -> Id {
        self.subscriber().new_span(span)
    }

    /// Record a remote downstream span.
    pub fn downstream_span(&self, span: &dyn SpanData) {
        self.subscriber().downstream_span(span);
    }

    /// Record that a span was started.
    pub fn start(&self, id: &Id, start_time: SystemTime) {
        self.subscriber().start(id, start_time);
    }

    /// Record that a span ended.
    pub fn end(&self, id: &Id, end_time: SystemTime) {
        self.subscriber().end(id, end_time);
    }

    /// Record new attributes for a span
    pub fn record(&self, id: &Id, record: &dyn SpanRecordData) {
        self.subscriber().record(id, record);
    }

    /// Record that a span follows from another span.
    pub fn record_follows_from(&self, id: &Id, follows_from: &dyn SpanFollowsFromData) {
        self.subscriber().record_follows_from(id, follows_from);
    }

    /// Record a newly created event.
    pub fn new_event(&self, event: &dyn NewEventData) {
        eprintln!("New event {} {}", event.target(), event.name());
        self.subscriber().new_event(event);
    }

    /// Record a remote downstream event.
    pub fn downstream_event(&self, event: &dyn EventData) {
        self.subscriber().downstream_event(event);
    }

    /// Record that a span has been entered.
    pub fn enter(&self, id: &Id) {
        self.subscriber().enter(id);
    }

    /// Record that a span has been exited.
    pub fn exit(&self, id: &Id) {
        self.subscriber().exit(id);
    }

    /// Record that a span has been cloned.
    pub fn clone_span(&self, id: &Id) -> Id {
        self.subscriber().clone_span(id)
    }

    /// Record that a given span has been dropped and possibly need to close.
    ///
    /// **Note**: since this function is called when spans are dropped,
    /// implementations should ensure that they are unwind-safe. Panicking from
    /// inside of a `try_close` function may cause a double panic, if the span
    /// was dropped due to a thread unwinding.
    pub fn try_close(&self, id: Id) -> bool {
        self.subscriber().try_close(id)
    }

    /// Return the current span if defined.
    pub fn current_span(&self) -> Current {
        self.subscriber().current_span()
    }

    /// Downgrade this `Dispatch` to a [`WeakDispatch`].
    pub fn downgrade(&self) -> WeakDispatch {
        let subscriber = self.subscriber.as_ref().map(Arc::downgrade);
        WeakDispatch { subscriber }
    }

    /// Return `true` if this `Dispatch` forwards to a [`Subscriber`] of type `T`.
    pub fn is<T: Any>(&self) -> bool {
        self.subscriber().is::<T>()
    }

    /// Returns some reference that this `Dispatch` points to if it is of type `T`, or `None` otherwise.
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.subscriber().downcast_ref::<T>()
    }
}

impl Default for Dispatch {
    /// Returns the current default dispatcher
    fn default() -> Self {
        get_default(|default| default.clone())
    }
}

impl fmt::Debug for Dispatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dispatch")
            .field("subscriber", &format_args!("{:p}", self.subscriber()))
            .finish()
    }
}

impl<S> From<S> for Dispatch
where
    S: Subscriber + Send + Sync + 'static,
{
    #[inline]
    fn from(subscriber: S) -> Self {
        Dispatch::new(subscriber)
    }
}

/// A `Dispatch` with a non-owning reference.
#[derive(Debug, Clone)]
pub struct WeakDispatch {
    subscriber: Option<Weak<dyn Subscriber + Send + Sync>>,
}

impl WeakDispatch {
    /// Attempt to upgrade this `WeakDispatch` to a [`Dispatch`].
    ///
    /// Return `None` if the referenced `Subscriber` has been dropped.
    pub fn upgrade(&self) -> Option<Dispatch> {
        if let Some(subscriber) = self.subscriber.as_ref() {
            subscriber.upgrade().map(|subscriber| Dispatch {
                subscriber: Some(subscriber),
            })
        } else {
            Some(Dispatch { subscriber: None })
        }
    }
}

impl State {
    /// Replaces the current default dispatcher on this thread with the provided
    /// dispatcher.Any
    ///
    /// Dropping the returned `ResetGuard` will reset the default dispatcher to
    /// the previous value.
    #[inline]
    fn set_default(new_dispatch: Dispatch) -> DefaultGuard {
        let prior = CURRENT_STATE
            .try_with(|state| {
                state.can_enter.set(true);
                state.default.replace(Some(new_dispatch))
            })
            .ok()
            .flatten();
        SCOPED_COUNT.fetch_add(1, Ordering::Release);
        DefaultGuard(prior, PhantomData)
    }

    #[inline]
    fn enter(&self) -> Option<Entered<'_>> {
        if self.can_enter.replace(false) {
            Some(Entered(self))
        } else {
            None
        }
    }
}

// ===== impl Entered =====

impl<'a> Entered<'a> {
    #[inline]
    fn current(&self) -> Ref<'a, Dispatch> {
        let default = self.0.default.borrow();
        Ref::map(default, |default| match default {
            Some(default) => default,
            None => get_global(),
        })
    }
}

impl Drop for Entered<'_> {
    #[inline]
    fn drop(&mut self) {
        self.0.can_enter.set(true);
    }
}

// ===== impl DefaultGuard =====

impl Drop for DefaultGuard {
    #[inline]
    fn drop(&mut self) {
        // Replace the dispatcher and then drop the old one outside
        // of the thread-local context. Dropping the dispatch may
        // lead to the drop of a subscriber which, in the process,
        // could then also attempt to access the same thread local
        // state -- causing a clash.
        let prev = CURRENT_STATE.try_with(|state| state.default.replace(self.0.take()));
        SCOPED_COUNT.fetch_sub(1, Ordering::Release);
        drop(prev)
    }
}

/// Returned if setting the global dispatcher fails.
pub struct SetGlobalDefaultError {
    _no_construct: (),
}

impl fmt::Debug for SetGlobalDefaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SetGlobalDefaultError")
            .field(&Self::MESSAGE)
            .finish()
    }
}

impl fmt::Display for SetGlobalDefaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(Self::MESSAGE)
    }
}

impl std::error::Error for SetGlobalDefaultError {}

impl SetGlobalDefaultError {
    const MESSAGE: &'static str = "a global default trace dispatcher has already been set";
}

#[cfg(test)]
mod test {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::{
        metadata::{AttributeSet, EMPTY_ATTRIBUTE_SET, Metadata, Severity},
        span,
    };

    #[allow(dead_code)]
    trait AssertSync: Sync {}
    impl AssertSync for DefaultGuard {}

    #[test]
    fn dispatch_is() {
        let dispatcher = Dispatch::new(NoSubscriber::default());
        assert!(dispatcher.is::<NoSubscriber>());
    }

    #[test]
    fn dispatch_downcasts() {
        let dispatcher = Dispatch::new(NoSubscriber::default());
        assert!(dispatcher.downcast_ref::<NoSubscriber>().is_some());
    }

    struct TestSpan {
        target: &'static str,
        name: &'static str,
        level: Severity,
    }
    impl Metadata for TestSpan {
        fn name(&self) -> &str {
            self.name
        }

        fn target(&self) -> &str {
            self.target
        }

        fn severity_text(&self) -> &str {
            self.level.as_str()
        }

        fn severity_number(&self) -> Severity {
            self.level
        }
    }
    impl NewSpanData for TestSpan {
        fn parent(&self) -> Option<Id> {
            None
        }

        fn attributes(&self) -> &dyn AttributeSet {
            EMPTY_ATTRIBUTE_SET
        }
    }
    impl NewEventData for TestSpan {
        fn span(&self) -> Option<Id> {
            None
        }

        fn attributes(&self) -> &dyn AttributeSet {
            EMPTY_ATTRIBUTE_SET
        }

        fn timestamp(&self) -> SystemTime {
            SystemTime::now()
        }

        fn body(&self) -> Option<&dyn crate::metadata::Value> {
            None
        }
    }

    static TEST_SPAN: TestSpan = TestSpan {
        name: "test",
        target: module_path!(),
        level: Severity::Debug,
    };

    #[test]
    fn events_dont_infinite_loop() {
        fn mk_event() {
            get_default(|current| current.new_event(&TEST_SPAN));
        }

        // This test ensures that an event triggered within a subscriber
        // won't cause an infinite loop of events.
        struct TestSubscriber;
        impl Subscriber for TestSubscriber {
            fn new_span(&self, _: &dyn span::NewSpanData) -> Id {
                Id {
                    trace_id: span::TraceId::from_u128(0xAAAAAAAA),
                    span_id: span::SpanId::from_u64(0xAAAA),
                }
            }

            fn enter(&self, _: &span::Id) {}
            fn exit(&self, _: &span::Id) {}
            fn upstream_span(&self, _: &Id) {}
            fn downstream_span(&self, _: &dyn SpanData) {}
            fn start(&self, _: &Id, _: SystemTime) {}
            fn end(&self, _: &Id, _: SystemTime) {}
            fn record(&self, _: &Id, _: &dyn SpanRecordData) {}
            fn record_follows_from(&self, _: &Id, _: &dyn SpanFollowsFromData) {}
            fn downstream_event(&self, _: &dyn EventData) {}
            fn new_event(&self, _: &dyn NewEventData) {
                static EVENTS: AtomicUsize = AtomicUsize::new(0);
                assert_eq!(
                    EVENTS.fetch_add(1, Ordering::Relaxed),
                    0,
                    "event method called twice!"
                );
                mk_event();
            }
        }

        with_default(&Dispatch::new(TestSubscriber), mk_event)
    }

    #[test]
    fn spans_dont_infinite_loop() {
        // This test ensures that a span created within a subscriber
        // won't cause an infinite loop of new spans.

        fn mk_span() {
            get_default(|current| current.new_span(&TEST_SPAN));
        }

        struct TestSubscriber;
        impl Subscriber for TestSubscriber {
            fn new_span(&self, _: &dyn span::NewSpanData) -> Id {
                static NEW_SPANS: AtomicUsize = AtomicUsize::new(0);
                assert_eq!(
                    NEW_SPANS.fetch_add(1, Ordering::Relaxed),
                    0,
                    "new_span method called twice!"
                );
                mk_span();
                Id {
                    trace_id: span::TraceId::from_u128(0xAAAAAAAA),
                    span_id: span::SpanId::from_u64(0xAAAA),
                }
            }

            fn enter(&self, _: &span::Id) {}
            fn exit(&self, _: &span::Id) {}
            fn upstream_span(&self, _: &Id) {}
            fn downstream_span(&self, _: &dyn SpanData) {}
            fn start(&self, _: &Id, _: SystemTime) {}
            fn end(&self, _: &Id, _: SystemTime) {}
            fn new_event(&self, _: &dyn NewEventData) {}
            fn downstream_event(&self, _: &dyn EventData) {}
            fn record(&self, _: &Id, _: &dyn SpanRecordData) {}
            fn record_follows_from(&self, _: &Id, _: &dyn SpanFollowsFromData) {}
        }

        with_default(&Dispatch::new(TestSubscriber), mk_span)
    }

    #[test]
    fn default_no_subscriber() {
        let default_dispatcher = Dispatch::default();
        assert!(default_dispatcher.is::<NoSubscriber>());
    }

    #[test]
    fn default_dispatch() {
        struct TestSubscriber;
        impl Subscriber for TestSubscriber {
            fn new_span(&self, _: &dyn span::NewSpanData) -> Id {
                Id {
                    trace_id: span::TraceId::from_u128(0xAAAAAAAA),
                    span_id: span::SpanId::from_u64(0xAAAA),
                }
            }

            fn enter(&self, _: &span::Id) {}
            fn exit(&self, _: &span::Id) {}
            fn upstream_span(&self, _: &Id) {}
            fn downstream_span(&self, _: &dyn SpanData) {}
            fn start(&self, _: &Id, _: SystemTime) {}
            fn end(&self, _: &Id, _: SystemTime) {}
            fn new_event(&self, _: &dyn NewEventData) {}
            fn downstream_event(&self, _: &dyn EventData) {}
            fn record(&self, _: &Id, _: &dyn SpanRecordData) {}
            fn record_follows_from(&self, _: &Id, _: &dyn SpanFollowsFromData) {}
        }
        let guard = set_default(&Dispatch::new(TestSubscriber));
        let default_dispatcher = Dispatch::default();
        assert!(default_dispatcher.is::<TestSubscriber>());

        drop(guard);
        let default_dispatcher = Dispatch::default();
        assert!(default_dispatcher.is::<NoSubscriber>());
    }
}
