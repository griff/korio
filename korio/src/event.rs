use std::{borrow::Cow, time::SystemTime};

use crate::metadata::{AttributeSet, ExtraAttributes, KeyValue, Severity, Value};
use crate::span::{Id, Parent};

#[doc(inline)]
pub use korio_core::event::*;
use korio_core::metadata::{KeyBuf, Metadata};

pub struct RefEventBuilder<'n, 't, 's, 'b, A> {
    parent: Parent,
    target: Cow<'t, str>,
    name: Cow<'n, str>,
    severity_text: Option<Cow<'s, str>>,
    severity_number: Severity,
    timestamp: Option<SystemTime>,
    observed_timestamp: Option<SystemTime>,
    body: Option<&'b dyn Value>,
    attributes: ExtraAttributes<A>,
}

impl<'n, 't> RefEventBuilder<'n, 't, 'static, 'static, &'static dyn AttributeSet> {
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
            severity_text: Default::default(),
            timestamp: Default::default(),
            observed_timestamp: Default::default(),
            body: Default::default(),
            attributes: Default::default(),
        }
    }

    pub fn new_child_of<P, T, N>(parent: P, target: T, name: N, severity_number: Severity) -> Self
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
            severity_text: Default::default(),
            timestamp: Default::default(),
            observed_timestamp: Default::default(),
            body: Default::default(),
            attributes: Default::default(),
        }
    }
}

impl<'n, 't, 's, 'b, A: AttributeSet> RefEventBuilder<'n, 't, 's, 'b, A> {
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

    pub fn with_attributes<NA>(self, attributes: NA) -> RefEventBuilder<'n, 't, 's, 'b, NA> {
        RefEventBuilder {
            parent: self.parent,
            target: self.target,
            name: self.name,
            severity_number: self.severity_number,
            timestamp: self.timestamp,
            observed_timestamp: self.observed_timestamp,
            severity_text: self.severity_text,
            body: self.body,
            attributes: ExtraAttributes {
                attributes,
                extra: self.attributes.extra,
            },
        }
    }

    /// Sets the time when the event occurred measured by the origin clock, i.e. the time at the source.
    pub fn with_timestamp(mut self, timestamp: SystemTime) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Sets the observed event timestamp.
    pub fn with_observed_timestamp(mut self, timestamp: SystemTime) -> Self {
        self.observed_timestamp = Some(timestamp);
        self
    }

    /// Sets severity as text.
    pub fn with_severity_text<'s2, T>(self, text: T) -> RefEventBuilder<'n, 't, 's2, 'b, A>
    where
        T: Into<Cow<'s2, str>>,
    {
        RefEventBuilder {
            parent: self.parent,
            target: self.target,
            name: self.name,
            severity_number: self.severity_number,
            timestamp: self.timestamp,
            observed_timestamp: self.observed_timestamp,
            severity_text: Some(text.into()),
            body: self.body,
            attributes: self.attributes,
        }
    }

    /// Sets the message body of the log.
    pub fn with_body<'b2>(self, body: &'b2 dyn Value) -> RefEventBuilder<'n, 't, 's, 'b2, A> {
        RefEventBuilder {
            parent: self.parent,
            target: self.target,
            name: self.name,
            severity_number: self.severity_number,
            timestamp: self.timestamp,
            observed_timestamp: self.observed_timestamp,
            severity_text: self.severity_text,
            body: Some(body),
            attributes: self.attributes,
        }
    }
}

impl<'n, 't, 's, 'b, A: AttributeSet> Metadata for RefEventBuilder<'n, 't, 's, 'b, A> {
    fn target(&self) -> &str {
        &self.target
    }

    fn name(&self) -> &str {
        &self.name
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

impl<'n, 't, 's, 'b, A: AttributeSet> NewEventData for RefEventBuilder<'n, 't, 's, 'b, A> {
    fn is_root(&self) -> bool {
        matches!(self.parent, Parent::Root)
    }

    fn is_contextual(&self) -> bool {
        matches!(self.parent, Parent::Current)
    }

    fn span(&self) -> Option<Id> {
        match self.parent {
            Parent::Explicit(ref id) => Some(id.clone()),
            _ => None,
        }
    }

    fn timestamp(&self) -> SystemTime {
        self.timestamp
            .or(self.observed_timestamp)
            .unwrap_or_else(crate::time::now)
    }

    fn body(&self) -> Option<&dyn Value> {
        self.body.as_ref().copied()
    }

    fn attributes(&self) -> &dyn AttributeSet {
        &self.attributes
    }
}
