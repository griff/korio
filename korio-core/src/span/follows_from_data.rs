use std::time::SystemTime;

use super::Id;
use crate::metadata::{AttributeSet, EMPTY_ATTRIBUTE_SET};

/// Data for when one span follows from another.
pub trait SpanFollowsFromData {
    /// Return id of span that this follows from.
    fn from(&self) -> &Id;

    /// Return [`SystemTime`] representing when this event was first seen.
    fn observed_timestamp(&self) -> SystemTime;

    /// Return attributes of this link between spans.
    fn attributes(&self) -> &dyn AttributeSet;
}

impl SpanFollowsFromData for Id {
    fn from(&self) -> &Id {
        self
    }

    fn observed_timestamp(&self) -> SystemTime {
        SystemTime::now()
    }

    fn attributes(&self) -> &dyn AttributeSet {
        EMPTY_ATTRIBUTE_SET
    }
}

/// Data for when one span follows from another.
pub trait SpanFollowsFromDownstreamData: SpanFollowsFromData {
    /// [`Id`] of the span that follows from another span.
    fn id(&self) -> &Id;
}
