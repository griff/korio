use std::{
    collections::BTreeMap,
    num::NonZeroU64,
    time::{Duration, UNIX_EPOCH},
};

use capnp_convert::{ReadFrom, ReadInto as _, SetFrom as _, SetInto};
use korio::{
    Severity,
    span::{Id, SpanId},
};
use korio_data::{
    event::Event,
    span::{EndedSpan, SpanFollowsFrom, SpanRecord, StartedSpan},
    telemetry_event::TelemetryEvent,
    time::UnixTimeNano,
};

use crate::capnp::{
    capture_capnp::{span_record, telemetry_event},
    event_capnp,
    trace_capnp::{span, span_follows_from},
};

impl SetInto<event_capnp::event::Builder<'_>> for Event {
    fn set_into(&self, builder: &mut event_capnp::event::Builder<'_>) -> capnp::Result<()> {
        if let Some(id) = self.span.as_ref() {
            builder.reborrow().init_trace_id().set_from(&id.trace_id)?;
            builder.set_span_id(id.span_id.into_u64());
        }

        let unix_time: UnixTimeNano = self.timestamp.into();
        builder.set_time_unix_nano(unix_time.as_nanos_since_epoch());

        builder.set_observed_time_unix_nano(unix_time.as_nanos_since_epoch());
        builder.set_name(&self.name);
        builder.set_target(&self.target);

        if let Some(body) = self.body() {
            builder.reborrow().init_body().set_from(body)?;
        }

        builder.set_severity_number(self.severity_number.into());
        builder.set_severity_text(&self.severity_text);

        let mut attrs = builder
            .reborrow()
            .init_attributes(self.attributes().len() as u32);
        for (index, kv) in self.attributes().enumerate() {
            attrs.reborrow().get(index as u32).set_from(&kv)?;
        }
        Ok(())
    }
}

impl ReadFrom<event_capnp::event::Reader<'_>> for Event {
    fn read_from(reader: event_capnp::event::Reader<'_>) -> Result<Self, capnp::Error> {
        let name = reader.get_name()?.to_string()?.into();
        let target = reader.get_target()?.to_string()?.into();
        let severity_number: Severity = reader.get_severity_number()?.into();
        let mut attributes = BTreeMap::new();
        for item in reader.get_attributes()? {
            attributes.insert(
                item.get_key()?.to_string()?.into(),
                item.get_value()?.read_into()?,
            );
        }
        let body = if reader.has_body() {
            Some(reader.get_body()?.read_into()?)
        } else {
            None
        };

        let severity_text = if reader.has_severity_text() {
            reader.get_severity_text()?.to_string()?.into()
        } else {
            severity_number.as_str().into()
        };
        let timestamp = UNIX_EPOCH + Duration::from_nanos(reader.get_time_unix_nano());
        let observed_timestamp =
            UNIX_EPOCH + Duration::from_nanos(reader.get_observed_time_unix_nano());
        let span = if let Some(span_id) = NonZeroU64::new(reader.get_span_id()) {
            let trace_id = reader.get_trace_id()?.read_into()?;
            Some(Id {
                trace_id,
                span_id: SpanId::from_non_zero_u64(span_id),
            })
        } else {
            None
        };
        Ok(Event {
            name,
            target,
            timestamp,
            observed_timestamp,
            span,
            severity_text,
            severity_number,
            body,
            attributes,
        })
    }
}

impl SetInto<span_record::Builder<'_>> for SpanRecord {
    fn set_into(&self, builder: &mut span_record::Builder<'_>) -> capnp::Result<()> {
        builder
            .reborrow()
            .init_trace_id()
            .set_from(&self.id.trace_id)?;
        builder.set_span_id(self.id.span_id.into_u64());
        builder.set_observed_time_unix_nano(
            UnixTimeNano::from(self.observed_timestamp).as_nanos_since_epoch(),
        );

        let mut attrs = builder
            .reborrow()
            .init_attributes(self.attributes.len() as u32);
        for (index, kv) in self.attributes.iter().enumerate() {
            attrs.reborrow().get(index as u32).set_from(&kv)?;
        }
        Ok(())
    }
}

impl ReadFrom<span_record::Reader<'_>> for SpanRecord {
    fn read_from(reader: span_record::Reader<'_>) -> Result<Self, capnp::Error> {
        let id = Id {
            trace_id: reader.get_trace_id()?.read_into()?,
            span_id: SpanId::from_non_zero_u64(
                reader
                    .get_span_id()
                    .try_into()
                    .map_err(|_| capnp::Error::failed("invalid span id".into()))?,
            ),
        };
        let observed_timestamp =
            UNIX_EPOCH + Duration::from_nanos(reader.get_observed_time_unix_nano());
        let mut attributes = BTreeMap::new();
        for item in reader.get_attributes()? {
            attributes.insert(
                item.get_key()?.to_string()?.into(),
                item.get_value()?.read_into()?,
            );
        }

        Ok(SpanRecord {
            id,
            observed_timestamp,
            attributes,
        })
    }
}

impl SetInto<span_follows_from::Builder<'_>> for SpanFollowsFrom {
    fn set_into(&self, builder: &mut span_follows_from::Builder<'_>) -> capnp::Result<()> {
        let mut id = builder.reborrow().init_id();
        id.reborrow().init_trace_id().set_from(&self.id.trace_id)?;
        id.set_span_id(self.id.span_id.into_u64());
        let mut follows = builder.reborrow().init_follows();
        follows
            .reborrow()
            .init_trace_id()
            .set_from(&self.from.trace_id)?;
        follows.set_span_id(self.from.span_id.into_u64());
        builder.set_observed_time_unix_nano(
            UnixTimeNano::from(self.observed_timestamp).as_nanos_since_epoch(),
        );

        let mut attrs = builder
            .reborrow()
            .init_attributes(self.attributes.len() as u32);
        for (index, kv) in self.attributes.iter().enumerate() {
            attrs.reborrow().get(index as u32).set_from(&kv)?;
        }
        Ok(())
    }
}

impl ReadFrom<span_follows_from::Reader<'_>> for SpanFollowsFrom {
    fn read_from(reader: span_follows_from::Reader<'_>) -> Result<Self, capnp::Error> {
        let id = Id {
            trace_id: reader.get_id().get_trace_id()?.read_into()?,
            span_id: SpanId::from_non_zero_u64(
                reader
                    .get_id()
                    .get_span_id()
                    .try_into()
                    .map_err(|_| capnp::Error::failed("invalid span id".into()))?,
            ),
        };
        let from = Id {
            trace_id: reader.get_follows().get_trace_id()?.read_into()?,
            span_id: SpanId::from_non_zero_u64(
                reader
                    .get_follows()
                    .get_span_id()
                    .try_into()
                    .map_err(|_| capnp::Error::failed("invalid span id".into()))?,
            ),
        };
        let observed_timestamp =
            UNIX_EPOCH + Duration::from_nanos(reader.get_observed_time_unix_nano());
        let mut attributes = BTreeMap::new();
        for item in reader.get_attributes()? {
            attributes.insert(
                item.get_key()?.to_string()?.into(),
                item.get_value()?.read_into()?,
            );
        }

        Ok(SpanFollowsFrom {
            id,
            from,
            observed_timestamp,
            attributes,
        })
    }
}

impl SetInto<span::Builder<'_>> for StartedSpan {
    fn set_into(&self, builder: &mut span::Builder<'_>) -> capnp::Result<()> {
        builder
            .reborrow()
            .init_trace_id()
            .set_from(&self.id.trace_id)?;
        builder.set_span_id(self.id.span_id.into_u64());

        if let Some(parent_id) = self.parent_id.as_ref() {
            builder.set_parent_span_id(parent_id.span_id.into_u64());
        }

        builder.set_name(&self.name);
        builder.set_target(&self.target);
        builder.set_severity_number(self.severity_number.into());
        builder.set_severity_text(&self.severity_text);

        let start_time: UnixTimeNano = self.start_time.into();
        builder.set_start_time_unix_nano(start_time.as_nanos_since_epoch());

        let mut attrs = builder
            .reborrow()
            .init_attributes(self.attributes.len() as u32);
        for (index, kv) in self.attributes.iter().enumerate() {
            attrs.reborrow().get(index as u32).set_from(&kv)?;
        }
        Ok(())
    }
}

impl ReadFrom<span::Reader<'_>> for StartedSpan {
    fn read_from(reader: span::Reader<'_>) -> Result<Self, capnp::Error> {
        let id = Id {
            trace_id: reader.get_trace_id()?.read_into()?,
            span_id: SpanId::from_non_zero_u64(
                reader
                    .get_span_id()
                    .try_into()
                    .map_err(|_| capnp::Error::failed("invalid span id".into()))?,
            ),
        };
        let name = reader.get_name()?.to_string()?.into();
        let target = reader.get_target()?.to_string()?.into();
        let severity_number: Severity = reader.get_severity_number()?.into();
        let mut attributes = BTreeMap::new();
        for item in reader.get_attributes()? {
            attributes.insert(
                item.get_key()?.to_string()?.into(),
                item.get_value()?.read_into()?,
            );
        }

        let severity_text = if reader.has_severity_text() {
            reader.get_severity_text()?.to_string()?.into()
        } else {
            severity_number.as_str().into()
        };
        let start_time = UNIX_EPOCH + Duration::from_nanos(reader.get_start_time_unix_nano());
        let parent_id = if let Some(span_id) = NonZeroU64::new(reader.get_parent_span_id()) {
            Some(Id {
                trace_id: id.trace_id.clone(),
                span_id: SpanId::from_non_zero_u64(span_id),
            })
        } else {
            None
        };
        Ok(StartedSpan {
            parent_id,
            id,
            name,
            target,
            start_time,
            severity_text,
            severity_number,
            attributes,
        })
    }
}

impl SetInto<span::Builder<'_>> for EndedSpan {
    fn set_into(&self, builder: &mut span::Builder<'_>) -> capnp::Result<()> {
        builder
            .reborrow()
            .init_trace_id()
            .set_from(&self.id.trace_id)?;
        builder.set_span_id(self.id.span_id.into_u64());

        if let Some(parent_id) = self.parent_id.as_ref() {
            builder.set_parent_span_id(parent_id.span_id.into_u64());
        }

        builder.set_name(&self.name);
        builder.set_target(&self.target);
        builder.set_severity_number(self.severity_number.into());
        builder.set_severity_text(&self.severity_text);

        let start_time: UnixTimeNano = self.start_time.into();
        builder.set_start_time_unix_nano(start_time.as_nanos_since_epoch());
        let end_time: UnixTimeNano = self.end_time.into();
        builder.set_end_time_unix_nano(end_time.as_nanos_since_epoch());

        let mut attrs = builder
            .reborrow()
            .init_attributes(self.attributes.len() as u32);
        for (index, kv) in self.attributes.iter().enumerate() {
            attrs.reborrow().get(index as u32).set_from(&kv)?;
        }
        Ok(())
    }
}

impl ReadFrom<span::Reader<'_>> for EndedSpan {
    fn read_from(reader: span::Reader<'_>) -> Result<Self, capnp::Error> {
        let id = Id {
            trace_id: reader.get_trace_id()?.read_into()?,
            span_id: SpanId::from_non_zero_u64(
                reader
                    .get_span_id()
                    .try_into()
                    .map_err(|_| capnp::Error::failed("invalid span id".into()))?,
            ),
        };
        let name = reader.get_name()?.to_string()?.into();
        let target = reader.get_target()?.to_string()?.into();
        let severity_number: Severity = reader.get_severity_number()?.into();
        let mut attributes = BTreeMap::new();
        for item in reader.get_attributes()? {
            attributes.insert(
                item.get_key()?.to_string()?.into(),
                item.get_value()?.read_into()?,
            );
        }

        let severity_text = if reader.has_severity_text() {
            reader.get_severity_text()?.to_string()?.into()
        } else {
            severity_number.as_str().into()
        };
        let start_time = UNIX_EPOCH + Duration::from_nanos(reader.get_start_time_unix_nano());
        let end_time = UNIX_EPOCH + Duration::from_nanos(reader.get_end_time_unix_nano());
        let parent_id = if let Some(span_id) = NonZeroU64::new(reader.get_parent_span_id()) {
            Some(Id {
                trace_id: id.trace_id.clone(),
                span_id: SpanId::from_non_zero_u64(span_id),
            })
        } else {
            None
        };
        Ok(EndedSpan {
            parent_id,
            id,
            name,
            target,
            start_time,
            end_time,
            severity_text,
            severity_number,
            attributes,
        })
    }
}

impl SetInto<telemetry_event::Builder<'_>> for TelemetryEvent {
    fn set_into(&self, builder: &mut telemetry_event::Builder<'_>) -> capnp::Result<()> {
        match self {
            TelemetryEvent::SpanStart(span) => {
                builder.reborrow().init_span_start().set_from(span)?;
            }
            TelemetryEvent::SpanRecord(record) => {
                builder.reborrow().init_span_record().set_from(record)?;
            }
            TelemetryEvent::SpanRecordFollowsFrom(record) => {
                builder
                    .reborrow()
                    .init_span_record_follows_from()
                    .set_from(record)?;
            }
            TelemetryEvent::SpanClose { id, end_time } => {
                let mut b = builder.reborrow().init_span_close();
                b.reborrow().init_trace_id().set_from(&id.trace_id)?;
                b.set_span_id(id.span_id.into_u64());
                b.set_end_time_unix_nano(UnixTimeNano::from(*end_time).as_nanos_since_epoch());
            }
            TelemetryEvent::Event(event) => {
                builder.reborrow().init_event().set_from(event)?;
            }
        }
        Ok(())
    }
}

impl ReadFrom<telemetry_event::Reader<'_>> for TelemetryEvent {
    fn read_from(reader: telemetry_event::Reader<'_>) -> Result<Self, capnp::Error> {
        match reader.which()? {
            telemetry_event::Which::SpanStart(create) => {
                Ok(TelemetryEvent::SpanStart(create?.read_into()?))
            }
            telemetry_event::Which::SpanRecord(reader) => {
                Ok(TelemetryEvent::SpanRecord(reader?.read_into()?))
            }
            telemetry_event::Which::SpanRecordFollowsFrom(reader) => {
                Ok(TelemetryEvent::SpanRecordFollowsFrom(reader?.read_into()?))
            }
            telemetry_event::Which::SpanClose(reader) => {
                let id = Id {
                    trace_id: reader.get_trace_id()?.read_into()?,
                    span_id: SpanId::from_non_zero_u64(
                        reader
                            .get_span_id()
                            .try_into()
                            .map_err(|_| capnp::Error::failed("invalid span id".into()))?,
                    ),
                };
                let end_time = UNIX_EPOCH + Duration::from_nanos(reader.get_end_time_unix_nano());
                Ok(TelemetryEvent::SpanClose { id, end_time })
            }
            telemetry_event::Which::Event(event) => Ok(TelemetryEvent::Event(event?.read_into()?)),
        }
    }
}
