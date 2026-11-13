use std::time::SystemTime;

use korio_core::span::Id;

use crate::event::Event;
use crate::span::{SpanFollowsFrom, SpanRecord, StartedSpan};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum TelemetryEvent {
    SpanStart(StartedSpan),
    SpanRecord(SpanRecord),
    SpanRecordFollowsFrom(SpanFollowsFrom),
    Event(Event),
    SpanClose {
        id: Id,
        #[cfg_attr(feature = "serde", serde(with = "super::time::system_time_serde"))]
        end_time: SystemTime,
    },
}
