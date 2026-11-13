use std::collections::BTreeMap;
use std::time::SystemTime;

use korio_core::event::{EventData, NewEventData};
use korio_core::metadata::{AnyValue, AttributeSet, KeyBuf, Metadata, Severity, Value};
use korio_core::span::Id;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Event {
    pub span: Option<Id>,

    /// Target of the log record
    pub target: String,

    pub name: String,

    /// Record timestamp
    #[cfg_attr(feature = "serde", serde(with = "super::time::system_time_serde"))]
    pub timestamp: SystemTime,

    /// Timestamp for when the record was observed by OpenTelemetry
    #[cfg_attr(feature = "serde", serde(with = "super::time::system_time_serde"))]
    pub observed_timestamp: SystemTime,

    /// The original severity string from the source
    pub severity_text: String,

    /// The corresponding severity value, normalized
    pub severity_number: Severity,

    /// Record body
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub body: Option<AnyValue>,

    /// Additional attributes associated with this record
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "BTreeMap::is_empty", default)
    )]
    pub attributes: BTreeMap<KeyBuf, AnyValue>,
}

impl Event {
    pub fn from_new_data(span: Option<Id>, data: &dyn NewEventData) -> Self {
        let target = data.target().to_string();
        let name = data.name().to_string();
        let timestamp = data.timestamp();
        let observed_timestamp = SystemTime::now();
        let severity_text = data.severity_text().to_string();
        let severity_number = data.severity_number();
        let body = data.body().map(AnyValue::from_value);
        let mut attributes = BTreeMap::new();
        data.attributes().record(&mut attributes);
        Self {
            span,
            target,
            name,
            timestamp,
            observed_timestamp,
            severity_text,
            severity_number,
            body,
            attributes,
        }
    }

    pub fn from_data(data: &dyn EventData) -> Self {
        let span = data.span();
        let target = data.target().to_string();
        let name = data.name().to_string();
        let timestamp = data.timestamp();
        let observed_timestamp = data.observed_timestamp();
        let severity_text = data.severity_text().to_string();
        let severity_number = data.severity_number();
        let body = data.body().map(AnyValue::from_value);
        let mut attributes = BTreeMap::new();
        data.attributes().record(&mut attributes);
        Self {
            span,
            target,
            name,
            timestamp,
            observed_timestamp,
            severity_text,
            severity_number,
            body,
            attributes,
        }
    }

    pub fn body(&self) -> Option<&AnyValue> {
        self.body.as_ref()
    }

    pub fn attributes(&self) -> impl ExactSizeIterator<Item = (&KeyBuf, &AnyValue)> {
        self.attributes.iter()
    }
}

impl Metadata for Event {
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

impl EventData for Event {
    fn observed_timestamp(&self) -> SystemTime {
        self.observed_timestamp
    }

    fn attributes(&self) -> &dyn AttributeSet {
        &self.attributes
    }

    fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    fn body(&self) -> Option<&dyn Value> {
        self.body.as_ref().map(|v| v as &dyn Value)
    }

    fn span(&self) -> Option<Id> {
        self.span.clone()
    }
}
