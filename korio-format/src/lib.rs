#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(test, deny(warnings))]

//! # korio-format
//!
//! Helpers for the different kinds for streaming and storage formats supported
//! by `korio`. This crate can also be used to implement your own formats.

mod open_time_range;
pub mod time;
mod timed_event;
mod timed_events;
mod timed_range;
mod timed_stream_header;

pub use open_time_range::OpenTimeRange;
pub use timed_event::TimedEvent;
pub use timed_events::{TimedEvents, TimingEvents, WrapEvents};
pub use timed_range::TimedRange;
pub use timed_stream_header::TimedStreamHeader;
