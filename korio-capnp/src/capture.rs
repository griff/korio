use capnp::capability::{Client, FromClientHook, Promise, Rc};
use capnp_convert::{ReadInto, SetFrom as _};
use capnp_rpc::pry;
use futures_channel::mpsc;
use futures_util::StreamExt as _;
use korio::span::{Id, Span, SpanId};
use korio_capture::{Capture, SpanCaptureExt as _};
use korio_capture_data::TelemetryEventCapture;
use korio_data::telemetry_event::TelemetryEvent;
use tokio::task::spawn_local;
use tracing::error;

use crate::capnp::capture_capnp::{capture, telemetry_stream};
use crate::instrument::WithDispatch;

pub struct CaptureServer {
    capability: Client,
}

impl capture::Server for CaptureServer {
    fn capture(
        self: Rc<Self>,
        params: capture::CaptureParams,
        mut result: capture::CaptureResults,
    ) -> impl Future<Output = capnp::Result<()>> {
        let r = pry!(pry!(params.get()).get_capture());
        let receiver = pry!(r.get_receiver());
        let config = pry!(r.get_config());
        let id = Id {
            trace_id: pry!(pry!(config.get_trace_id()).read_into()),
            span_id: SpanId::from_non_zero_u64(pry!(
                config
                    .get_span_id()
                    .try_into()
                    .map_err(|_| capnp::Error::failed("invalid span id".into()))
            )),
        };

        let span = Span::from_upstream(&id);
        let capture = export_events(receiver);
        span.capture(&capture);

        result
            .get()
            .get_wrapped()
            .set_as_capability(Box::new(WithDispatch::wrap_current(
                self.capability.clone().into_client_hook(),
            )));
        Promise::ok(())
    }
}

async fn try_run_capture(
    mut receiver: mpsc::UnboundedReceiver<TelemetryEvent>,
    client: telemetry_stream::Client,
) -> capnp::Result<()> {
    while let Some(event) = receiver.next().await {
        let mut req = client.events_request();
        let b = req.get();
        b.init_events(1).get(0).set_from(&event)?;
        req.send().await?;
    }
    client.completed_request().send().promise.await?;
    Ok(())
}

async fn run_capture(
    receiver: mpsc::UnboundedReceiver<TelemetryEvent>,
    client: telemetry_stream::Client,
) {
    if let Err(error) = try_run_capture(receiver, client).await {
        error!(?error, "Error trying to export telemetry events");
    }
}

pub fn export_events(client: telemetry_stream::Client) -> Capture {
    let (sender, receiver) = mpsc::unbounded();
    spawn_local(run_capture(receiver, client));
    let capture = TelemetryEventCapture::with_exporter(sender);
    Capture::new(capture)
}
