pub use korio_core::metadata::*;

pub(crate) struct ExtraAttributes<A> {
    pub attributes: A,
    pub extra: Vec<KeyValue>,
}

impl Default for ExtraAttributes<&'static dyn AttributeSet> {
    fn default() -> Self {
        Self {
            attributes: EMPTY_ATTRIBUTE_SET,
            extra: Default::default(),
        }
    }
}
impl<A: AttributeSet> AttributeSet for ExtraAttributes<A> {
    fn record(&self, visitor: &mut dyn Visit) {
        self.attributes.record(visitor);
        self.extra.record(visitor);
    }

    fn len(&self) -> usize {
        self.attributes.len() + self.extra.len()
    }

    fn is_empty(&self) -> bool {
        self.attributes.is_empty() && self.extra.is_empty()
    }
}
