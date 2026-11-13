pub mod wrapper;

use std::marker::PhantomData;

use korio_core::Capture;
use tracing::{Dispatch, Subscriber};
use tracing_subscriber::{
    Layer,
    registry::{LookupSpan, Scope},
};

struct Captures(Vec<Capture>);

fn capture_iter<'a, R>(scope: Scope<'a, R>, f: impl Fn(&Capture))
where
    R: LookupSpan<'a>,
{
    for span in scope {
        let ext = span.extensions();
        let Some(Captures(captures)) = ext.get::<Captures>() else {
            continue;
        };
        for capture in captures {
            f(capture)
        }
    }
}

pub struct CaptureLayer<S> {
    get_context: WithRegisterContext,
    _registry: PhantomData<fn(S)>,
}
impl<S> CaptureLayer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    pub fn new() -> Self {
        Self {
            get_context: WithRegisterContext {
                register_capture: Self::register_capture,
            },
            _registry: PhantomData,
        }
    }

    fn register_capture(dispatch: &Dispatch, id: &tracing::span::Id, capture: &Capture) {
        let me = dispatch
            .downcast_ref::<Self>()
            .expect("subscriber should downcast to expected type; this is a bug!");
        let subscriber = dispatch
            .downcast_ref::<S>()
            .expect("subscriber should downcast to expected type; this is a bug!");
        let span = subscriber
            .span(id)
            .expect("registry should have a span for the current ID");
        let mut ext = span.extensions_mut();
        if let Some(Captures(captures)) = ext.get_mut::<Captures>() {
            captures.push(capture.clone());
        } else {
            ext.insert(Captures(vec![capture.clone()]));
        }
    }
}

impl<S> Layer<S> for CaptureLayer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let Some(scope) = ctx.span_scope(id) else {
            return;
        };
        capture_iter(scope, |capture| {
            //    capture.on_new_span(attrs, id);
        });
    }

    fn on_record(
        &self,
        _span: &tracing::span::Id,
        _values: &tracing::span::Record<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
    }

    fn on_follows_from(
        &self,
        _span: &tracing::span::Id,
        _follows: &tracing::span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
    }

    fn event_enabled(
        &self,
        event: &tracing::Event<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        let Some(mut scope) = ctx.event_scope(event) else {
            return false;
        };
        scope.any(|span| span.extensions().get::<Captures>().is_some())
    }

    fn on_event(
        &self,
        _event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
    }

    fn on_enter(&self, _id: &tracing::span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {}

    fn on_exit(&self, _id: &tracing::span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {}

    fn on_close(&self, _id: tracing::span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {}

    fn on_id_change(
        &self,
        _old: &tracing::span::Id,
        _new: &tracing::span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
    }
}

pub(crate) struct WithRegisterContext {
    register_capture: fn(&Dispatch, &tracing::span::Id, capture: &Capture),
}

impl WithRegisterContext {
    pub(crate) fn register_capture(
        &self,
        dispatch: &Dispatch,
        id: &tracing::span::Id,
        capture: &Capture,
    ) {
        (self.register_capture)(dispatch, id, capture)
    }
}

pub trait SpanCaptureExt {
    /// Observe this span using provided `capture`.
    fn capture(&self, capture: &Capture);
}

impl SpanCaptureExt for tracing::span::Span {
    fn capture(&self, capture: &Capture) {
        self.with_subscriber(|(id, dispatch)| {
            let Some(ctx) = dispatch.downcast_ref::<WithRegisterContext>() else {
                return;
            };
            ctx.register_capture(dispatch, id, capture);
        });
    }
}
