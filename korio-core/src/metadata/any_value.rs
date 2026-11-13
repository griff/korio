use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::metadata::KeyBuf;

use super::{AttributeSet, Key, KeyValue, Value, Visit};

/// Any supported value.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum AnyValue {
    /// An empty value.
    Empty,
    /// A Rust `bool` value.
    Bool(bool),
    /// A Rust `u8` value.
    U8(u8),
    /// A Rust `u16` value.
    U16(u16),
    /// A Rust `u32` value.
    U32(u32),
    /// A Rust `u64` value.
    U64(u64),
    /// A Rust `u128` value.
    U128(u128),
    /// A Rust `i8` value.
    I8(i8),
    /// A Rust `i16` value.
    I16(i16),
    /// A Rust `i32` value.
    I32(i32),
    /// A Rust `i64` value.
    I64(i64),
    /// A Rust `i128` value.
    I128(i128),
    /// A Rust `f32` value.
    F32(f32),
    /// A Rust `f64` value.
    F64(f64),
    /// A Rust `Cow<'static, str>` value.
    String(Cow<'static, str>),
    /// A Rust `Cow<'static, [u8]>` value.
    Bytes(Cow<'static, [u8]>),
    /// A Rust array value.
    Array(Vec<AnyValue>),
    /// A Rust map value.
    Map(Vec<KeyValue>),
}

const DUMMY: &Key = Key::new("dummy");

impl AnyValue {
    /// Construct an `AnyValue` from provided `value`.
    pub fn from_value<V: Value>(value: V) -> Self {
        let mut ret = Self::Empty;
        value.record(DUMMY, &mut ret);
        ret
    }
}

impl From<String> for AnyValue {
    fn from(value: String) -> Self {
        Self::String(Cow::Owned(value))
    }
}

#[forbid(clippy::missing_trait_methods)]
impl Visit for AnyValue {
    fn record_bool(&mut self, _field: &Key, value: bool) {
        *self = AnyValue::Bool(value);
    }

    fn record_i8(&mut self, _field: &Key, value: i8) {
        *self = AnyValue::I8(value);
    }

    fn record_i16(&mut self, _field: &Key, value: i16) {
        *self = AnyValue::I16(value);
    }

    fn record_i32(&mut self, _field: &Key, value: i32) {
        *self = AnyValue::I32(value);
    }

    fn record_i64(&mut self, _field: &Key, value: i64) {
        *self = AnyValue::I64(value);
    }

    fn record_i128(&mut self, _field: &Key, value: i128) {
        *self = AnyValue::I128(value);
    }

    fn record_u8(&mut self, _field: &Key, value: u8) {
        *self = AnyValue::U8(value);
    }

    fn record_u16(&mut self, _field: &Key, value: u16) {
        *self = AnyValue::U16(value);
    }

    fn record_u32(&mut self, _field: &Key, value: u32) {
        *self = AnyValue::U32(value);
    }

    fn record_u64(&mut self, _field: &Key, value: u64) {
        *self = AnyValue::U64(value);
    }

    fn record_u128(&mut self, _field: &Key, value: u128) {
        *self = AnyValue::U128(value);
    }

    fn record_f32(&mut self, _field: &Key, value: f32) {
        *self = AnyValue::F32(value);
    }

    fn record_f64(&mut self, _field: &Key, value: f64) {
        *self = AnyValue::F64(value);
    }

    fn record_str(&mut self, _field: &Key, value: &str) {
        *self = AnyValue::String(Cow::Owned(value.to_string()));
    }

    fn record_byte_slice(&mut self, _field: &Key, value: &[u8]) {
        *self = AnyValue::Bytes(Cow::Owned(value.to_vec()));
    }

    fn record_error(&mut self, _field: &Key, value: &(dyn std::error::Error + 'static)) {
        *self = AnyValue::String(Cow::Owned(format!("{:?}", value)));
    }

    fn record_debug(&mut self, _field: &Key, value: &dyn std::fmt::Debug) {
        *self = AnyValue::String(Cow::Owned(format!("{:?}", value)));
    }
}

impl Value for AnyValue {
    fn record(&self, field: &Key, visitor: &mut dyn Visit) {
        match self {
            AnyValue::Empty => {}
            AnyValue::Bool(value) => visitor.record_bool(field, *value),
            AnyValue::U8(value) => visitor.record_u64(field, *value as u64),
            AnyValue::U16(value) => visitor.record_u64(field, *value as u64),
            AnyValue::U32(value) => visitor.record_u64(field, *value as u64),
            AnyValue::U64(value) => visitor.record_u64(field, *value),
            AnyValue::U128(value) => visitor.record_debug(field, value),
            AnyValue::I8(value) => visitor.record_i64(field, *value as i64),
            AnyValue::I16(value) => visitor.record_i64(field, *value as i64),
            AnyValue::I32(value) => visitor.record_i64(field, *value as i64),
            AnyValue::I64(value) => visitor.record_i64(field, *value),
            AnyValue::I128(value) => visitor.record_debug(field, value),
            AnyValue::F32(value) => visitor.record_f64(field, *value as f64),
            AnyValue::F64(value) => visitor.record_f64(field, *value),
            AnyValue::String(value) => visitor.record_str(field, value),
            AnyValue::Bytes(value) => visitor.record_byte_slice(field, value),
            AnyValue::Array(_value) => todo!(),
            AnyValue::Map(_value) => todo!(),
        }
    }
}

impl AttributeSet for BTreeMap<KeyBuf, AnyValue> {
    fn record(&self, visitor: &mut dyn Visit) {
        for (field, value) in self.iter() {
            value.record(field, visitor);
        }
    }

    fn len(&self) -> usize {
        BTreeMap::len(self)
    }

    fn is_empty(&self) -> bool {
        BTreeMap::is_empty(self)
    }
}

#[forbid(clippy::missing_trait_methods)]
impl Visit for BTreeMap<KeyBuf, AnyValue> {
    fn record_bool(&mut self, field: &Key, value: bool) {
        let mut any_value = AnyValue::Empty;
        any_value.record_bool(field, value);
        self.insert(field.to_owned(), any_value);
    }

    fn record_i8(&mut self, field: &Key, value: i8) {
        let mut any_value = AnyValue::Empty;
        any_value.record_i8(field, value);
        self.insert(field.to_owned(), any_value);
    }

    fn record_i16(&mut self, field: &Key, value: i16) {
        let mut any_value = AnyValue::Empty;
        any_value.record_i16(field, value);
        self.insert(field.to_owned(), any_value);
    }

    fn record_i32(&mut self, field: &Key, value: i32) {
        let mut any_value = AnyValue::Empty;
        any_value.record_i32(field, value);
        self.insert(field.to_owned(), any_value);
    }

    fn record_i64(&mut self, field: &Key, value: i64) {
        let mut any_value = AnyValue::Empty;
        any_value.record_i64(field, value);
        self.insert(field.to_owned(), any_value);
    }

    fn record_i128(&mut self, field: &Key, value: i128) {
        let mut any_value = AnyValue::Empty;
        any_value.record_i128(field, value);
        self.insert(field.to_owned(), any_value);
    }

    fn record_u8(&mut self, field: &Key, value: u8) {
        let mut any_value = AnyValue::Empty;
        any_value.record_u8(field, value);
        self.insert(field.to_owned(), any_value);
    }

    fn record_u16(&mut self, field: &Key, value: u16) {
        let mut any_value = AnyValue::Empty;
        any_value.record_u16(field, value);
        self.insert(field.to_owned(), any_value);
    }

    fn record_u32(&mut self, field: &Key, value: u32) {
        let mut any_value = AnyValue::Empty;
        any_value.record_u32(field, value);
        self.insert(field.to_owned(), any_value);
    }

    fn record_u64(&mut self, field: &Key, value: u64) {
        let mut any_value = AnyValue::Empty;
        any_value.record_u64(field, value);
        self.insert(field.to_owned(), any_value);
    }

    fn record_u128(&mut self, field: &Key, value: u128) {
        let mut any_value = AnyValue::Empty;
        any_value.record_u128(field, value);
        self.insert(field.to_owned(), any_value);
    }

    fn record_f32(&mut self, field: &Key, value: f32) {
        let mut any_value = AnyValue::Empty;
        any_value.record_f32(field, value);
        self.insert(field.to_owned(), any_value);
    }

    fn record_f64(&mut self, field: &Key, value: f64) {
        let mut any_value = AnyValue::Empty;
        any_value.record_f64(field, value);
        self.insert(field.to_owned(), any_value);
    }

    fn record_str(&mut self, field: &Key, value: &str) {
        let mut any_value = AnyValue::Empty;
        any_value.record_str(field, value);
        self.insert(field.to_owned(), any_value);
    }

    fn record_byte_slice(&mut self, field: &Key, value: &[u8]) {
        let mut any_value = AnyValue::Empty;
        any_value.record_byte_slice(field, value);
        self.insert(field.to_owned(), any_value);
    }

    fn record_error(&mut self, field: &Key, value: &(dyn std::error::Error + 'static)) {
        let mut any_value = AnyValue::Empty;
        any_value.record_error(field, value);
        self.insert(field.to_owned(), any_value);
    }

    fn record_debug(&mut self, field: &Key, value: &dyn std::fmt::Debug) {
        let mut any_value = AnyValue::Empty;
        any_value.record_debug(field, value);
        self.insert(field.to_owned(), any_value);
    }
}
