use std::time::SystemTime;

use super::Id;
use crate::metadata::AttributeSet;

/// Record of updated or new attribute for a span.
pub trait SpanRecordData {
    /// Return [`SystemTime`] representing when this event was first seen.
    fn observed_timestamp(&self) -> SystemTime;

    /// Return new attributes of this span.
    fn attributes(&self) -> &dyn AttributeSet;
}

/// Record of updated or new attribute for a downstream span.
pub trait SpanRecordDownstreamData: SpanRecordData {
    /// Return id of span this record is about.
    fn id(&self) -> &Id;
}

impl SpanRecordData for &dyn AttributeSet {
    fn observed_timestamp(&self) -> SystemTime {
        SystemTime::now()
    }

    fn attributes(&self) -> &dyn AttributeSet {
        self
    }
}

/// An extension trait for `SpanRecordData` for recording.
pub trait SpanRecordDownstreamDataRecord: Sized {
    /// Record this data to the current `Dispatch`.
    fn record_all(&self);
}

impl<E: SpanRecordDownstreamData> SpanRecordDownstreamDataRecord for E {
    fn record_all(&self) {
        crate::dispatcher::get_default(|dispatch| dispatch.record(self.id(), self))
    }
}
