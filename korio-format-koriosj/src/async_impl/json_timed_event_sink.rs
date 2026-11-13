use futures_util::sink::{Sink, SinkExt as _};
use korio_format::{TimedEvent, TimedStreamHeader};
use pin_project_lite::pin_project;
use serde::Serialize;
use tokio::io::AsyncWrite;
use tokio_util::codec::FramedWrite;

use crate::{JsonLinesCodec, JsonLinesCodecError};

pin_project! {
    /// A sink for the [JSON Timed Stream] format.
    /// 
    /// [JSON Timed Stream]: crate#format-spec
    pub struct JsonTimedEventSink<W, Item> {
        header: TimedStreamHeader,
        #[pin]
        framed: FramedWrite<W, JsonLinesCodec<TimedEvent<Item>>>,
    }
}

impl<W, Item> JsonTimedEventSink<W, Item>
where
    W: AsyncWrite + Unpin,
{
    /// Return a `JsonTimedEventSink` for the given `writer`.
    pub async fn new(writer: W) -> Result<Self, JsonLinesCodecError> {
        Self::with_header(Default::default(), writer).await
    }

    /// Return a `JsonTimedEventSink` for the given `writer` that uses the given `TimedStreamHeader`.
    pub async fn with_header(header: TimedStreamHeader, writer: W) -> Result<Self, JsonLinesCodecError> {
        let mut framed = FramedWrite::new(writer, JsonLinesCodec::<TimedStreamHeader>::default());
        framed.feed(header.clone()).await?;
        let framed = framed.map_encoder(JsonLinesCodec::into_item_codec::<TimedEvent<Item>>);
        Ok(Self { header, framed })
    }
}

impl<W, Item> Sink<TimedEvent<Item>> for JsonTimedEventSink<W, Item>
where
    W: AsyncWrite,
    Item: Serialize,
{
    type Error = JsonLinesCodecError;

    fn poll_ready(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.project().framed.poll_ready(cx)
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: TimedEvent<Item>) -> Result<(), Self::Error> {
        self.project().framed.start_send(item)
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.project().framed.poll_flush(cx)
    }

    fn poll_close(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.project().framed.poll_close(cx)
    }
}
