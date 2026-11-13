//! Capture spans and events for export
use std::any::Any;
use std::ops::Deref as _;
use std::sync::{Arc, Weak};
use std::time::SystemTime;

use korio::dispatcher::Dispatch;
use korio::span::{Id, NewSpanData, SpanFollowsFromData, SpanRecordData};
use korio_subscriber::Registry;
use korio_subscriber::layer::{Context, Observer, SubscriberExt};

pub mod export;
pub mod extract;
mod layer;

pub use export::Exporter;
pub use layer::{CaptureLayer, SpanCaptureExt};

/// Trait for observing and capturing things related to spans.
pub trait CaptureObserver: Observer {
    /// This observer became part of a capture.
    fn on_register_capture(&self, capture: &Capture) {
        let _ = capture;
    }

    fn on_capture(&self, id: &Id, capture: &Capture, ctx: &dyn Context) {
        let _ = (id, capture, ctx);
    }
}

/// Capture all things related to a span, its children and any events connected to it.
#[derive(Debug, Clone)]
pub struct Capture {
    inner: Arc<dyn CaptureObserver + Send + Sync>,
}

impl Capture {
    /// Create a new `Capture` for the provided [`Observer`].
    pub fn new<O>(observer: O) -> Self
    where
        O: CaptureObserver + Send + Sync + 'static,
    {
        let this = Self {
            inner: Arc::new(observer),
        };
        this.inner.on_register_capture(&this);
        this
    }

    pub(crate) fn on_capture(&self, id: &Id, ctx: &dyn Context) {
        self.inner.on_capture(id, self, ctx);
    }

    /// Downgrade this capture to a [`WeakCapture`].
    pub fn downgrade(&self) -> WeakCapture {
        WeakCapture {
            inner: Arc::downgrade(&self.inner),
        }
    }

    fn observer(&self) -> &(dyn Observer + Send + Sync) {
        self.inner.deref()
    }

    /// Return `true` if this `Dispatch` forwards to a [`Subscriber`] of type `T`.
    pub fn is<T: Any>(&self) -> bool {
        self.observer().is::<T>()
    }

    /// Returns some reference that this `Dispatch` points to if it is of type `T`, or `None` otherwise.
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.observer().downcast_ref::<T>()
    }
}

impl Observer for Capture {
    /// A span was created.
    fn on_new_span(&self, id: &Id, span: &dyn NewSpanData, ctx: &dyn Context) {
        self.inner.on_new_span(id, span, ctx);
    }

    fn on_downstream_span(&self, span: &dyn korio::span::SpanData, ctx: &dyn Context) {
        let _ = (span, ctx);
    }

    fn on_upstream_span(&self, id: &Id, ctx: &dyn Context) {
        let _ = (id, ctx);
    }

    /// A span was started.
    fn on_start(&self, id: &Id, start_time: SystemTime, ctx: &dyn Context) {
        self.inner.on_start(id, start_time, ctx);
    }

    /// A span ended.
    fn on_end(&self, id: &Id, end_time: SystemTime, ctx: &dyn Context) {
        self.inner.on_end(id, end_time, ctx);
    }

    /// A span recorded new attributes.
    fn on_record(&self, id: &Id, record: &dyn SpanRecordData, ctx: &dyn Context) {
        self.inner.on_record(id, record, ctx);
    }

    /// A span recorded that it follows from another span.
    fn on_record_follows_from(&self, id: &Id, link: &dyn SpanFollowsFromData, ctx: &dyn Context) {
        self.inner.on_record_follows_from(id, link, ctx);
    }

    /// A span was entered.
    fn on_enter(&self, id: &Id, ctx: &dyn Context) {
        self.inner.on_enter(id, ctx);
    }

    /// A span exited.
    fn on_exit(&self, id: &Id, ctx: &dyn Context) {
        self.inner.on_exit(id, ctx);
    }

    /// A span was closed.
    fn on_close(&self, id: &Id, ctx: &dyn Context) {
        self.inner.on_close(id, ctx);
    }

    fn on_new_event(&self, event: &dyn korio::event::NewEventData, ctx: &dyn Context) {
        self.inner.on_new_event(event, ctx);
    }

    fn on_downstream_event(&self, event: &dyn korio::event::EventData, ctx: &dyn Context) {
        self.inner.on_downstream_event(event, ctx);
    }

    fn on_id_change(&self, old: &Id, new: &Id, ctx: &dyn Context) {
        self.inner.on_id_change(old, new, ctx);
    }

    unsafe fn downcast_raw(&self, id: std::any::TypeId) -> Option<*const ()> {
        if id == std::any::TypeId::of::<Self>() {
            Some(self as *const _ as *const ())
        } else {
            // SAFETY: Passing to the inner observer.
            unsafe { self.inner.downcast_raw(id) }
        }
    }
}

/// A version of [`Capture`] that holds a non-owning reference to a [`CaptureObserver`].
#[derive(Debug, Clone)]
pub struct WeakCapture {
    inner: Weak<dyn CaptureObserver + Send + Sync>,
}

impl WeakCapture {
    /// Attempt to upgrade this `WeakCapture` to a [`Capture`].
    ///
    /// Return `None` if the referenced `Capture` has been dropped.
    pub fn upgrade(&self) -> Option<Capture> {
        self.inner.upgrade().map(|inner| Capture { inner })
    }
}

pub fn default_capture_dispatch() -> Dispatch {
    Dispatch::new(Registry::new().with(CaptureLayer::new()))
}
