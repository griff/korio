//! Subscribers collect and record data.

use std::any::{Any, TypeId};
use std::time::SystemTime;

use crate::dispatcher::Dispatch;
use crate::event::{EventData, NewEventData};
use crate::span::{Current, Id, NewSpanData, SpanData, SpanFollowsFromData, SpanRecordData};

/// Trait representing the functions required to collect data.
pub trait Subscriber: 'static {
    /// Record a remote upstream span and context.
    fn upstream_span(&self, id: &Id);

    /// Record a newly created span.
    fn new_span(&self, span: &dyn NewSpanData) -> Id;

    /// Record a remote downstream span.
    fn downstream_span(&self, span: &dyn SpanData);

    /// Record that a span was started.
    fn start(&self, id: &Id, start_time: SystemTime);

    /// Record that a span ended.
    fn end(&self, id: &Id, end_time: SystemTime);

    /// Record new attributes for a span
    fn record(&self, id: &Id, record: &dyn SpanRecordData);

    /// Record that a span follows from another span.
    fn record_follows_from(&self, id: &Id, follows_from: &dyn SpanFollowsFromData);

    /// Record a newly created event.
    fn new_event(&self, event: &dyn NewEventData);

    /// Record a remote downstream event.
    fn downstream_event(&self, event: &dyn EventData);

    /// Record that a span has been entered.
    fn enter(&self, id: &Id);

    /// Record that a span has been exited.
    fn exit(&self, id: &Id);

    /// Record that a span has been cloned.
    fn clone_span(&self, id: &Id) -> Id {
        id.clone()
    }

    /// Record that a given span has been dropped and possibly need to close.
    ///
    /// **Note**: since this function is called when spans are dropped,
    /// implementations should ensure that they are unwind-safe. Panicking from
    /// inside of a `try_close` function may cause a double panic, if the span
    /// was dropped due to a thread unwinding.
    fn try_close(&self, id: Id) -> bool {
        let _ = id;
        true
    }

    /// Return the current span if defined.
    fn current_span(&self) -> Current {
        Current::unknown()
    }

    /// Invoked when this subscriber becomes a Dispatch.
    fn on_register_dispatch(&self, dispatch: &Dispatch) {
        let _ = dispatch;
    }

    /// # Safety
    ///
    /// The [`downcast_ref`] method expects that the pointer returned by
    /// `downcast_raw` is non-null and points to a valid instance of the type
    /// with the provided `TypeId`. Failure to ensure this will result in
    /// undefined behaviour, so implementing `downcast_raw` is unsafe.
    ///
    /// [`downcast_ref`]: #method.downcast_ref
    unsafe fn downcast_raw(&self, id: TypeId) -> Option<*const ()> {
        if id == TypeId::of::<Self>() {
            Some(self as *const Self as *const ())
        } else {
            None
        }
    }
}

impl dyn Subscriber {
    /// Returns `true` if this `Subscriber` is the same type as `T`.
    pub fn is<T: Any>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }

    /// Returns some reference to this `Subscriber` value if it is of type `T`,
    /// or `None` if it isn't.
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        unsafe {
            let raw = self.downcast_raw(TypeId::of::<T>())?;
            if raw.is_null() {
                None
            } else {
                Some(&*(raw as *const _))
            }
        }
    }
}

impl dyn Subscriber + Send {
    /// Returns `true` if this [`Subscriber`] is the same type as `T`.
    pub fn is<T: Any>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }

    /// Returns some reference to this [`Subscriber`] value if it is of type `T`,
    /// or `None` if it isn't.
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        unsafe {
            let raw = self.downcast_raw(TypeId::of::<T>())?;
            if raw.is_null() {
                None
            } else {
                Some(&*(raw as *const _))
            }
        }
    }
}

impl dyn Subscriber + Sync {
    /// Returns `true` if this [`Subscriber`] is the same type as `T`.
    pub fn is<T: Any>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }

    /// Returns some reference to this `[`Subscriber`] value if it is of type `T`,
    /// or `None` if it isn't.
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        unsafe {
            let raw = self.downcast_raw(TypeId::of::<T>())?;
            if raw.is_null() {
                None
            } else {
                Some(&*(raw as *const _))
            }
        }
    }
}

impl dyn Subscriber + Send + Sync {
    /// Returns `true` if this [`Subscriber`] is the same type as `T`.
    pub fn is<T: Any>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }

    /// Returns some reference to this [`Subscriber`] value if it is of type `T`,
    /// or `None` if it isn't.
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        unsafe {
            let raw = self.downcast_raw(TypeId::of::<T>())?;
            if raw.is_null() {
                None
            } else {
                Some(&*(raw as *const _))
            }
        }
    }
}

/// A no-op [`Subscriber`].
///
/// [`NoSubscriber`] implements the [`Subscriber`] trait by never being enabled,
/// never being interested in any callsite, and dropping all spans and events.
#[derive(Copy, Clone, Debug, Default)]
pub struct NoSubscriber(());

impl Subscriber for NoSubscriber {
    fn new_span(&self, _: &dyn NewSpanData) -> Id {
        Id {
            trace_id: crate::span::TraceId::from_u128(0xDEADBEEF),
            span_id: crate::span::SpanId::from_u64(0xDEAD),
        }
    }
    fn upstream_span(&self, _: &Id) {}
    fn downstream_span(&self, _: &dyn SpanData) {}
    fn start(&self, _: &Id, _: SystemTime) {}
    fn end(&self, _: &Id, _: SystemTime) {}
    fn record(&self, _: &Id, _: &dyn SpanRecordData) {}

    fn record_follows_from(&self, _: &Id, _: &dyn SpanFollowsFromData) {}
    fn new_event(&self, event: &dyn NewEventData) {
        eprintln!(
            "No subscriber new event {} {}",
            event.target(),
            event.name()
        );
    }
    fn downstream_event(&self, _: &dyn EventData) {}
    fn enter(&self, _: &Id) {}
    fn exit(&self, _: &Id) {}
}

impl NoSubscriber {
    /// Returns a new `NoSubscriber`.
    #[must_use]
    pub const fn new() -> Self {
        Self(())
    }
}
