use std::{
    fmt,
    num::{NonZeroU64, NonZeroU128},
};

/// The full identifier for a span.
///
/// This includes both the actual [`SpanId`] as well as the [`TraceId`] that
/// this span belongs to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Id {
    /// Id of the trace that this span belongs to.
    pub trace_id: TraceId,
    /// Local span id within the trace.
    pub span_id: SpanId,
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.trace_id, self.span_id)
    }
}

impl<'a> From<&'a Id> for Option<Id> {
    fn from(id: &'a Id) -> Self {
        Some(id.clone())
    }
}

/// Identifier for a trace.
///
/// Each trace is uniquely identified by one global id that is generated at the
/// start of the trace. A given trace then contain multiple spans.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TraceId(NonZeroU128);

impl TraceId {
    /// Constructs a new trace ID from the given `u128`.
    ///
    /// <pre class="ignore" style="white-space:normal;font:inherit;">
    ///     <strong>Note</strong>: Trace IDs must be greater than zero.
    /// </pre>
    ///
    /// # Panics
    /// - If the provided `u128` is 0.
    pub fn from_u128(u: u128) -> Self {
        TraceId(NonZeroU128::new(u).expect("trace IDs must be > 0"))
    }

    /// Constructs a new trace ID from the given `NonZeroU128`.
    ///
    /// Unlike [`TraceId::from_u128`](TraceId::from_u128()), this will never panic.
    #[inline]
    pub const fn from_non_zero_u128(id: NonZeroU128) -> Self {
        TraceId(id)
    }

    // Allow `into` by-ref since we don't want to impl Copy for Id
    #[allow(clippy::wrong_self_convention)]
    /// Returns the span's ID as a `u128`.
    pub fn into_u128(&self) -> u128 {
        self.0.get()
    }

    // Allow `into` by-ref since we don't want to impl Copy for Id
    #[allow(clippy::wrong_self_convention)]
    /// Returns the span's ID as a `NonZeroU128`.
    #[inline]
    pub const fn into_non_zero_u128(&self) -> NonZeroU128 {
        self.0
    }
}

impl From<NonZeroU128> for TraceId {
    fn from(value: NonZeroU128) -> Self {
        TraceId(value)
    }
}

impl fmt::LowerHex for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl fmt::Debug for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:032x}")
    }
}

/// Identifier for a span in a trace.
///
/// This is unique within a trace.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SpanId(NonZeroU64);

impl SpanId {
    /// Constructs a new span ID from the given `u64`.
    ///
    /// <pre class="ignore" style="white-space:normal;font:inherit;">
    ///     <strong>Note</strong>: Span IDs must be greater than zero.
    /// </pre>
    ///
    /// # Panics
    /// - If the provided `u64` is 0.
    pub fn from_u64(u: u64) -> Self {
        SpanId(NonZeroU64::new(u).expect("span IDs must be > 0"))
    }

    /// Constructs a new span ID from the given `NonZeroU64`.
    ///
    /// Unlike [`Id::from_u64`](Id::from_u64()), this will never panic.
    #[inline]
    pub const fn from_non_zero_u64(id: NonZeroU64) -> Self {
        SpanId(id)
    }

    // Allow `into` by-ref since we don't want to impl Copy for Id
    #[allow(clippy::wrong_self_convention)]
    /// Returns the span's ID as a `u64`.
    pub fn into_u64(&self) -> u64 {
        self.0.get()
    }

    // Allow `into` by-ref since we don't want to impl Copy for Id
    #[allow(clippy::wrong_self_convention)]
    /// Returns the span's ID as a `NonZeroU64`.
    #[inline]
    pub const fn into_non_zero_u64(&self) -> NonZeroU64 {
        self.0
    }
}

impl fmt::LowerHex for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl fmt::Debug for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:016x}")
    }
}

impl From<NonZeroU64> for SpanId {
    fn from(value: NonZeroU64) -> Self {
        Self(value)
    }
}

impl<'a> From<&'a SpanId> for Option<SpanId> {
    fn from(id: &'a SpanId) -> Self {
        Some(*id)
    }
}
