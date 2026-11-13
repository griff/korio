use std::{fmt, time::SystemTime};

use auto_impl::auto_impl;

use super::Id;
use crate::metadata::{AttributeSet, EMPTY_ATTRIBUTE_SET, Metadata, Severity};

/// Data for creating a span.
#[auto_impl(&, Box)]
pub trait NewSpanData: Metadata {
    /// Return true if the new span should be a root.
    fn is_root(&self) -> bool {
        false
    }

    /// Return true if the new span should use currently active span as a parent.
    fn is_contextual(&self) -> bool {
        self.parent().is_none()
    }

    /// Return explicit parent id of this span.
    /// `None` will use the currently active span as a parent.
    fn parent(&self) -> Option<Id> {
        None
    }

    /// Return attributes of this span.
    fn attributes(&self) -> &dyn AttributeSet {
        EMPTY_ATTRIBUTE_SET
    }
}

/// Data for a created span.
#[auto_impl(&, Box)]
pub trait SpanData: Metadata {
    /// Id for this span.
    fn id(&self) -> &Id;

    /// Return parent id of this span. If this is a root span `None` is returned.
    fn parent(&self) -> Option<Id> {
        None
    }

    /// Return attributes of this span.
    fn attributes(&self) -> &dyn AttributeSet {
        EMPTY_ATTRIBUTE_SET
    }
}

/// Data for a started span.
#[auto_impl(&, Box)]
pub trait StartedSpanData: SpanData {
    /// Return [`SystemTime`] for when this span was started.
    fn start_time(&self) -> SystemTime;
}

/// Data for an ended span.
#[auto_impl(&, Box)]
pub trait EndedSpanData: StartedSpanData {
    /// Return [`SystemTime`] for when this span ended.
    fn end_time(&self) -> SystemTime;
}

/// Data for an active span.
///
/// An active span is one that has been created but not yet dropped.
#[auto_impl(&, Box)]
pub trait ActiveSpanData: SpanData {
    /// Return [`SystemTime`] for when this span was started.
    fn start_time(&self) -> Option<SystemTime>;

    /// Return [`SystemTime`] for when this span ended.
    fn end_time(&self) -> Option<SystemTime>;
}

/// A mapping from [`NewSpanData`] to [`SpanData`].
pub struct RegisteredSpanData<'e> {
    /// Id of this new span.
    pub id: &'e Id,
    /// Resolved span that this event belongs to.
    pub parent: Option<Id>,
    /// The data for this span
    pub span: &'e dyn NewSpanData,
}

impl fmt::Debug for RegisteredSpanData<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegisteredSpanData")
            .field("id", &self.id)
            .field("parent", &self.parent)
            .field("name", &self.span.name())
            .field("target", &self.span.target())
            .finish()
    }
}

impl Metadata for RegisteredSpanData<'_> {
    fn name(&self) -> &str {
        self.span.name()
    }

    fn target(&self) -> &str {
        self.span.target()
    }

    fn severity_number(&self) -> Severity {
        self.span.severity_number()
    }

    fn severity_text(&self) -> &str {
        self.span.severity_text()
    }
}

impl SpanData for RegisteredSpanData<'_> {
    fn id(&self) -> &Id {
        self.id
    }

    fn parent(&self) -> Option<Id> {
        self.parent.clone()
    }

    fn attributes(&self) -> &dyn AttributeSet {
        self.span.attributes()
    }
}
