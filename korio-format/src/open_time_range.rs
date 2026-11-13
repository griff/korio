use std::ops::RangeBounds;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};


/// A range of `SystemTime` with a possible unbounded ending.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OpenTimeRange {
    /// Start of the range
    #[serde(with = "super::time::system_time_serde")]
    pub start: SystemTime,

    /// End of the range
    #[serde(skip_serializing_if="Option::is_none", default, with = "super::time::option_system_time_serde")]
    pub end: Option<SystemTime>,
}

impl OpenTimeRange {
    /// Create a range that goes from [`SystemTime::now()`] with no ending.
    pub fn from_now() -> Self {
        Self { start: crate::time::now(), end: None }
    }

    /// Return the duration between start and end of this range.
    pub fn duration(&self) -> Option<Duration> {
        if let Some(end_time) = self.end {
            end_time.duration_since(self.start).ok()
        } else {
            None
        }
    }
}

impl RangeBounds<SystemTime> for OpenTimeRange {
    fn start_bound(&self) -> std::ops::Bound<&SystemTime> {
        std::ops::Bound::Included(&self.start)
    }

    fn end_bound(&self) -> std::ops::Bound<&SystemTime> {
        if let Some(end_time) = self.end.as_ref() {
            std::ops::Bound::Excluded(end_time)
        } else {
            std::ops::Bound::Unbounded
        }
    }
}