use std::io::Read;
use std::marker::PhantomData;

use capnp::serialize::{read_message_no_alloc, try_read_message_no_alloc};
use capnp::traits::Owned;
use capnp_convert::{ReadFrom, ReadInto};
use capnp_futures::ReadStream;
use futures_util::{Stream, TryStreamExt as _};
use korio_format::{TimedEvent, TimedEvents, TimedStreamHeader, WrapEvents};
use tokio::io::AsyncRead;

use crate::capnp::koriosc_capnp::{timed_event, timed_stream_header};

const BUFSIZE: usize = 1_000_000;

pub struct CapnpTimedStream<R, E, EO> {
    header: TimedStreamHeader,
    buffer: Vec<u8>,
    stream: R,
    phantom: PhantomData<(E, EO)>,
}

impl<R, E, EO> CapnpTimedStream<R, E, EO>
where
    R: Read,
{
    pub fn event_stream(mut read: R) -> capnp::Result<Self> {
        let mut scratch_space = vec![0_u8; BUFSIZE];
        let reader = read_message_no_alloc(&mut read, &mut scratch_space, Default::default())?;

        let reader: timed_stream_header::Reader = reader.get_root()?;
        let header = reader.read_into()?;
        Ok(Self::with_buffer(scratch_space, header, read))
    }
}

impl<R, E, EO> CapnpTimedStream<R, E, EO>
where
    R: Read,
{
    pub fn with_buffer(buffer: Vec<u8>, header: TimedStreamHeader, read: R) -> Self {
        Self {
            header,
            buffer,
            stream: read,
            phantom: PhantomData,
        }
    }
}

impl<R, E, EO> TimedEvents for CapnpTimedStream<R, E, EO> {
    fn get_header(&self) -> &TimedStreamHeader {
        &self.header
    }
}

impl<R, E, EO> Iterator for CapnpTimedStream<R, E, EO>
where
    R: Read,
    EO: Owned + 'static,
    E: for<'r> ReadFrom<EO::Reader<'r>>,
{
    type Item = capnp::Result<TimedEvent<E>>;

    fn next(&mut self) -> Option<Self::Item> {
        let reader =
            match try_read_message_no_alloc(&mut self.stream, &mut self.buffer, Default::default())
                .transpose()
            {
                Some(Ok(reader)) => reader,
                Some(Err(err)) => return Some(Err(err)),
                None => return None,
            };

        let reader: timed_event::Reader<'_, EO> = match reader.get_root() {
            Ok(reader) => reader,
            Err(err) => return Some(Err(err)),
        };
        Some(reader.read_into())
    }
}

pub async fn read_timed_stream_header<R>(reader: R) -> capnp::Result<TimedStreamHeader>
where
    R: AsyncRead + Unpin,
{
    let reader = tokio_util::compat::TokioAsyncReadCompatExt::compat(reader);
    let r = capnp_futures::serialize::read_message(reader, Default::default()).await?;
    r.get_root::<timed_stream_header::Reader>()?.read_into()
}

pub async fn timed_stream<'reader, EO, R, E>(
    mut reader: R,
) -> capnp::Result<impl Stream<Item = capnp::Result<TimedEvent<E>>> + TimedEvents + 'reader>
where
    R: AsyncRead + Unpin + 'reader,
    EO: Owned + 'static,
    E: for<'r> ReadFrom<EO::Reader<'r>>,
{
    let header = read_timed_stream_header(&mut reader).await?;
    let reader = tokio_util::compat::TokioAsyncReadCompatExt::compat(reader);
    let stream = ReadStream::new(reader, Default::default()).and_then(|reader| async move {
        reader.get_root::<timed_event::Reader<EO>>()?.read_into()
    });
    Ok(WrapEvents::with_header(stream, header))
}
