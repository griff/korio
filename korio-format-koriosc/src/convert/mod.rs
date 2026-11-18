use std::{
    ops::Range,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use capnp_convert::{ReadFrom, ReadInto, SetFrom, SetInto};
use korio_format::{OpenTimeRange, TimedEvent, TimedRange, TimedStreamHeader, time::UnixTimeNano};

use crate::capnp::koriosc_capnp::{
    timed_event, timed_range, timed_stream_header, unix_time_nano_range,
};

impl SetInto<timed_stream_header::Builder<'_>> for TimedStreamHeader {
    fn set_into(&self, builder: &mut timed_stream_header::Builder<'_>) -> capnp::Result<()> {
        builder
            .reborrow()
            .init_time_range()
            .set_from(&self.time_range)?;
        Ok(())
    }
}

impl ReadFrom<timed_stream_header::Reader<'_>> for TimedStreamHeader {
    fn read_from(reader: timed_stream_header::Reader<'_>) -> Result<Self, capnp::Error> {
        let time_range = reader.get_time_range()?.read_into()?;
        Ok(TimedStreamHeader { time_range })
    }
}

impl<E, Event> SetInto<timed_event::Builder<'_, Event>> for TimedEvent<E>
where
    Event: capnp::traits::Owned,
    for<'b> E: SetInto<Event::Builder<'b>>,
{
    fn set_into(&self, builder: &mut timed_event::Builder<'_, Event>) -> capnp::Result<()> {
        let ts: UnixTimeNano = self.timestamp.into();
        builder.set_timestamp(ts.as_nanos_since_epoch());
        builder.reborrow().init_event().set_from(&self.event)?;
        Ok(())
    }
}

impl<Event, E> ReadFrom<timed_event::Reader<'_, Event>> for TimedEvent<E>
where
    Event: capnp::traits::Owned,
    E: for<'r> ReadFrom<Event::Reader<'r>>,
{
    fn read_from(reader: timed_event::Reader<'_, Event>) -> Result<Self, capnp::Error> {
        let timestamp_dur = Duration::from_nanos(reader.get_timestamp());
        let event = reader.get_event()?.read_into()?;
        Ok(TimedEvent {
            timestamp: UNIX_EPOCH + timestamp_dur,
            event,
        })
    }
}

impl<E, Event> SetInto<timed_range::Builder<'_, Event>> for TimedRange<E>
where
    Event: capnp::traits::Owned,
    for<'b> E: SetInto<Event::Builder<'b>>,
{
    fn set_into(&self, builder: &mut timed_range::Builder<'_, Event>) -> capnp::Result<()> {
        builder.reborrow().init_range().set_from(&self.range)?;
        builder.reborrow().init_event().set_from(&self.event)?;
        Ok(())
    }
}

impl<Event, E> ReadFrom<timed_range::Reader<'_, Event>> for TimedRange<E>
where
    Event: capnp::traits::Owned,
    E: for<'r> ReadFrom<Event::Reader<'r>>,
{
    fn read_from(reader: timed_range::Reader<'_, Event>) -> Result<Self, capnp::Error> {
        let range = reader.get_range()?.read_into()?;
        let event = reader.get_event()?.read_into()?;
        Ok(TimedRange { range, event })
    }
}

impl SetInto<unix_time_nano_range::Builder<'_>> for Range<UnixTimeNano> {
    fn set_into(&self, builder: &mut unix_time_nano_range::Builder<'_>) -> capnp::Result<()> {
        builder.set_start(self.start.as_nanos_since_epoch());
        builder.set_end(self.end.as_nanos_since_epoch());
        Ok(())
    }
}

impl ReadFrom<unix_time_nano_range::Reader<'_>> for Range<UnixTimeNano> {
    fn read_from(reader: unix_time_nano_range::Reader<'_>) -> Result<Self, capnp::Error> {
        let start = UnixTimeNano::from_nanos_since_epoch(reader.get_start());
        let end = UnixTimeNano::from_nanos_since_epoch(reader.get_end());
        Ok(start..end)
    }
}

impl SetInto<unix_time_nano_range::Builder<'_>> for Range<SystemTime> {
    fn set_into(&self, builder: &mut unix_time_nano_range::Builder<'_>) -> capnp::Result<()> {
        let start: UnixTimeNano = self.start.into();
        builder.set_start(start.as_nanos_since_epoch());
        let end: UnixTimeNano = self.end.into();
        builder.set_end(end.as_nanos_since_epoch());
        Ok(())
    }
}

impl ReadFrom<unix_time_nano_range::Reader<'_>> for Range<SystemTime> {
    fn read_from(reader: unix_time_nano_range::Reader<'_>) -> Result<Self, capnp::Error> {
        let range: Range<UnixTimeNano> = reader.read_into()?;
        Ok(range.start.into()..range.end.into())
    }
}

impl SetInto<unix_time_nano_range::Builder<'_>> for OpenTimeRange {
    fn set_into(&self, builder: &mut unix_time_nano_range::Builder<'_>) -> capnp::Result<()> {
        let start: UnixTimeNano = self.start.into();
        builder.set_start(start.as_nanos_since_epoch());
        if let Some(end) = self.end {
            let end: UnixTimeNano = end.into();
            builder.set_end(end.as_nanos_since_epoch());
        }
        Ok(())
    }
}

impl ReadFrom<unix_time_nano_range::Reader<'_>> for OpenTimeRange {
    fn read_from(reader: unix_time_nano_range::Reader<'_>) -> Result<Self, capnp::Error> {
        let start = UnixTimeNano::from_nanos_since_epoch(reader.get_start());
        if reader.get_end() > 0 {
            let end = UnixTimeNano::from_nanos_since_epoch(reader.get_end());
            Ok(OpenTimeRange {
                start: start.into(),
                end: Some(end.into()),
            })
        } else {
            Ok(OpenTimeRange {
                start: start.into(),
                end: None,
            })
        }
    }
}
