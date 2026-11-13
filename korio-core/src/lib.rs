#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(test, deny(warnings))]

//! Core types for Korio.

pub mod dispatcher;
pub mod event;
pub mod metadata;
pub mod span;
pub mod subscriber;

pub use self::metadata::Severity;
pub use self::subscriber::Subscriber;

#[doc(hidden)]
pub mod _exported {
    pub use smol_str;
}
