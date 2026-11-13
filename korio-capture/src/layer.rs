use std::{any::TypeId, fmt, marker::PhantomData};

use korio::{Subscriber, dispatcher::Dispatch, span::Id};
use korio_subscriber::{
    Layer,
    layer::{Context, LookupContext, LookupSpan, Observer},
};
use tracing::trace;

use crate::Capture;

pub struct CaptureLayer<S> {
    get_register: WithRegisterCapture,
    _registry: PhantomData<fn(S)>,
}

impl<S> CaptureLayer<S>
where
    S: LookupSpan + Subscriber + 'static,
{
    pub fn new() -> Self {
        Self {
            get_register: WithRegisterCapture {
                capture_fn: Self::capture,
            },
            _registry: PhantomData,
        }
    }
}

impl<S> CaptureLayer<S>
where
    S: LookupSpan + Subscriber + 'static,
{
    fn capture(dispatch: &Dispatch, id: &Id, capture: &Capture) {
        trace!(%id, "looking up subscriber");
        let subscriber = dispatch
            .downcast_ref::<S>()
            .expect("subscriber should downcast to expected type; this is a bug!");
        let ctx = LookupContext::new(subscriber);
        {
            trace!(%id, "looking up span");
            let mut span = ctx
                .span(id)
                .expect("registry should have a span for the current ID");
            let mut ext = span.extensions_mut();
            trace!(%id, "get captures");
            if let Some(Captures(captures)) = ext.get_mut::<Captures>() {
                trace!(%id, "push capture");
                captures.push(capture.clone());
            } else {
                trace!(%id, "create captures");
                ext.insert(Captures(vec![capture.clone()]));
            }
            trace!(%id, "call on_capture");
        }
        capture.on_capture(id, &ctx);
    }
}

impl<S> Default for CaptureLayer<S>
where
    S: LookupSpan + Subscriber + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> fmt::Debug for CaptureLayer<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CaptureLayer")
            .field("_registry", &self._registry)
            .finish()
    }
}

impl<S: Subscriber> Layer<S> for CaptureLayer<S> {}

impl<S: 'static> Observer for CaptureLayer<S> {
    fn on_new_span(&self, id: &Id, span: &dyn korio::span::NewSpanData, ctx: &dyn Context) {
        let parent = std::fmt::from_fn(|f| match span.parent() {
            Some(span) => write!(f, "{span}"),
            None => f.write_str("nil"),
        });
        trace!(span.name = span.name(), span.id=%id, span.parent=%parent, "on_new_span");
        let span_ref = ctx.span(id).expect("Subscriber is missing provided span");
        for span_ref in span_ref.scope() {
            let captures = span_ref.extensions().get::<Captures>().cloned();
            if let Some(captures) = captures {
                for capture in captures.iter() {
                    capture.on_new_span(id, span, ctx);
                }
            }
        }
    }

    fn on_downstream_span(&self, span: &dyn korio::span::SpanData, ctx: &dyn Context) {
        let span_ref = ctx
            .span(span.id())
            .expect("Subscriber is missing provided span");
        for span_ref in span_ref.scope() {
            let captures = span_ref.extensions().get::<Captures>().cloned();
            if let Some(captures) = captures {
                for capture in captures.iter() {
                    capture.on_downstream_span(span, ctx);
                }
            }
        }
    }

    fn on_upstream_span(&self, id: &Id, ctx: &dyn Context) {
        let span_ref = ctx.span(id).expect("Subscriber is missing provided span");
        for span_ref in span_ref.scope() {
            let captures = span_ref.extensions().get::<Captures>().cloned();
            if let Some(captures) = captures {
                for capture in captures.iter() {
                    capture.on_upstream_span(id, ctx);
                }
            }
        }
    }

    fn on_start(&self, id: &Id, start_time: std::time::SystemTime, ctx: &dyn Context) {
        trace!(span.id = %id, span.start_time = ?start_time, "on_start");
        let span_ref = ctx.span(id).expect("Subscriber is missing provided span");
        for span_ref in span_ref.scope() {
            let captures = span_ref.extensions().get::<Captures>().cloned();
            if let Some(captures) = captures {
                for capture in captures.iter() {
                    capture.on_start(id, start_time, ctx);
                }
            }
        }
    }

    fn on_end(&self, id: &Id, end_time: std::time::SystemTime, ctx: &dyn Context) {
        trace!(span.id = %id, span.end_time = ?end_time, "on_end");
        let span_ref = ctx.span(id).expect("Subscriber is missing provided span");
        for span_ref in span_ref.scope() {
            let captures = span_ref.extensions().get::<Captures>().cloned();
            if let Some(captures) = captures {
                for capture in captures.iter() {
                    capture.on_end(id, end_time, ctx);
                }
            }
        }
    }

    fn on_record(&self, id: &Id, record: &dyn korio::span::SpanRecordData, ctx: &dyn Context) {
        let span_ref = ctx.span(id).expect("Subscriber is missing provided span");
        for span_ref in span_ref.scope() {
            let captures = span_ref.extensions().get::<Captures>().cloned();
            if let Some(captures) = captures {
                for capture in captures.iter() {
                    capture.on_record(id, record, ctx);
                }
            }
        }
    }

    fn on_record_follows_from(
        &self,
        id: &Id,
        link: &dyn korio::span::SpanFollowsFromData,
        ctx: &dyn Context,
    ) {
        let span_ref = ctx.span(id).expect("Subscriber is missing provided span");
        for span_ref in span_ref.scope() {
            let captures = span_ref.extensions().get::<Captures>().cloned();
            if let Some(captures) = captures {
                for capture in captures.iter() {
                    capture.on_record_follows_from(id, link, ctx);
                }
            }
        }
    }

    fn on_new_event(&self, event: &dyn korio::event::NewEventData, ctx: &dyn Context) {
        eprintln!("Capture new event {} {}", event.target(), event.name());
        let span = std::fmt::from_fn(|f| match event.span() {
            Some(span) => write!(f, "{span}"),
            None => f.write_str("nil"),
        });
        trace!(
            event.target = event.target(),
            event.name = event.name(),
            event.span = %span,
            "on_new_event"
        );
        if let Some(span_ref) = ctx.event_span(event) {
            for span_ref in span_ref.scope() {
                let captures = span_ref.extensions().get::<Captures>().cloned();
                if let Some(captures) = captures {
                    for capture in captures.iter() {
                        trace!("Sending capture");
                        capture.on_new_event(event, ctx);
                    }
                }
            }
        }
    }

    fn on_downstream_event(&self, event: &dyn korio::event::EventData, ctx: &dyn Context) {
        let span = std::fmt::from_fn(|f| match event.span() {
            Some(span) => write!(f, "{span}"),
            None => f.write_str("nil"),
        });
        trace!(
            event.target = event.target(),
            event.name = event.name(),
            event.span = %span,
            "on_downstream_event"
        );
        if let Some(span_ref) = ctx.downstream_event_span(event) {
            for span_ref in span_ref.scope() {
                let captures = span_ref.extensions().get::<Captures>().cloned();
                if let Some(captures) = captures {
                    for capture in captures.iter() {
                        trace!("Sending capture");
                        capture.on_downstream_event(event, ctx);
                    }
                }
            }
        }
    }

    fn on_enter(&self, id: &Id, ctx: &dyn Context) {
        trace!(span.id = %id, "on_enter");
        let span_ref = ctx.span(id).expect("Subscriber is missing provided span");
        for span_ref in span_ref.scope() {
            let captures = span_ref.extensions().get::<Captures>().cloned();
            if let Some(captures) = captures {
                for capture in captures.iter() {
                    capture.on_enter(id, ctx);
                }
            }
        }
    }

    fn on_exit(&self, id: &Id, ctx: &dyn Context) {
        trace!(span.id = %id, "on_exit");
        let span_ref = ctx.span(id).expect("Subscriber is missing provided span");
        for span_ref in span_ref.scope() {
            let captures = span_ref.extensions().get::<Captures>().cloned();
            if let Some(captures) = captures {
                for capture in captures.iter() {
                    capture.on_exit(id, ctx);
                }
            }
        }
    }

    fn on_close(&self, id: &Id, ctx: &dyn Context) {
        trace!(span.id = %id, "on_close");
        let span_ref = ctx.span(id).expect("Subscriber is missing provided span");
        for span_ref in span_ref.scope() {
            let captures = span_ref.extensions().get::<Captures>().cloned();
            if let Some(captures) = captures {
                for capture in captures.iter() {
                    capture.on_close(id, ctx);
                }
            }
        }
    }

    fn on_id_change(&self, old: &Id, new: &Id, ctx: &dyn Context) {
        let span_ref = ctx.span(old).expect("Subscriber is missing provided span");
        for span_ref in span_ref.scope() {
            let captures = span_ref.extensions().get::<Captures>().cloned();
            if let Some(captures) = captures {
                for capture in captures.iter() {
                    capture.on_id_change(old, new, ctx);
                }
            }
        }
    }

    // SAFETY: this is safe because the `WithRegisterCapture` function pointer is valid
    // for the lifetime of `&self`.
    unsafe fn downcast_raw(&self, id: TypeId) -> Option<*const ()> {
        match id {
            id if id == TypeId::of::<Self>() => Some(self as *const _ as *const ()),
            id if id == TypeId::of::<WithRegisterCapture>() => {
                Some(&self.get_register as *const _ as *const ())
            }
            _ => None,
        }
    }
}

#[derive(Clone)]
struct Captures(Vec<Capture>);

impl Captures {
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Capture> {
        self.0.iter()
    }
}

pub struct WithRegisterCapture {
    capture_fn: fn(&Dispatch, &Id, capture: &Capture),
}

impl WithRegisterCapture {
    pub(crate) fn capture(&self, dispatch: &Dispatch, id: &Id, capture: &Capture) {
        (self.capture_fn)(dispatch, id, capture)
    }
}

pub trait SpanCaptureExt {
    /// Observe this span using provided `capture`.
    fn capture(&self, capture: &Capture);
}

impl SpanCaptureExt for korio::span::Span {
    fn capture(&self, capture: &Capture) {
        self.with_subscriber(|(id, dispatch)| {
            let Some(with_capture) = dispatch.downcast_ref::<WithRegisterCapture>() else {
                return;
            };
            trace!(%id, "calling capture on subscriber");
            with_capture.capture(dispatch, id, capture);
        });
    }
}
