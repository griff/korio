use std::{ops::Range, time::SystemTime};

use serde::{Deserialize, Serialize};


/// An event with a time range associated that marks when the event was active.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TimedRange<I> {
    /// Time range where this event was active.
    #[serde(with = "super::time::system_time_range_serde")]
    pub range: Range<SystemTime>,

    /// The actual event.
    pub event: I,
}