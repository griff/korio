use std::borrow::Cow;

use korio_core::metadata::{KeyBuf, Metadata};

use super::{Id, NewSpanData, Parent};
use crate::metadata::{AttributeSet, ExtraAttributes, KeyValue, Severity, Value};

pub struct RefSpanBuilder<'n, 't, 's, A> {
    parent: Parent,
    target: Cow<'t, str>,
    name: Cow<'n, str>,
    severity_number: Severity,
    attributes: ExtraAttributes<A>,
    severity_text: Option<Cow<'s, str>>,
}

impl<'n, 't> RefSpanBuilder<'n, 't, 'static, &'static dyn AttributeSet> {
    pub fn new<T, N>(target: T, name: N, severity_number: Severity) -> Self
    where
        T: Into<Cow<'t, str>>,
        N: Into<Cow<'n, str>>,
    {
        Self {
            parent: Parent::Current,
            target: target.into(),
            name: name.into(),
            severity_number,
            //start_time: Default::default(),
            attributes: Default::default(),
            severity_text: Default::default(),
        }
    }

    pub fn child_of<P, T, N>(parent: P, target: T, name: N, severity_number: Severity) -> Self
    where
        P: Into<Option<Id>>,
        T: Into<Cow<'t, str>>,
        N: Into<Cow<'n, str>>,
    {
        let parent = match parent.into() {
            Some(id) => Parent::Explicit(id),
            None => Parent::Root,
        };
        Self {
            parent,
            target: target.into(),
            name: name.into(),
            severity_number,
            //start_time: Default::default(),
            attributes: Default::default(),
            severity_text: Default::default(),
        }
    }
}

impl<'n, 't, 's, A: AttributeSet> RefSpanBuilder<'n, 't, 's, A> {
    pub fn with_attribute<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<KeyBuf>,
        V: Value,
    {
        self.attributes.extra.push(KeyValue::with_value(key, value));
        self
    }

    pub fn with_opt_attribute<K, V>(self, key: K, value: Option<V>) -> Self
    where
        K: Into<KeyBuf>,
        V: Value,
    {
        let Some(value) = value else {
            return self;
        };
        self.with_attribute(key, value)
    }

    pub fn with_attributes<NA>(self, attributes: NA) -> RefSpanBuilder<'n, 't, 's, NA> {
        RefSpanBuilder {
            parent: self.parent,
            name: self.name,
            target: self.target,
            severity_number: self.severity_number,
            //start_time: self.start_time,
            attributes: ExtraAttributes {
                attributes,
                extra: self.attributes.extra,
            },
            severity_text: self.severity_text,
        }
    }

    #[doc(hidden)]
    pub fn record_all<NA>(self, attributes: NA) -> RefSpanBuilder<'n, 't, 's, NA> {
        self.with_attributes(attributes)
    }

    pub fn with_severity_text<'s2, T>(self, text: T) -> RefSpanBuilder<'n, 't, 's2, A>
    where
        T: Into<Cow<'s2, str>>,
    {
        RefSpanBuilder {
            parent: self.parent,
            name: self.name,
            target: self.target,
            severity_number: self.severity_number,
            //start_time: self.start_time,
            attributes: self.attributes,
            severity_text: Some(text.into()),
        }
    }
}

impl<'n, 't, 's, A: AttributeSet> Metadata for RefSpanBuilder<'n, 't, 's, A> {
    fn name(&self) -> &str {
        &self.name
    }

    fn target(&self) -> &str {
        &self.target
    }

    fn severity_text(&self) -> &str {
        self.severity_text
            .as_deref()
            .unwrap_or_else(|| self.severity_number.as_str())
    }

    fn severity_number(&self) -> Severity {
        self.severity_number
    }
}

impl<'n, 't, 's, A: AttributeSet> NewSpanData for RefSpanBuilder<'n, 't, 's, A> {
    fn attributes(&self) -> &dyn AttributeSet {
        &self.attributes
    }

    fn is_root(&self) -> bool {
        matches!(self.parent, Parent::Root)
    }

    fn is_contextual(&self) -> bool {
        matches!(self.parent, Parent::Current)
    }

    fn parent(&self) -> Option<Id> {
        match self.parent {
            Parent::Explicit(ref id) => Some(id.clone()),
            _ => None,
        }
    }
}
