use std::time::SystemTime;

use serde::{Deserialize, Serialize};

/// An event with a timestamp when it occurred.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimedEvent<I> {
    /// Timestamp that this event was registered.
    #[serde(with = "super::time::system_time_serde")]
    pub timestamp: SystemTime,

    /// The actual event.
    pub event: I
}
