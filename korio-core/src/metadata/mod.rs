//! Core data traits for Korio.

mod any_value;
mod attributes;
mod severity;
mod value;

pub use any_value::AnyValue;
pub use attributes::{AsKey, AttributeSet, EMPTY_ATTRIBUTE_SET, Key, KeyBuf, KeyValue, Visit};
use auto_impl::auto_impl;
pub use severity::Severity;
pub use value::{DebugValue, DisplayValue, Value, debug, display};

/// Metadata describing an event or span.
#[auto_impl(&, Box, Arc)]
pub trait Metadata {
    /// Return name of the event or span.
    fn name(&self) -> &str;

    /// Return target of the event or span.
    fn target(&self) -> &str;

    /// Return textual version of the severity.
    fn severity_text(&self) -> &str {
        self.severity_number().as_str()
    }

    /// Return severity of event or span as an enum.
    fn severity_number(&self) -> Severity;
}

/// Helper trait for giving access to metadata that might be unknown.
pub trait MetadataProvider {
    /// Return metadata if known.
    fn metadata(&self) -> Option<&dyn Metadata>;
}
