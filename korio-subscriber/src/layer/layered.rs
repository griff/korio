use std::any::{Any, TypeId};
use std::fmt;
use std::marker::PhantomData;
use std::time::SystemTime;

use korio_core::Subscriber;
use korio_core::event::{EventData, NewEventData};
use korio_core::span::{Current, Id, NewSpanData, SpanData, SpanFollowsFromData, SpanRecordData};

use super::{Context, Layer, LookupContext, LookupSpan, Observer};
use crate::Registry;

pub struct Layered<L, I, S = I> {
    layer: L,
    inner: I,
    _s: PhantomData<fn(S)>,
}

impl<L, I, S> Layered<L, I, S> {
    pub(crate) fn new(layer: L, inner: I) -> Self {
        Self {
            layer,
            inner,
            _s: PhantomData,
        }
    }
}

impl<L, S> Layered<L, S>
where
    L: Layer<S>,
    S: Subscriber + LookupSpan,
{
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
                Some(&*(raw as *const T))
            }
        }
    }
}

impl<L, S> Subscriber for Layered<L, S>
where
    L: Layer<S>,
    S: Subscriber + LookupSpan,
{
    fn upstream_span(&self, id: &Id) {
        self.inner.upstream_span(id);
        self.layer.on_upstream_span(id, &self.ctx());
    }

    fn new_span(&self, span: &dyn NewSpanData) -> Id {
        let id = self.inner.new_span(span);
        self.layer.on_new_span(&id, &span, &self.ctx());
        id
    }

    fn downstream_span(&self, span: &dyn SpanData) {
        self.inner.downstream_span(span);
        self.layer.on_downstream_span(span, &self.ctx());
    }

    fn start(&self, id: &Id, start_time: SystemTime) {
        self.inner.start(id, start_time);
        self.layer.on_start(id, start_time, &self.ctx());
    }

    fn end(&self, id: &Id, end_time: SystemTime) {
        self.inner.end(id, end_time);
        self.layer.on_end(id, end_time, &self.ctx());
    }

    fn record(&self, id: &Id, record: &dyn SpanRecordData) {
        self.inner.record(id, record);
        self.layer.on_record(id, record, &self.ctx());
    }

    fn record_follows_from(&self, id: &Id, follows_from: &dyn SpanFollowsFromData) {
        self.inner.record_follows_from(id, follows_from);
        self.layer
            .on_record_follows_from(id, follows_from, &self.ctx());
    }

    fn new_event(&self, event: &dyn NewEventData) {
        eprintln!("Layered new event {} {}", event.target(), event.name());
        self.inner.new_event(event);
        self.layer.on_new_event(event, &self.ctx());
    }

    fn downstream_event(&self, event: &dyn EventData) {
        self.inner.downstream_event(event);
        self.layer.on_downstream_event(event, &self.ctx());
    }

    fn enter(&self, id: &Id) {
        self.inner.enter(id);
        self.layer.on_enter(id, &self.ctx());
    }

    fn exit(&self, id: &Id) {
        self.inner.exit(id);
        self.layer.on_exit(id, &self.ctx());
    }

    fn clone_span(&self, old: &Id) -> Id {
        let new = self.inner.clone_span(old);
        if &new != old {
            self.layer.on_id_change(old, &new, &self.ctx());
        }
        new
    }

    fn try_close(&self, id: Id) -> bool {
        let subscriber = &self.inner as &dyn Subscriber;
        let mut guard = subscriber
            .downcast_ref::<Registry>()
            .map(|registry| registry.start_close(id.clone()));

        if self.inner.try_close(id.clone()) {
            // If we have a registry's close guard, indicate that the span is
            // closing.
            if let Some(g) = guard.as_mut() {
                g.set_closing()
            };
            self.layer.on_close(&id, &self.ctx());
            true
        } else {
            false
        }
    }

    fn current_span(&self) -> Current {
        self.inner.current_span()
    }

    fn on_register_dispatch(&self, dispatch: &korio_core::dispatcher::Dispatch) {
        self.inner.on_register_dispatch(dispatch);
        self.layer.on_register_dispatch(dispatch);
    }

    #[doc(hidden)]
    unsafe fn downcast_raw(&self, id: TypeId) -> Option<*const ()> {
        // If downcasting to `Self`, return a pointer to `self`.
        if id == TypeId::of::<Self>() {
            return Some(self as *const _ as *const ());
        }

        unsafe { self.layer.downcast_raw(id) }.or_else(|| unsafe { self.inner.downcast_raw(id) })
    }
}

impl<A, B, S> Layer<S> for Layered<A, B, S>
where
    A: Layer<S>,
    B: Layer<S>,
    S: Subscriber,
{
    fn on_register_dispatch(&self, dispatch: &korio_core::dispatcher::Dispatch) {
        self.layer.on_register_dispatch(dispatch);
        self.inner.on_register_dispatch(dispatch);
    }

    fn on_layer(&mut self, subscriber: &mut S) {
        self.layer.on_layer(subscriber);
        self.inner.on_layer(subscriber);
    }
}

impl<A, B, S> Observer for Layered<A, B, S>
where
    A: Layer<S>,
    B: Layer<S>,
    S: Subscriber,
{
    #[inline]
    fn on_new_span(&self, id: &Id, span: &dyn NewSpanData, ctx: &dyn Context) {
        self.inner.on_new_span(id, span, ctx);
        self.layer.on_new_span(id, span, ctx);
    }

    #[inline]
    fn on_downstream_span(&self, span: &dyn SpanData, ctx: &dyn Context) {
        self.inner.on_downstream_span(span, ctx);
        self.layer.on_downstream_span(span, ctx);
    }

    #[inline]
    fn on_upstream_span(&self, id: &Id, ctx: &dyn Context) {
        self.inner.on_upstream_span(id, ctx);
        self.layer.on_upstream_span(id, ctx);
    }

    #[inline]
    fn on_start(&self, id: &Id, start_time: SystemTime, ctx: &dyn Context) {
        self.inner.on_start(id, start_time, ctx);
        self.layer.on_start(id, start_time, ctx);
    }

    #[inline]
    fn on_end(&self, id: &Id, end_time: SystemTime, ctx: &dyn Context) {
        self.inner.on_end(id, end_time, ctx);
        self.layer.on_end(id, end_time, ctx);
    }

    #[inline]
    fn on_record(&self, id: &Id, record: &dyn SpanRecordData, ctx: &dyn Context) {
        self.inner.on_record(id, record, ctx);
        self.layer.on_record(id, record, ctx);
    }

    #[inline]
    fn on_record_follows_from(&self, id: &Id, link: &dyn SpanFollowsFromData, ctx: &dyn Context) {
        self.inner.on_record_follows_from(id, link, ctx);
        self.layer.on_record_follows_from(id, link, ctx);
    }

    #[inline]
    fn on_new_event(&self, event: &dyn NewEventData, ctx: &dyn Context) {
        self.inner.on_new_event(event, ctx);
        self.layer.on_new_event(event, ctx);
    }

    #[inline]
    fn on_downstream_event(&self, event: &dyn EventData, ctx: &dyn Context) {
        self.inner.on_downstream_event(event, ctx);
        self.layer.on_downstream_event(event, ctx);
    }

    #[inline]
    fn on_enter(&self, id: &Id, ctx: &dyn Context) {
        self.inner.on_enter(id, ctx);
        self.layer.on_enter(id, ctx);
    }

    #[inline]
    fn on_exit(&self, id: &Id, ctx: &dyn Context) {
        self.inner.on_exit(id, ctx);
        self.layer.on_exit(id, ctx);
    }

    #[inline]
    fn on_close(&self, id: &Id, ctx: &dyn Context) {
        self.inner.on_close(id, ctx);
        self.layer.on_close(id, ctx);
    }

    #[inline]
    fn on_id_change(&self, old: &Id, new: &Id, ctx: &dyn Context) {
        self.inner.on_id_change(old, new, ctx);
        self.layer.on_id_change(old, new, ctx);
    }

    #[doc(hidden)]
    unsafe fn downcast_raw(&self, id: TypeId) -> Option<*const ()> {
        match id {
            // If downcasting to `Self`, return a pointer to `self`.
            id if id == TypeId::of::<Self>() => Some(self as *const _ as *const ()),

            // Otherwise, try to downcast both branches normally...
            _ => unsafe { self.layer.downcast_raw(id) }
                .or_else(|| unsafe { self.inner.downcast_raw(id) }),
        }
    }
}

impl<A, B, S> fmt::Debug for Layered<A, B, S>
where
    A: fmt::Debug,
    B: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Layered")
            .field("layer", &self.layer)
            .field("inner", &self.inner)
            .finish()
    }
}

impl<L, S> Layered<L, S>
where
    S: Subscriber + LookupSpan,
{
    fn ctx(&self) -> LookupContext<'_, S> {
        LookupContext::new(&self.inner)
    }
}
