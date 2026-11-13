use korio_core::{
    event::NewEventData,
    metadata::{EMPTY_ATTRIBUTE_SET, Key, Severity},
    span::{Id, NewFollowsFromData, NewSpanData, SpanFollowsFromData},
};
use tracing_subscriber::registry::LookupSpan;

pub struct Wrapper<KS> {
    subscriber: KS,
}

impl<KS, S> tracing_subscriber::Layer<S> for Wrapper<KS>
where
    KS: korio_core::Subscriber,
    S: tracing::Subscriber + for<'l> LookupSpan<'l>,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        tracing_id: &tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let tracing_span = ctx
            .span(tracing_id)
            .expect("bug: subscriber should contain span it gave us");
        let parent_id = if let Some(parent) = tracing_span.parent() {
            parent.extensions().get::<Id>().cloned()
        } else {
            None
        };
        let span = WrapSpan {
            parent_id: parent_id.as_ref(),
            tracing_span,
            severity: Severity::Debug,
            severity_text: "Muh!",
            attributes: WrapAttributes(attrs),
        };
        let id = self.subscriber.new_span(&span);
        span.tracing_span.extensions_mut().insert(id);
    }

    fn on_record(
        &self,
        tracing_id: &tracing::span::Id,
        values: &tracing::span::Record<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let tracing_span = ctx
            .span(tracing_id)
            .expect("bug: subscriber should contain span it gave us");
        if let Some(id) = tracing_span.extensions().get::<Id>() {
            let record = WrapRecord { id, values };
            self.subscriber.record(&record);
        }
    }

    fn on_follows_from(
        &self,
        tracing_id: &tracing::span::Id,
        follows: &tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let tracing_span = ctx
            .span(tracing_id)
            .expect("bug: subscriber should contain span it gave us");
        let extensions = tracing_span.extensions();
        let Some(id) = extensions.get::<Id>() else {
            return;
        };

        let Some(follows) = ctx.span(follows) else {
            return;
        };
        if let Some(follows) = follows.extensions().get::<Id>() {
            let follows_from = FollowsFrom(id, follows);
            self.subscriber.record_follows_from(&follows_from);
        }
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let tracing_span = ctx.event_span(event);
        let extensions = tracing_span.as_ref().map(|s| s.extensions());
        let span_id = extensions.as_ref().and_then(|e| e.get::<Id>());
        let wrap = WrapEvent {
            span_id,
            severity_text: todo!(),
            severity: todo!(),
            timestamp: todo!(),
            message: todo!(),
            event,
        };
    }

    fn on_enter(
        &self,
        tracing_id: &tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let tracing_span = ctx
            .span(tracing_id)
            .expect("bug: subscriber should contain span it gave us");
        if let Some(id) = tracing_span.extensions().get::<Id>() {
            self.subscriber.enter(id);
        }
    }

    fn on_exit(
        &self,
        tracing_id: &tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let tracing_span = ctx
            .span(tracing_id)
            .expect("bug: subscriber should contain span it gave us");
        if let Some(id) = tracing_span.extensions().get::<Id>() {
            self.subscriber.exit(&id);
        }
    }

    fn on_close(
        &self,
        tracing_id: tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let tracing_span = ctx
            .span(&tracing_id)
            .expect("bug: subscriber should contain span it gave us");
        if let Some(id) = tracing_span.extensions_mut().remove::<Id>() {
            self.subscriber.try_close(id);
        }
    }
}

pub const SEVERITY_FIELD_NAME: &str = "korio.severity";
pub const SEVERITY_TEXT_FIELD_NAME: &str = "korio.severity_text";

struct SeverityVisitor {
    severity: Option<Severity>,
    severity_text: Option<String>,
}

impl tracing::field::Visit for SeverityVisitor {
    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        if field.name() == SEVERITY_FIELD_NAME {
            self.severity = Some(u16::try_from(value).map(Severity::from).unwrap_or_default());
        }
    }
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == SEVERITY_TEXT_FIELD_NAME {
            self.severity_text = Some(value.to_owned());
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn core::fmt::Debug) {
        if field.name() == SEVERITY_TEXT_FIELD_NAME {
            self.severity_text = Some(format!("{value:?}"));
        }
    }
}

struct WrapEvent<'a> {
    span_id: Option<&'a Id>,
    severity_text: &'a str,
    severity: Severity,
    timestamp: std::time::SystemTime,
    message: Option<&'a str>,
    event: &'a tracing::Event<'a>,
}

impl<'a> NewEventData for WrapEvent<'a> {
    fn span(&self) -> Option<Id> {
        self.span_id.cloned()
    }

    fn name(&self) -> &str {
        self.event.metadata().name()
    }

    fn target(&self) -> &str {
        self.event.metadata().target()
    }

    fn severity_text(&self) -> &str {
        self.severity_text
    }

    fn severity_number(&self) -> Severity {
        self.severity
    }

    fn timestamp(&self) -> std::time::SystemTime {
        self.timestamp
    }

    fn body(&self) -> Option<&dyn korio_core::metadata::Value> {
        self.message
            .as_ref()
            .map(|m| m as &dyn korio_core::metadata::Value)
    }

    fn attributes(&self) -> &dyn korio_core::metadata::AttributeSet {
        self
    }
}

impl<'a> korio_core::metadata::AttributeSet for WrapEvent<'a> {
    fn len(&self) -> usize {
        self.event.metadata().fields().len()
    }

    fn record(&self, visitor: &mut dyn korio_core::metadata::Visit) {
        let mut v = ConvertVisitor(visitor);
        self.event.record(&mut v);
    }
}

pub struct WrapSpan<'a, S: LookupSpan<'a>> {
    parent_id: Option<&'a Id>,
    severity_text: &'a str,
    severity: Severity,
    tracing_span: tracing_subscriber::registry::SpanRef<'a, S>,
    attributes: WrapAttributes<'a>,
}

impl<'a, S> NewSpanData for WrapSpan<'a, S>
where
    S: LookupSpan<'a>,
{
    fn parent(&self) -> Option<Id> {
        self.parent_id.cloned()
    }

    fn target(&self) -> &str {
        self.tracing_span.metadata().target()
    }

    fn name(&self) -> &str {
        self.tracing_span.metadata().name()
    }

    fn severity_text(&self) -> &str {
        self.severity_text
    }

    fn severity_number(&self) -> Severity {
        self.severity
    }

    fn attributes(&self) -> &dyn korio_core::metadata::AttributeSet {
        &self.attributes
    }
}

struct WrapAttributes<'a>(&'a tracing::span::Attributes<'a>);
impl<'a> korio_core::metadata::AttributeSet for WrapAttributes<'a> {
    fn len(&self) -> usize {
        self.0.values().len()
    }

    fn record(&self, visitor: &mut dyn korio_core::metadata::Visit) {
        let mut v = ConvertVisitor(visitor);
        self.0.values().record(&mut v);
    }
}

struct WrapRecord<'r> {
    id: &'r Id,
    values: &'r tracing::span::Record<'r>,
}
impl korio_core::span::SpanRecordData for WrapRecord<'_> {
    fn id(&self) -> &Id {
        self.id
    }

    fn attributes(&self) -> &dyn korio_core::metadata::AttributeSet {
        self
    }
}
impl korio_core::metadata::AttributeSet for WrapRecord<'_> {
    fn len(&self) -> usize {
        self.values.len()
    }

    fn record(&self, visitor: &mut dyn korio_core::metadata::Visit) {
        let mut v = ConvertVisitor(visitor);
        self.values.record(&mut v);
    }
}

struct ConvertVisitor<'v>(&'v mut dyn korio_core::metadata::Visit);

#[forbid(clippy::missing_trait_methods)]
impl tracing::field::Visit for ConvertVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn core::fmt::Debug) {
        let key = Key::new(field.name());
        self.0.record_debug(&key, value);
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        let key = Key::new(field.name());
        self.0.record_f64(&key, value);
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        let key = Key::new(field.name());
        self.0.record_i64(&key, value);
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        let key = Key::new(field.name());
        self.0.record_u64(key, value);
    }

    fn record_i128(&mut self, field: &tracing::field::Field, value: i128) {
        let key = Key::new(field.name());
        self.0.record_i128(key, value);
    }

    fn record_u128(&mut self, field: &tracing::field::Field, value: u128) {
        let key = Key::new(field.name());
        self.0.record_u128(key, value);
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        let key = Key::new(field.name());
        self.0.record_bool(key, value);
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        let key = Key::new(field.name());
        self.0.record_str(key, value);
    }

    fn record_bytes(&mut self, field: &tracing::field::Field, value: &[u8]) {
        let key = Key::new(field.name());
        self.0.record_byte_slice(key, value);
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        let key = Key::new(field.name());
        self.0.record_error(key, value);
    }
}

struct FollowsFrom<'s>(&'s Id, &'s Id);
impl SpanFollowsFromData for FollowsFrom<'_> {
    fn id(&self) -> &Id {
        self.0
    }
}
impl NewFollowsFromData for FollowsFrom<'_> {
    fn follows(&self) -> &Id {
        self.1
    }

    fn attributes(&self) -> &dyn korio_core::metadata::AttributeSet {
        EMPTY_ATTRIBUTE_SET
    }
}
