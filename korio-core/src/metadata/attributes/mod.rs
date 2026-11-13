//! Key-value data for events and spans.

mod attribute_set;
mod key;
mod key_value;
mod visit;

pub use attribute_set::{AttributeSet, EMPTY_ATTRIBUTE_SET};
pub use key::*;
pub use key_value::KeyValue;
pub use visit::Visit;
