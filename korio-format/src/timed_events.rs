use std::pin::Pin;
use std::task::{Poll, ready};
use std::time::Instant;
use std::{marker::PhantomData, task::Context};

use futures_core::Stream;
use futures_sink::Sink;
use pin_project_lite::pin_project;

use crate::{TimedEvent, TimedStreamHeader};

/// Trait for a 'Sink' or 'Stream' that has a 'TimedStreamHeader'.
pub trait TimedEvents {
    /// Acquires a reference to the `TimedStreamHeader` backing this `TimedEvents`.
    fn get_header(&self) -> &TimedStreamHeader;
}

pin_project! {
    /// Wrapper for `Sink` or `Stream` that has a 'TimedStreamHeader' and implements 'TimedEvents'.
    pub struct WrapEvents<S> {
        header: TimedStreamHeader,
        #[pin]
        inner: S,
    }
}

impl<S> WrapEvents<S> {
    /// Create a new `TimingEvents` with default header and that forwards to the
    /// given other sink.
    pub fn new(inner: S) -> Self {
        Self {
            header: Default::default(),
            inner,
        }
    }

    /// Create a new `TimedEvents` that forwards to the given other sink
    /// and whose timestamps are based on the start time specified in the
    /// provided `TimedStreamHeader`.
    pub fn with_header(inner: S, header: TimedStreamHeader) -> Self {
        Self { header, inner }
    }

    /// Acquires a reference to the underlying sink or stream that this `TimedEvents` is pulling from.
    pub fn get_ref(&self) -> &S {
        &self.inner
    }

    /// Consumes this `TimedEvents`, returning the underlying header and sink or stream.
    pub fn into_inner(self) -> (TimedStreamHeader, S) {
        (self.header, self.inner)
    }
}

impl<S> TimedEvents for WrapEvents<S> {
    /// Acquires a reference to the `TimedStreamHeader` backing this `TimedEvents`.
    fn get_header(&self) -> &TimedStreamHeader {
        &self.header
    }
}

impl<S, I> Sink<I> for WrapEvents<S>
where
    S: Sink<I>,
{
    type Error = S::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), Self::Error> {
        self.project().inner.start_send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx)
    }
}

impl<S> Stream for WrapEvents<S>
where
    S: Stream,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

pin_project! {
    /// `Sink` or `Stream` for timing events and sending them on.
    pub struct TimingEvents<S, I> {
        header: Header,
        #[pin]
        inner: S,
        _phantom: PhantomData<I>,
    }
}

impl<S, I> TimingEvents<S, I> {
    /// Create a new `TimingEvents` with default header and that forwards to the
    /// given other sink.
    pub fn new(inner: S) -> Self {
        Self {
            header: Header::default(),
            inner,
            _phantom: PhantomData,
        }
    }

    /// Create a new `TimingEvents` that forwards to the given other sink
    /// and whose timestamps are based on the start time specified in the
    /// provided `TimedStreamHeader`.
    pub fn with_header(inner: S, header: TimedStreamHeader) -> Self {
        Self {
            header: Header::new(header),
            inner,
            _phantom: PhantomData,
        }
    }

    /// Acquires a reference to the underlying sink or stream that this `TimedEvents` is pulling from.
    pub fn get_ref(&self) -> &S {
        &self.inner
    }

    /// Consumes this `TimedEvents`, returning the underlying header and sink or stream.
    pub fn into_inner(self) -> (TimedStreamHeader, S) {
        (self.header.header, self.inner)
    }
}

impl<S, I> TimedEvents for TimingEvents<S, I> {
    /// Acquires a reference to the `TimedStreamHeader` backing this `TimedEvents`.
    fn get_header(&self) -> &TimedStreamHeader {
        &self.header.header
    }
}

impl<S, I> Sink<I> for TimingEvents<S, I>
where
    S: Sink<TimedEvent<I>>,
{
    type Error = S::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), Self::Error> {
        let me = self.project();
        let item = me.header.event(item);
        me.inner.start_send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx)
    }
}

impl<S, I, E> Stream for TimingEvents<S, I>
where
    S: Stream<Item = Result<I, E>>,
{
    type Item = Result<TimedEvent<I>, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let me = self.project();
        match ready!(me.inner.poll_next(cx)) {
            Some(Ok(item)) => Poll::Ready(Some(Ok(me.header.event(item)))),
            Some(Err(err)) => Poll::Ready(Some(Err(err))),
            None => Poll::Ready(None),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[derive(Debug)]
struct Header {
    pub header: TimedStreamHeader,
    start_instant: Instant,
}

impl Header {
    pub fn new(header: TimedStreamHeader) -> Self {
        let start_instant = Instant::now();
        Self {
            header,
            start_instant,
        }
    }

    pub fn event<E>(&self, event: E) -> TimedEvent<E> {
        TimedEvent {
            timestamp: self.header.time_range.start + self.start_instant.elapsed(),
            event,
        }
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            header: Default::default(),
            start_instant: Instant::now(),
        }
    }
}
