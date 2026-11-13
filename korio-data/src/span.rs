use std::{collections::BTreeMap, time::SystemTime};

use korio_core::metadata::{AnyValue, AttributeSet, KeyBuf, Metadata, Severity};
use korio_core::span::{
    ActiveSpanData, EndedSpanData, Id, NewSpanData, SpanData, SpanFollowsFromData,
    SpanFollowsFromDownstreamData, SpanRecordData, SpanRecordDownstreamData, StartedSpanData,
};

use crate::telemetry_event::TelemetryEvent;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CreatedSpan {
    pub parent_id: Option<Id>,

    pub id: Id,

    pub target: String,

    pub name: String,

    pub severity_text: String,

    /// The corresponding severity value, normalized
    pub severity_number: Severity,

    /// Additional attributes associated with this record
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "BTreeMap::is_empty", default)
    )]
    pub attributes: BTreeMap<KeyBuf, AnyValue>,
}

impl CreatedSpan {
    pub fn from_new_data(id: Id, parent_id: Option<Id>, data: &dyn NewSpanData) -> Self {
        let target = data.target().to_string();
        let name = data.name().to_string();
        let severity_number = data.severity_number();
        let severity_text = data.severity_text().to_string();
        let mut attributes = BTreeMap::new();
        data.attributes().record(&mut attributes);
        Self {
            parent_id,
            id,
            target,
            name,
            severity_text,
            severity_number,
            attributes,
        }
    }

    pub fn from_data(data: &dyn SpanData) -> Self {
        let id = data.id().clone();
        let parent_id = data.parent();
        let target = data.target().to_string();
        let name = data.name().to_string();
        let severity_number = data.severity_number();
        let severity_text = data.severity_text().to_string();
        let mut attributes = BTreeMap::new();
        data.attributes().record(&mut attributes);
        Self {
            parent_id,
            id,
            target,
            name,
            severity_text,
            severity_number,
            attributes,
        }
    }

    pub fn attributes(&self) -> impl ExactSizeIterator<Item = (&KeyBuf, &AnyValue)> {
        self.attributes.iter()
    }

    pub fn start_with_timestamp(self, start_time: SystemTime) -> StartedSpan {
        StartedSpan {
            parent_id: self.parent_id,
            id: self.id,
            name: self.name,
            target: self.target,
            start_time,
            severity_text: self.severity_text,
            severity_number: self.severity_number,
            attributes: self.attributes,
        }
    }

    pub fn start(self) -> StartedSpan {
        self.start_with_timestamp(SystemTime::now())
    }
}

impl Metadata for CreatedSpan {
    fn target(&self) -> &str {
        &self.target
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn severity_text(&self) -> &str {
        &self.severity_text
    }

    fn severity_number(&self) -> Severity {
        self.severity_number
    }
}

impl SpanData for CreatedSpan {
    fn id(&self) -> &Id {
        &self.id
    }

    fn parent(&self) -> Option<Id> {
        self.parent_id.clone()
    }

    fn attributes(&self) -> &dyn AttributeSet {
        &self.attributes
    }
}

impl ActiveSpanData for CreatedSpan {
    fn start_time(&self) -> Option<SystemTime> {
        None
    }

    fn end_time(&self) -> Option<SystemTime> {
        None
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct StartedSpan {
    pub parent_id: Option<Id>,
    pub id: Id,
    pub target: String,
    pub name: String,
    #[cfg_attr(feature = "serde", serde(with = "super::time::system_time_serde"))]
    pub start_time: SystemTime,

    /// The original severity string from the source
    pub severity_text: String,

    /// The corresponding severity value, normalized
    pub severity_number: Severity,

    /// Additional attributes associated with this record
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "BTreeMap::is_empty", default)
    )]
    pub attributes: BTreeMap<KeyBuf, AnyValue>,
}

impl StartedSpan {
    pub fn from_data(data: &dyn StartedSpanData) -> Self {
        let parent_id = data.parent().clone();
        let id = data.id().clone();
        let target = data.target().to_string();
        let name = data.name().to_string();
        let severity_number = data.severity_number();
        let severity_text = data.severity_text().to_string();
        let start_time = data.start_time();
        let mut attributes = BTreeMap::new();
        data.attributes().record(&mut attributes);
        Self {
            parent_id,
            id,
            target,
            name,
            severity_text,
            severity_number,
            start_time,
            attributes,
        }
    }

    pub fn record_attributes(&mut self, attributes: &dyn AttributeSet) {
        attributes.record(&mut self.attributes)
    }

    pub fn end_with_timestamp(self, end_time: SystemTime) -> EndedSpan {
        EndedSpan {
            parent_id: self.parent_id,
            id: self.id,
            name: self.name,
            target: self.target,
            start_time: self.start_time,
            end_time,
            severity_text: self.severity_text,
            severity_number: self.severity_number,
            attributes: self.attributes,
        }
    }

    pub fn end(self) -> EndedSpan {
        self.end_with_timestamp(SystemTime::now())
    }
}

impl Metadata for StartedSpan {
    fn target(&self) -> &str {
        &self.target
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn severity_text(&self) -> &str {
        &self.severity_text
    }

    fn severity_number(&self) -> Severity {
        self.severity_number
    }
}

impl SpanData for StartedSpan {
    fn id(&self) -> &Id {
        &self.id
    }

    fn parent(&self) -> Option<Id> {
        self.parent_id.clone()
    }

    fn attributes(&self) -> &dyn AttributeSet {
        &self.attributes
    }
}

impl StartedSpanData for StartedSpan {
    fn start_time(&self) -> SystemTime {
        self.start_time
    }
}

impl ActiveSpanData for StartedSpan {
    fn start_time(&self) -> Option<SystemTime> {
        Some(self.start_time)
    }

    fn end_time(&self) -> Option<SystemTime> {
        None
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct EndedSpan {
    pub parent_id: Option<Id>,
    pub id: Id,
    pub target: String,
    pub name: String,
    #[cfg_attr(feature = "serde", serde(with = "super::time::system_time_serde"))]
    pub start_time: SystemTime,

    #[cfg_attr(feature = "serde", serde(with = "super::time::system_time_serde"))]
    pub end_time: SystemTime,

    /// The original severity string from the source
    pub severity_text: String,

    /// The corresponding severity value, normalized
    pub severity_number: Severity,

    /// Additional attributes associated with this record
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "BTreeMap::is_empty", default)
    )]
    pub attributes: BTreeMap<KeyBuf, AnyValue>,
}

impl EndedSpan {
    pub fn from_data(data: &dyn EndedSpanData) -> Self {
        let parent_id = data.parent().clone();
        let id = data.id().clone();
        let target = data.target().to_string();
        let name = data.name().to_string();
        let severity_number = data.severity_number();
        let severity_text = data.severity_text().to_string();
        let start_time = data.start_time();
        let end_time = data.end_time();
        let mut attributes = BTreeMap::new();
        data.attributes().record(&mut attributes);
        Self {
            parent_id,
            id,
            target,
            name,
            severity_text,
            severity_number,
            start_time,
            end_time,
            attributes,
        }
    }

    pub fn into_start_event(self) -> TelemetryEvent {
        let span = StartedSpan {
            parent_id: self.parent_id,
            id: self.id,
            name: self.name,
            target: self.target,
            start_time: self.start_time,
            severity_text: self.severity_text,
            severity_number: self.severity_number,
            attributes: self.attributes,
        };
        TelemetryEvent::SpanStart(span)
    }
}

impl Metadata for EndedSpan {
    fn target(&self) -> &str {
        &self.target
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn severity_text(&self) -> &str {
        &self.severity_text
    }

    fn severity_number(&self) -> Severity {
        self.severity_number
    }
}

impl SpanData for EndedSpan {
    fn id(&self) -> &Id {
        &self.id
    }

    fn parent(&self) -> Option<Id> {
        self.parent_id.clone()
    }

    fn attributes(&self) -> &dyn AttributeSet {
        &self.attributes
    }
}

impl StartedSpanData for EndedSpan {
    fn start_time(&self) -> SystemTime {
        self.start_time
    }
}

impl EndedSpanData for EndedSpan {
    fn end_time(&self) -> SystemTime {
        self.end_time
    }
}

impl ActiveSpanData for EndedSpan {
    fn start_time(&self) -> Option<SystemTime> {
        Some(self.start_time)
    }

    fn end_time(&self) -> Option<SystemTime> {
        Some(self.end_time)
    }
}

pub enum SpanLifecycle {
    Created(CreatedSpan),
    Started(StartedSpan),
    Ended(EndedSpan),
    Invalid,
}

impl SpanLifecycle {
    pub fn start_with_timestamp(&mut self, start_time: SystemTime) {
        *self = match std::mem::replace(self, Self::Invalid) {
            SpanLifecycle::Created(created_span) => {
                Self::Started(created_span.start_with_timestamp(start_time))
            }
            value => value,
        };
    }

    pub fn start(&mut self) {
        self.start_with_timestamp(SystemTime::now());
    }

    pub fn end_with_timestamp(&mut self, end_time: SystemTime) {
        *self = match std::mem::replace(self, Self::Invalid) {
            SpanLifecycle::Created(created_span) => {
                let started_span = created_span.start_with_timestamp(end_time);
                Self::Ended(started_span.end_with_timestamp(end_time))
            }
            SpanLifecycle::Started(started_span) => {
                Self::Ended(started_span.end_with_timestamp(end_time))
            }
            value => value,
        };
    }

    pub fn end(&mut self) {
        self.end_with_timestamp(SystemTime::now());
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SpanFollowsFrom {
    pub id: Id,
    pub from: Id,

    #[cfg_attr(feature = "serde", serde(with = "super::time::system_time_serde"))]
    pub observed_timestamp: SystemTime,

    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "BTreeMap::is_empty", default)
    )]
    pub attributes: BTreeMap<KeyBuf, AnyValue>,
}

impl SpanFollowsFrom {
    pub fn from_data(id: Id, data: &dyn SpanFollowsFromData) -> Self {
        let from = data.from().clone();
        let observed_timestamp = data.observed_timestamp();
        let mut attributes = BTreeMap::new();
        data.attributes().record(&mut attributes);
        Self {
            id,
            from,
            observed_timestamp,
            attributes,
        }
    }
}

impl SpanFollowsFromData for SpanFollowsFrom {
    fn from(&self) -> &Id {
        &self.from
    }

    fn observed_timestamp(&self) -> SystemTime {
        self.observed_timestamp
    }

    fn attributes(&self) -> &dyn AttributeSet {
        &self.attributes
    }
}

impl SpanFollowsFromDownstreamData for SpanFollowsFrom {
    fn id(&self) -> &Id {
        &self.id
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SpanRecord {
    pub id: Id,

    #[cfg_attr(feature = "serde", serde(with = "super::time::system_time_serde"))]
    pub observed_timestamp: SystemTime,

    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "BTreeMap::is_empty", default)
    )]
    pub attributes: BTreeMap<KeyBuf, AnyValue>,
}

impl SpanRecord {
    pub fn from_data(id: Id, data: &dyn SpanRecordData) -> Self {
        let observed_timestamp = data.observed_timestamp();
        let mut attributes = BTreeMap::new();
        data.attributes().record(&mut attributes);
        Self {
            id,
            observed_timestamp,
            attributes,
        }
    }
}

impl SpanRecordData for SpanRecord {
    fn observed_timestamp(&self) -> SystemTime {
        self.observed_timestamp
    }

    fn attributes(&self) -> &dyn AttributeSet {
        &self.attributes
    }
}

impl SpanRecordDownstreamData for SpanRecord {
    fn id(&self) -> &Id {
        &self.id
    }
}
