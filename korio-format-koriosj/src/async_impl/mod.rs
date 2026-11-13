use futures_channel::mpsc;
use futures_util::StreamExt as _;
use korio_capture_data::TelemetryEventCapture;
use korio_data::telemetry_event::TelemetryEvent;
use korio_format::{TimedEvents, TimingEvents};
use tokio::{io::AsyncWrite, spawn};

mod codec;
mod json_timed_event_sink;

pub use codec::{JsonLinesCodec, JsonLinesCodecError};
pub use json_timed_event_sink::JsonTimedEventSink;

/// Return [`TelemetryEventCapture`] for sending [`TelemetryEvent`] in `koriosj` format to provided `writer`.
pub fn setup_output<W>(writer: W) -> TelemetryEventCapture<mpsc::UnboundedSender<TelemetryEvent>>
where
    W: AsyncWrite + Unpin + Send + 'static,
{
    let (sender, receiver) = mpsc::unbounded();
    spawn(async move {
        let events = TimingEvents::new(receiver.map(Ok));
        let sink = JsonTimedEventSink::with_header(events.get_header().clone(), writer).await?;
        events.forward(sink).await
    });
    TelemetryEventCapture::with_exporter(sender)
}
