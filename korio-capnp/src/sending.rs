use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use capnp_convert::ReadInto as _;
use korio::event::EventDataEmit;
use korio::span::SpanDataCreate;
use korio_data::telemetry_event::TelemetryEvent;

use crate::capnp::capture_capnp::telemetry_stream;

#[derive(Debug, Default)]
pub struct ImportStream {
    spans: RefCell<HashMap<korio::span::Id, korio::Span>>,
}

impl ImportStream {
    fn send_events(&self, params: telemetry_stream::EventsParams) -> capnp::Result<()> {
        for event in params.get()?.get_events()? {
            let event: TelemetryEvent = event.read_into()?;
            match event {
                TelemetryEvent::SpanStart(span_data) => {
                    let span = span_data.create_downstream();
                    span.start_with_timestamp(span_data.start_time);
                    if let Some(id) = span.id() {
                        self.spans.borrow_mut().insert(id, span);
                    }
                }
                TelemetryEvent::SpanRecord(record) => {
                    if let Some(span) = self.spans.borrow().get(&record.id) {
                        span.record_data(&record);
                    }
                }
                TelemetryEvent::SpanRecordFollowsFrom(link) => {
                    if let Some(span) = self.spans.borrow().get(&link.id) {
                        span.follows_from_data(&link);
                    }
                }
                TelemetryEvent::SpanClose { id, end_time } => {
                    if let Some(span) = self.spans.borrow().get(&id) {
                        span.end_with_timestamp(end_time);
                    }
                }
                TelemetryEvent::Event(event) => {
                    event.emit_downstream();
                }
            }
        }
        Ok(())
    }

    fn close_events(&self) -> capnp::Result<()> {
        for (_, span) in self.spans.take().into_iter() {
            span.end();
        }
        Ok(())
    }
}

impl telemetry_stream::Server for ImportStream {
    async fn events(self: Rc<Self>, params: telemetry_stream::EventsParams) -> capnp::Result<()> {
        self.send_events(params)
    }

    async fn completed(
        self: Rc<Self>,
        _params: telemetry_stream::CompletedParams,
        _result: telemetry_stream::CompletedResults,
    ) -> capnp::Result<()> {
        self.close_events()
    }
}
