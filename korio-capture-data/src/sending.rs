use std::collections::HashMap;
use std::convert::Infallible;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::SystemTime;

use futures_sink::Sink;
use korio::event::EventDataEmit;
use korio::span::{Id, Span};
use korio_data::telemetry_event::TelemetryEvent;

#[derive(Debug, Default)]
pub struct KorioDataSink {
    open_spans: HashMap<Id, Span>,
    end_on_drop: bool,
}

impl KorioDataSink {
    pub fn send(&mut self, event: TelemetryEvent) {
        match event {
            TelemetryEvent::SpanStart(span) => {
                let span = Span::from_downstream(&span);
                if let Some(id) = span.id() {
                    self.open_spans.insert(id, span);
                }
            }
            TelemetryEvent::SpanRecord(record) => {
                if let Some(span) = self.open_spans.get(&record.id) {
                    span.record_data(&record);
                }
            }
            TelemetryEvent::SpanRecordFollowsFrom(record) => {
                if let Some(span) = self.open_spans.get(&record.id) {
                    span.follows_from_data(&record);
                }
            }
            TelemetryEvent::Event(event) => {
                event.emit_downstream();
            }
            TelemetryEvent::SpanClose { id, end_time } => {
                if let Some(span) = self.open_spans.remove(&id) {
                    span.end_with_timestamp(end_time);
                }
            }
        }
    }

    pub fn end_all(&mut self) {
        self.end_all_with_timestamp(SystemTime::now());
    }

    pub fn end_all_with_timestamp(&mut self, end_time: SystemTime) {
        for (_, span) in self.open_spans.drain() {
            span.end_with_timestamp(end_time);
        }
    }
}

impl Drop for KorioDataSink {
    fn drop(&mut self) {
        if self.end_on_drop {
            self.end_all();
        }
    }
}

impl Sink<TelemetryEvent> for KorioDataSink {
    type Error = Infallible;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, event: TelemetryEvent) -> Result<(), Self::Error> {
        self.get_mut().send(event);
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.get_mut();
        if this.end_on_drop {
            this.end_all();
        }
        Poll::Ready(Ok(()))
    }
}
