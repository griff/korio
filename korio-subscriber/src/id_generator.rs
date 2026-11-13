use std::fmt;
use std::num::{NonZeroU64, NonZeroU128};

use korio_core::span::{SpanId, TraceId};
use rand::RngExt as _;

pub trait IdGenerator: Send + Sync + fmt::Debug {
    fn generate_trace_id(&self) -> TraceId;
    fn generate_span_id(&self) -> SpanId;
}

#[derive(Debug, Clone, Default)]
pub struct RandomIdGenerator();

impl IdGenerator for RandomIdGenerator {
    fn generate_trace_id(&self) -> TraceId {
        TraceId::from(rand::rng().random::<NonZeroU128>())
    }

    fn generate_span_id(&self) -> SpanId {
        SpanId::from(rand::rng().random::<NonZeroU64>())
    }
}
