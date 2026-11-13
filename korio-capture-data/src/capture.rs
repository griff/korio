use std::any::TypeId;
use std::time::SystemTime;

use korio::Subscriber;
use korio::event::NewEventData;
use korio::span::{Id, NewSpanData, SpanRecordData};
use korio_capture::{CaptureObserver, Exporter};
use korio_data::event::Event;
use korio_data::span::{CreatedSpan, SpanRecord};
use korio_data::telemetry_event::TelemetryEvent;
use korio_subscriber::Layer;
use korio_subscriber::layer::{Context, Observer};
use tracing::trace;

#[derive(Debug)]
pub struct TelemetryEventCapture<E> {
    exporter: E,
}

impl<E: Exporter<TelemetryEvent>> TelemetryEventCapture<E> {
    pub fn with_exporter(exporter: E) -> Self {
        Self { exporter }
    }
}

impl<E> CaptureObserver for TelemetryEventCapture<E> where
    E: Exporter<TelemetryEvent> + std::fmt::Debug + 'static
{
}

impl<E, S> Layer<S> for TelemetryEventCapture<E>
where
    E: Exporter<TelemetryEvent> + std::fmt::Debug + 'static,
    S: Subscriber,
{
}

impl<E> Observer for TelemetryEventCapture<E>
where
    E: Exporter<TelemetryEvent> + std::fmt::Debug + 'static,
{
    fn on_new_span(&self, id: &Id, span: &dyn NewSpanData, ctx: &dyn Context) {
        trace!(span.id=%id, span.target=span.target(), span.name=span.name(), "on_new_span");
        let mut span_ref = ctx.span(id).expect("Subscriber is missing provided span");
        let parent_id = span_ref.parent().map(|span| span.id());
        let span = korio_data::span::CreatedSpan::from_new_data(id.clone(), parent_id, span);
        span_ref.extensions_mut().insert(span);
    }

    fn on_downstream_span(&self, span: &dyn korio::span::SpanData, ctx: &dyn Context) {
        let mut span_ref = ctx
            .span(span.id())
            .expect("Subscriber is missing provided span");
        let span = CreatedSpan::from_data(span);
        span_ref.extensions_mut().insert(span);
    }

    fn on_start(&self, id: &Id, start_time: SystemTime, ctx: &dyn Context) {
        trace!(span.id=%id, "on_start");
        let mut span_ref = ctx.span(id).expect("Subscriber is missing provided span");
        if let Some(span) = span_ref.extensions_mut().remove::<CreatedSpan>() {
            let span = span.start_with_timestamp(start_time);
            self.exporter.export(TelemetryEvent::SpanStart(span));
        }
    }

    fn on_end(&self, id: &Id, end_time: SystemTime, _ctx: &dyn Context) {
        trace!(span.id=%id, "on_end");
        self.exporter.export(TelemetryEvent::SpanClose {
            id: id.clone(),
            end_time,
        });
    }

    fn on_record(&self, id: &Id, record: &dyn SpanRecordData, _ctx: &dyn Context) {
        let record = SpanRecord::from_data(id.clone(), record);
        self.exporter.export(TelemetryEvent::SpanRecord(record));
    }

    fn on_new_event(&self, event: &dyn NewEventData, ctx: &dyn Context) {
        let span = ctx.event_span(event).map(|span| span.id());
        let event = Event::from_new_data(span, event);
        self.exporter.export(TelemetryEvent::Event(event));
    }

    fn on_downstream_event(&self, event: &dyn korio::event::EventData, _ctx: &dyn Context) {
        let event = Event::from_data(event);
        self.exporter.export(TelemetryEvent::Event(event));
    }

    fn on_close(&self, id: &Id, _ctx: &dyn Context) {
        trace!(span.id=%id, "on_close");
    }

    // SAFETY: this is safe because `exporter` is valid for the lifetime of `&self`.
    unsafe fn downcast_raw(&self, id: TypeId) -> Option<*const ()> {
        if id == TypeId::of::<Self>() {
            Some(self as *const _ as *const ())
        } else if id == TypeId::of::<E>() {
            Some((&self.exporter) as *const _ as *const ())
        } else {
            unsafe { self.exporter.downcast_raw(id) }
        }
    }
}
