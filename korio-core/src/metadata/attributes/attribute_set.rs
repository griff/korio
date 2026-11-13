use auto_impl::auto_impl;

use crate::metadata::{Key, Value};

use super::Visit;

/// A set of [`Field`]s and their corresponding values.
#[auto_impl(&, Box)]
pub trait AttributeSet {
    /// Returns number of attributes.
    fn len(&self) -> usize;

    /// Returns true if this [`AttributeSet`] is empty.
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Record the attributes in this [`AttributeSet`] to the provided visitor.
    fn record(&self, visitor: &mut dyn Visit);
}

struct EmptyAttributeSet;
impl AttributeSet for EmptyAttributeSet {
    fn record(&self, _visitor: &mut dyn Visit) {}

    #[inline(always)]
    fn len(&self) -> usize {
        0
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        true
    }
}

/// An empty [`AttrbuteSet`].
pub const EMPTY_ATTRIBUTE_SET: &'static dyn AttributeSet = &EmptyAttributeSet;

impl<const N: usize> AttributeSet for [(&Key, &dyn Value); N] {
    fn len(&self) -> usize {
        N
    }

    fn record(&self, visitor: &mut dyn Visit) {
        for (key, value) in self.iter() {
            value.record(key, visitor);
        }
    }
}
