use std::{fmt, sync::Arc};

use crate::metadata::MetadataProvider;

use super::Id;

enum CurrentInner {
    Current {
        id: Id,
        metadata: Arc<dyn MetadataProvider>,
    },
    None,
    Unknown,
}

/// The currently active span.
#[derive(Debug)]
pub struct Current {
    inner: CurrentInner,
}

impl Current {
    /// Return a [`Current`] where the span of the provided [`Id`] is active.
    pub fn new(id: Id, metadata: Arc<dyn MetadataProvider>) -> Self {
        Self {
            inner: CurrentInner::Current { id, metadata },
        }
    }

    /// Return a [`Current`] with no active span.
    pub fn none() -> Self {
        Self {
            inner: CurrentInner::None,
        }
    }

    /// Return a [`Current`] where that marks that a subscriber does not track active span.
    pub(crate) fn unknown() -> Self {
        Self {
            inner: CurrentInner::Unknown,
        }
    }

    /// Return whether the status of the active span is known.
    pub fn is_known(&self) -> bool {
        !matches!(self.inner, CurrentInner::Unknown)
    }

    /// Return [`Id`] of the active span if known.
    pub fn id(&self) -> Option<&Id> {
        match self.inner {
            CurrentInner::Current { ref id, .. } => Some(id),
            _ => None,
        }
    }

    /// Return the inner [`Id`] if there is an active span.
    pub fn into_inner(self) -> Option<Id> {
        match self.inner {
            CurrentInner::Current { id, .. } => Some(id),
            _ => None,
        }
    }
}

impl<'c> From<&'c Current> for Option<&'c Id> {
    fn from(value: &'c Current) -> Self {
        value.id()
    }
}

impl From<&'_ Current> for Option<Id> {
    fn from(value: &'_ Current) -> Self {
        value.id().cloned()
    }
}

impl From<Current> for Option<Id> {
    fn from(value: Current) -> Self {
        value.into_inner()
    }
}

impl fmt::Debug for CurrentInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Current { id, metadata } => {
                let metadata = fmt::from_fn(|f| {
                    if let Some(metadata) = metadata.metadata() {
                        f.debug_struct("Metadata")
                            .field("name", &metadata.name())
                            .field("target", &metadata.target())
                            .field("severity_number", &metadata.severity_number())
                            .field("severity_text", &metadata.severity_text())
                            .finish()
                    } else {
                        f.write_str("None")
                    }
                });
                f.debug_struct("Current")
                    .field("id", id)
                    .field("metadata", &metadata)
                    .finish()
            }
            Self::None => write!(f, "None"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}
