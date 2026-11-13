#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(test, deny(warnings))]

//! Implementations of the JSON Timed Stream format.
//! 
//! # Format Spec
//! 
//! Based on [JSON Lines] with a header on the first line followed by lines of
//! events.
//! 
//! The header is the JSON representation of [`TimedStreamHeader`] while the events
//! are JSON representations of [`TimedEvent`] for a given event.
//! 
//! [JSON Lines]: https://jsonlines.org/

mod async_impl;
pub use async_impl::*;

#[cfg(feature = "blocking")]
pub mod blocking;
