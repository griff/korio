use std::fmt;
use std::sync::Arc;

use korio_core::Subscriber;
use korio_core::dispatcher::Dispatch;
use korio_core::event::{EventData, NewEventData};
use korio_core::metadata::Metadata;
use korio_core::span::{Current, Extensions, ExtensionsMut, Id};

mod layered;
mod observer;
pub use layered::Layered;
pub use observer::Observer;
use tracing::trace;

pub trait Layer<S>: Observer
where
    S: Subscriber,
    Self: 'static,
{
    fn on_register_dispatch(&self, dispatch: &Dispatch) {
        let _ = dispatch;
    }

    fn on_layer(&mut self, subscriber: &mut S) {
        let _ = subscriber;
    }

    fn and_then<L>(self, layer: L) -> Layered<L, Self, S>
    where
        L: Layer<S>,
        Self: Sized,
    {
        Layered::new(layer, self)
    }

    fn with_subscriber(self, inner: S) -> Layered<Self, S>
    where
        Self: Sized,
    {
        Layered::new(self, inner)
    }

    fn boxed(self) -> Box<dyn Layer<S> + Send + Sync + 'static>
    where
        Self: Sized + Layer<S> + Send + Sync + 'static,
        S: Subscriber,
    {
        Box::new(self)
    }
}

/// Extension trait adding a `with(Layer)` combinator to `Subscriber`s.
pub trait SubscriberExt: Subscriber + sealed::Sealed {
    /// Wraps `self` with the provided `layer`.
    fn with<L>(self, layer: L) -> Layered<L, Self>
    where
        L: Layer<Self>,
        Self: Sized,
    {
        Layered::new(layer, self)
    }
}

impl<S: Subscriber> sealed::Sealed for S {}
impl<S: Subscriber> SubscriberExt for S {}

mod sealed {
    pub trait Sealed {}
}

/// Subscriber context for `Observer`.
pub trait Context {
    /// Return currently active span
    fn current_span(&self) -> Current;

    /// Lookup span with provided id.
    fn span(&self, id: &Id) -> Option<SpanRef<'_>>;

    fn lookup_current(&self) -> Option<SpanRef<'_>> {
        self.span(self.current_span().id()?)
    }

    fn span_scope(&self, id: &Id) -> Option<Scope<'_>> {
        self.span(id).map(|span| span.scope())
    }

    fn event_span(&self, event: &dyn NewEventData) -> Option<SpanRef<'_>> {
        if event.is_root() {
            None
        } else if event.is_contextual() {
            self.lookup_current()
        } else {
            self.span(&event.span()?)
        }
    }

    fn event_scope(&self, event: &dyn NewEventData) -> Option<Scope<'_>> {
        self.event_span(event).map(|span| span.scope())
    }

    fn downstream_event_span(&self, event: &dyn EventData) -> Option<SpanRef<'_>> {
        self.span(&event.span()?)
    }

    fn downstream_event_scope(&self, event: &dyn EventData) -> Option<Scope<'_>> {
        self.downstream_event_span(event).map(|span| span.scope())
    }
}

/// Span data and extensions.
#[derive(Clone)]
pub struct SpanRef<'l> {
    lookup: &'l dyn LookupSpan,
    data: Arc<dyn SpanRefData>,
}

impl fmt::Debug for SpanRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpanRef")
            .field("id", &self.data.id())
            .finish()
    }
}

impl<'l> SpanRef<'l> {
    /// Create a new `SpanRef` with provided `data` and [`LookupSpan`].
    pub fn new(lookup: &'l dyn LookupSpan, data: Arc<dyn SpanRefData>) -> Self {
        Self { lookup, data }
    }
}

impl<'l> SpanRef<'l> {
    /// Return `Id` of this span.
    pub fn id(&self) -> Id {
        self.data.id()
    }

    /// Return parent of this span if any.
    pub fn parent(&self) -> Option<SpanRef<'l>> {
        self.data
            .parent()
            .and_then(|id| self.lookup.span_data(&id))
            .map(|data| SpanRef {
                lookup: self.lookup,
                data,
            })
    }

    /// Retur extensions for this span.
    pub fn extensions(&self) -> Extensions<'_> {
        trace!("extensions");
        self.data.extensions()
    }

    /// Return mutable extensions for this span.
    pub fn extensions_mut(&mut self) -> ExtensionsMut<'_> {
        trace!("extensions_mut");
        self.data.extensions_mut()
    }

    pub fn scope(&self) -> Scope<'l> {
        Scope {
            lookup: self.lookup,
            next: Some(self.data.id()),
        }
    }
}

/// An iterator over the parents of a span, ordered from leaf to root.
///
/// This is returned from [`SpanRef::scope`].
#[derive(Clone)]
pub struct Scope<'l> {
    lookup: &'l dyn LookupSpan,
    next: Option<Id>,
}

impl<'l> Scope<'l> {
    /// Flip the order of the iterator to return spans from root to leaf.
    pub fn from_root(self) -> ScopeFromRoot<'l> {
        #[cfg(feature = "smallvec")]
        type Buf<T> = smallvec::SmallVec<T>;
        #[cfg(not(feature = "smallvec"))]
        type Buf<T> = Vec<T>;
        let spans = self.collect::<Buf<_>>().into_iter().rev();
        ScopeFromRoot { spans }
    }
}

impl<'l> Iterator for Scope<'l> {
    type Item = SpanRef<'l>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.lookup.span_data(self.next.as_ref()?)?;
        self.next = current.parent();

        Some(SpanRef {
            lookup: self.lookup,
            data: current,
        })
    }
}

/// An iterator over the parents of a span, ordered from root to leaf.
///
/// This is returned from [`Scope::from_root`].
#[derive(Clone)]
pub struct ScopeFromRoot<'l> {
    #[cfg(feature = "smallvec")]
    spans: std::iter::Rev<smallvec::IntoIter<SpanRefArray<'l>>>,
    #[cfg(not(feature = "smallvec"))]
    spans: std::iter::Rev<std::vec::IntoIter<SpanRef<'l>>>,
}

#[cfg(feature = "smallvec")]
type SpanRefArray<'l> = [SpanRef<'l>; 16];

impl<'l> Iterator for ScopeFromRoot<'l> {
    type Item = SpanRef<'l>;

    fn next(&mut self) -> Option<Self::Item> {
        self.spans.next()
    }
}

/// Helper trait for looking up spans.
pub trait LookupSpan {
    /// Lookup `SpanRefData` for provided `id`.
    fn span_data(&self, id: &Id) -> Option<Arc<dyn SpanRefData>>;
}

/// Implementation data for a `SpanRef`.
pub trait SpanRefData {
    /// Return `Id` of this span.
    fn id(&self) -> Id;

    /// Return parent of this span if any.
    fn parent(&self) -> Option<Id>;

    /// Return metadata of this span if known;
    fn metadata(&self) -> Option<&dyn Metadata>;

    /// Retur extensions for this span.
    fn extensions(&self) -> Extensions<'_>;

    /// Return mutable extensions for this span.
    fn extensions_mut(&self) -> ExtensionsMut<'_>;
}

pub struct LookupContext<'s, S> {
    subscriber: &'s S,
}

impl<'s, S: LookupSpan + Subscriber> LookupContext<'s, S> {
    pub fn new(subscriber: &'s S) -> Self {
        Self { subscriber }
    }
}

impl<S: LookupSpan + Subscriber> Context for LookupContext<'_, S> {
    fn current_span(&self) -> Current {
        self.subscriber.current_span()
    }

    fn span(&self, id: &Id) -> Option<SpanRef<'_>> {
        self.subscriber
            .span_data(id)
            .map(|data| SpanRef::new(self.subscriber, data))
    }
}
