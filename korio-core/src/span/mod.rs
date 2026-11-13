//! Spans are tasks that cover a given timespan.

mod current;
mod extensions;
mod follows_from_data;
mod id;
mod span_data;
mod span_record;

pub use current::Current;
pub use extensions::{ExtensionSet, Extensions, ExtensionsMut};
pub use follows_from_data::{SpanFollowsFromData, SpanFollowsFromDownstreamData};
pub use id::{Id, SpanId, TraceId};
pub use span_data::{
    ActiveSpanData, EndedSpanData, NewSpanData, RegisteredSpanData, SpanData, StartedSpanData,
};
pub use span_record::{SpanRecordData, SpanRecordDownstreamData, SpanRecordDownstreamDataRecord};
