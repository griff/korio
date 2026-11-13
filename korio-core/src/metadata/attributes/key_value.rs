use super::{AttributeSet, KeyBuf, Visit};
use crate::metadata::{AnyValue, Key, Value};

/// A pair of [`Key`] and [`AnyValue`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct KeyValue {
    /// The key of this pair.
    pub key: KeyBuf,
    /// The value of this pair.
    pub value: AnyValue,
}

impl KeyValue {
    /// Construct a new `KeyValue` with provided `key` and `value`.
    pub fn new<K, V>(key: K, value: V) -> Self
    where
        K: Into<KeyBuf>,
        V: Into<AnyValue>,
    {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    /// Construct a new `KeyValue` with provided `key` and `value`.
    pub fn with_value<K, V>(key: K, value: V) -> Self
    where
        K: Into<KeyBuf>,
        V: Value,
    {
        Self {
            key: key.into(),
            value: AnyValue::from_value(value),
        }
    }
}

impl AttributeSet for KeyValue {
    fn len(&self) -> usize {
        1
    }

    fn record(&self, visitor: &mut dyn Visit) {
        self.value.record(&self.key, visitor);
    }
}

impl From<(KeyBuf, AnyValue)> for KeyValue {
    fn from((key, value): (KeyBuf, AnyValue)) -> Self {
        KeyValue::new(key, value)
    }
}

impl AttributeSet for Vec<KeyValue> {
    fn record(&self, visitor: &mut dyn Visit) {
        for kv in self.iter() {
            kv.record(visitor);
        }
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }
}

#[forbid(clippy::missing_trait_methods)]
impl Visit for Vec<KeyValue> {
    fn record_bool(&mut self, field: &Key, value: bool) {
        let mut v = AnyValue::Empty;
        v.record_bool(field, value);
        self.push(KeyValue {
            key: field.to_owned(),
            value: v,
        })
    }

    fn record_i8(&mut self, field: &Key, value: i8) {
        let mut v = AnyValue::Empty;
        v.record_i8(field, value);
        self.push(KeyValue {
            key: field.to_owned(),
            value: v,
        })
    }

    fn record_i16(&mut self, field: &Key, value: i16) {
        let mut v = AnyValue::Empty;
        v.record_i16(field, value);
        self.push(KeyValue {
            key: field.to_owned(),
            value: v,
        })
    }

    fn record_i32(&mut self, field: &Key, value: i32) {
        let mut v = AnyValue::Empty;
        v.record_i32(field, value);
        self.push(KeyValue {
            key: field.to_owned(),
            value: v,
        })
    }

    fn record_i64(&mut self, field: &Key, value: i64) {
        let mut v = AnyValue::Empty;
        v.record_i64(field, value);
        self.push(KeyValue {
            key: field.to_owned(),
            value: v,
        })
    }

    fn record_i128(&mut self, field: &Key, value: i128) {
        let mut v = AnyValue::Empty;
        v.record_i128(field, value);
        self.push(KeyValue {
            key: field.to_owned(),
            value: v,
        })
    }

    fn record_u8(&mut self, field: &Key, value: u8) {
        let mut v = AnyValue::Empty;
        v.record_u8(field, value);
        self.push(KeyValue {
            key: field.to_owned(),
            value: v,
        })
    }

    fn record_u16(&mut self, field: &Key, value: u16) {
        let mut v = AnyValue::Empty;
        v.record_u16(field, value);
        self.push(KeyValue {
            key: field.to_owned(),
            value: v,
        })
    }

    fn record_u32(&mut self, field: &Key, value: u32) {
        let mut v = AnyValue::Empty;
        v.record_u32(field, value);
        self.push(KeyValue {
            key: field.to_owned(),
            value: v,
        })
    }

    fn record_u64(&mut self, field: &Key, value: u64) {
        let mut v = AnyValue::Empty;
        v.record_u64(field, value);
        self.push(KeyValue {
            key: field.to_owned(),
            value: v,
        })
    }

    fn record_u128(&mut self, field: &Key, value: u128) {
        let mut v = AnyValue::Empty;
        v.record_u128(field, value);
        self.push(KeyValue {
            key: field.to_owned(),
            value: v,
        })
    }

    fn record_f32(&mut self, field: &Key, value: f32) {
        let mut v = AnyValue::Empty;
        v.record_f32(field, value);
        self.push(KeyValue {
            key: field.to_owned(),
            value: v,
        })
    }

    fn record_f64(&mut self, field: &Key, value: f64) {
        let mut v = AnyValue::Empty;
        v.record_f64(field, value);
        self.push(KeyValue {
            key: field.to_owned(),
            value: v,
        })
    }

    fn record_str(&mut self, field: &Key, value: &str) {
        let mut v = AnyValue::Empty;
        v.record_str(field, value);
        self.push(KeyValue {
            key: field.to_owned(),
            value: v,
        })
    }

    fn record_byte_slice(&mut self, field: &Key, value: &[u8]) {
        let mut v = AnyValue::Empty;
        v.record_byte_slice(field, value);
        self.push(KeyValue {
            key: field.to_owned(),
            value: v,
        })
    }

    fn record_error(&mut self, field: &Key, value: &(dyn std::error::Error + 'static)) {
        let mut v = AnyValue::Empty;
        v.record_error(field, value);
        self.push(KeyValue {
            key: field.to_owned(),
            value: v,
        })
    }

    fn record_debug(&mut self, field: &Key, value: &dyn std::fmt::Debug) {
        let mut v = AnyValue::Empty;
        v.record_debug(field, value);
        self.push(KeyValue {
            key: field.to_owned(),
            value: v,
        })
    }
}
