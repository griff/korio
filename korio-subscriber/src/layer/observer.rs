use std::{
    any::{Any, TypeId},
    time::SystemTime,
};

use korio_core::{
    event::{EventData, NewEventData},
    span::{Id, NewSpanData, SpanData, SpanFollowsFromData, SpanRecordData},
};

use crate::layer::Context;

/// Trait for observing things related to spans.
pub trait Observer: std::fmt::Debug + 'static {
    /// A span was created.
    fn on_new_span(&self, id: &Id, span: &dyn NewSpanData, ctx: &dyn Context) {
        let _ = (id, span, ctx);
    }

    /// A downstream span was created.
    fn on_downstream_span(&self, span: &dyn SpanData, ctx: &dyn Context) {
        let _ = (span, ctx);
    }

    /// A upstream span was registered.
    fn on_upstream_span(&self, id: &Id, ctx: &dyn Context) {
        let _ = (id, ctx);
    }

    /// A span was started.
    fn on_start(&self, id: &Id, start_time: SystemTime, ctx: &dyn Context) {
        let _ = (id, start_time, ctx);
    }

    /// A span ended.
    fn on_end(&self, id: &Id, end_time: SystemTime, ctx: &dyn Context) {
        let _ = (id, end_time, ctx);
    }

    /// A span recorded new attributes.
    fn on_record(&self, id: &Id, record: &dyn SpanRecordData, ctx: &dyn Context) {
        let _ = (id, record, ctx);
    }

    /// A span recorded that it follows from another span.
    fn on_record_follows_from(&self, id: &Id, link: &dyn SpanFollowsFromData, ctx: &dyn Context) {
        let _ = (id, link, ctx);
    }

    /// A new event happened.
    fn on_new_event(&self, event: &dyn NewEventData, ctx: &dyn Context) {
        let _ = (event, ctx);
    }

    /// A new event happened.
    fn on_downstream_event(&self, event: &dyn EventData, ctx: &dyn Context) {
        let _ = (event, ctx);
    }

    /// A span was entered.
    fn on_enter(&self, id: &Id, ctx: &dyn Context) {
        let _ = (id, ctx);
    }

    /// A span exited.
    fn on_exit(&self, id: &Id, ctx: &dyn Context) {
        let _ = (id, ctx);
    }

    /// A span was closed.
    fn on_close(&self, id: &Id, ctx: &dyn Context) {
        let _ = (id, ctx);
    }

    /// The id of a span changed.
    fn on_id_change(&self, old: &Id, new: &Id, ctx: &dyn Context) {
        let _ = (old, new, ctx);
    }

    #[doc(hidden)]
    unsafe fn downcast_raw(&self, id: TypeId) -> Option<*const ()> {
        if id == TypeId::of::<Self>() {
            Some(self as *const _ as *const ())
        } else {
            None
        }
    }
}

impl dyn Observer {
    /// Returns `true` if this `Observer` is the same type as `T`.
    pub fn is<T: Any>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }

    /// Returns some reference to this `Observer` value if it is of type `T`,
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

impl dyn Observer + Send {
    /// Returns `true` if this [`Observer`] is the same type as `T`.
    pub fn is<T: Any>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }

    /// Returns some reference to this [`Observer`] value if it is of type `T`,
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

impl dyn Observer + Sync {
    /// Returns `true` if this [`Observer`] is the same type as `T`.
    pub fn is<T: Any>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }

    /// Returns some reference to this `[`Observer`] value if it is of type `T`,
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

impl dyn Observer + Send + Sync {
    /// Returns `true` if this [`Observer`] is the same type as `T`.
    pub fn is<T: Any>(&self) -> bool {
        self.downcast_ref::<T>().is_some()
    }

    /// Returns some reference to this [`Observer`] value if it is of type `T`,
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
