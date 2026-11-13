use std::fmt;
use std::time::SystemTime;

use crate::dispatcher;
use crate::metadata::{AttributeSet, EMPTY_ATTRIBUTE_SET, Metadata, Severity, Value};
use crate::span::Id;

/// Trait for a new event yet to be registered with a subscriber.
pub trait NewEventData: Metadata {
    /// Return true if the new event should be a root.
    fn is_root(&self) -> bool {
        false
    }

    /// Return true if the new event should belong to the currently active span.
    fn is_contextual(&self) -> bool {
        self.span().is_none()
    }

    /// Return explicit id of the span that this event is part of.
    ///
    /// When the span is the implicit currently active span this returns
    /// `None`.
    fn span(&self) -> Option<Id> {
        None
    }

    /// Return [`SystemTime`] representing when this event occurred.
    fn timestamp(&self) -> SystemTime;

    /// Return [`Value`] representing the body if present.
    fn body(&self) -> Option<&dyn Value> {
        None
    }

    /// Return attributes of this span.
    fn attributes(&self) -> &dyn AttributeSet {
        EMPTY_ATTRIBUTE_SET
    }
}

/// A data for a registered event.
pub trait EventData: Metadata {
    /// Return [`SystemTime`] representing when this event was first seen.
    fn observed_timestamp(&self) -> SystemTime {
        self.timestamp()
    }

    /// Return id of the span that this event is part of.
    fn span(&self) -> Option<Id> {
        None
    }

    /// Return [`SystemTime`] representing when this event occurred.
    fn timestamp(&self) -> SystemTime;

    /// Return [`Value`] representing the body if present.
    fn body(&self) -> Option<&dyn Value> {
        None
    }

    /// Return attributes of this span.
    fn attributes(&self) -> &dyn AttributeSet {
        EMPTY_ATTRIBUTE_SET
    }
}

/// An extension trait for `NewEventData` for emitting events.
pub trait NewEventDataEmit: Sized {
    /// Emit an event of this data to the current `Dispatch`.
    fn emit(&self);
}

impl<E: NewEventData> NewEventDataEmit for E {
    fn emit(&self) {
        dispatcher::get_default(|dispatch| dispatch.new_event(self))
    }
}

/// An extension trait for `EventData` for emitting downstream events.
pub trait EventDataEmit: Sized {
    /// Emit a downstream event of this data to the current `Dispatch`.
    fn emit_downstream(&self);
}

impl<E: EventData> EventDataEmit for E {
    fn emit_downstream(&self) {
        dispatcher::get_default(|dispatch| dispatch.downstream_event(self))
    }
}

/// A mapping from [`NewEventData`] to [`EventData`].
pub struct RegisteredEventData<'e> {
    /// Resolved span that this event belongs to.
    pub span: Option<Id>,
    /// When this event was observed.
    pub observed_timestamp: SystemTime,
    /// The event data for the event.
    pub event: &'e dyn NewEventData,
}

impl fmt::Debug for RegisteredEventData<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegisteredEventData")
            .field("span", &self.span)
            .field("observed_timestamp", &self.observed_timestamp)
            .field("name", &self.event.name())
            .field("target", &self.event.target())
            .finish()
    }
}

impl Metadata for RegisteredEventData<'_> {
    fn name(&self) -> &str {
        self.event.name()
    }

    fn target(&self) -> &str {
        self.event.target()
    }

    fn severity_number(&self) -> Severity {
        self.event.severity_number()
    }

    fn severity_text(&self) -> &str {
        self.event.severity_text()
    }
}

impl EventData for RegisteredEventData<'_> {
    fn timestamp(&self) -> SystemTime {
        self.event.timestamp()
    }

    fn observed_timestamp(&self) -> SystemTime {
        self.observed_timestamp
    }

    fn span(&self) -> Option<Id> {
        self.span.clone()
    }

    fn body(&self) -> Option<&dyn Value> {
        self.event.body()
    }

    fn attributes(&self) -> &dyn AttributeSet {
        self.event.attributes()
    }
}
