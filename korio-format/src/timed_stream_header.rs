use serde::{Deserialize, Serialize};

use crate::OpenTimeRange;


/// Header for several formats.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TimedStreamHeader {
    /// The time range that this stream covers.
    pub time_range: OpenTimeRange,
}

impl Default for TimedStreamHeader {
    fn default() -> Self {
        Self { time_range: OpenTimeRange::from_now() }
    }
}
