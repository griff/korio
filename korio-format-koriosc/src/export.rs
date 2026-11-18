use std::pin::pin;

use capnp::message;
use capnp_convert::SetFrom as _;
use capnp_futures::serialize::write_message;
use futures_util::{AsyncWriteExt, Stream, TryStreamExt as _};
use korio_capnp::capnp::capture_capnp;
use korio_data::telemetry_event::TelemetryEvent;
use korio_format::{TimedEvent, TimedEvents, TimedStreamHeader, TimingEvents};
use tokio::io::AsyncWrite;
use tracing::error;

use crate::capnp::koriosc_capnp::{timed_event, timed_stream_header};

const BUFSIZE: usize = 1_000_000;

pub async fn run_timed_event_stream<W, S, E>(stream: S, writer: W) -> capnp::Result<()>
where
    W: AsyncWrite + Unpin,
    S: TimedEvents + Stream<Item = Result<TimedEvent<TelemetryEvent>, E>>,
    capnp::Error: From<E>,
{
    let mut writer = tokio_util::compat::TokioAsyncWriteCompatExt::compat_write(writer);
    let mut scratch_space = vec![0_u8; BUFSIZE];
    let mut allocator = capnp::message::ScratchSpaceHeapAllocator::new(&mut scratch_space);
    {
        let mut builder = message::Builder::new(&mut allocator);
        let mut b: timed_stream_header::Builder = builder.init_root();
        b.set_from(stream.get_header())?;
        write_message(&mut writer, builder).await?;
        writer.flush().await?;
    }
    let mut stream = pin!(stream);
    while let Some(event) = stream.as_mut().try_next().await? {
        let mut builder = message::Builder::new(&mut allocator);
        let mut b: timed_event::Builder<'_, capture_capnp::telemetry_event::Owned> =
            builder.init_root();
        b.set_from(&event)?;
        write_message(&mut writer, builder).await?;
    }
    Ok(())
}

pub async fn try_run_stream_header<W, S, E>(
    header: TimedStreamHeader,
    receiver: S,
    writer: W,
) -> capnp::Result<()>
where
    W: AsyncWrite + Unpin,
    S: Stream<Item = Result<TelemetryEvent, E>>,
    capnp::Error: From<E>,
{
    let stream = TimingEvents::with_header(receiver, header);
    run_timed_event_stream(stream, writer).await
}

pub async fn try_run_stream<W, S, E>(receiver: S, writer: W) -> capnp::Result<()>
where
    W: AsyncWrite + Unpin,
    S: Stream<Item = Result<TelemetryEvent, E>>,
    capnp::Error: From<E>,
{
    try_run_stream_header(Default::default(), receiver, writer).await
}

pub async fn run_stream<W, S, E>(receiver: S, writer: W)
where
    W: AsyncWrite + Unpin,
    S: Stream<Item = Result<TelemetryEvent, E>>,
    capnp::Error: From<E>,
{
    if let Err(error) = try_run_stream(receiver, writer).await {
        error!(?error, "Error trying to export telemetry events");
    }
}
