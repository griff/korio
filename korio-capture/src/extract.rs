use std::{fmt, marker::PhantomData};

use korio::metadata::{Key, KeyBuf, Visit};
use tracing::trace;

pub trait Extractor {
    type Value;

    /// Visit a boolean value.
    fn from_bool(field: &Key, value: bool) -> Option<Self::Value> {
        let _ = (field, value);
        None
    }

    /// Visit a signed 8-bit integer value.
    fn from_i8(field: &Key, value: i8) -> Option<Self::Value> {
        let _ = (field, value);
        None
    }

    /// Visit a signed 16-bit integer value.
    fn from_i16(field: &Key, value: i16) -> Option<Self::Value> {
        let _ = (field, value);
        None
    }

    /// Visit a signed 32-bit integer value.
    fn from_i32(field: &Key, value: i32) -> Option<Self::Value> {
        let _ = (field, value);
        None
    }

    /// Visit a signed 64-bit integer value.
    fn from_i64(field: &Key, value: i64) -> Option<Self::Value> {
        let _ = (field, value);
        None
    }

    /// Visit a signed 128-bit integer value.
    fn from_i128(field: &Key, value: i128) -> Option<Self::Value> {
        let _ = (field, value);
        None
    }

    /// Visit an unsigned 8-bit integer value.
    fn from_u8(field: &Key, value: u8) -> Option<Self::Value> {
        let _ = (field, value);
        None
    }

    /// Visit an unsigned 16-bit integer value.
    fn from_u16(field: &Key, value: u16) -> Option<Self::Value> {
        let _ = (field, value);
        None
    }

    /// Visit an unsigned 32-bit integer value.
    fn from_u32(field: &Key, value: u32) -> Option<Self::Value> {
        let _ = (field, value);
        None
    }

    /// Visit an unsigned 64-bit integer value.
    fn from_u64(field: &Key, value: u64) -> Option<Self::Value> {
        let _ = (field, value);
        None
    }

    /// Visit an unsigned 128-bit integer value.
    fn from_u128(field: &Key, value: u128) -> Option<Self::Value> {
        let _ = (field, value);
        None
    }

    /// Visit a f32 value.
    fn from_f32(field: &Key, value: f32) -> Option<Self::Value> {
        let _ = (field, value);
        None
    }

    /// Visit a double-precision floating point value.
    fn from_f64(field: &Key, value: f64) -> Option<Self::Value> {
        let _ = (field, value);
        None
    }

    fn from_str(field: &Key, value: &str) -> Option<Self::Value> {
        let _ = (field, value);
        None
    }

    fn from_byte_slice(field: &Key, value: &[u8]) -> Option<Self::Value> {
        let _ = (field, value);
        None
    }

    fn from_error(field: &Key, value: &(dyn std::error::Error + 'static)) -> Option<Self::Value> {
        let _ = (field, value);
        None
    }

    fn from_debug(field: &Key, value: &dyn fmt::Debug) -> Option<Self::Value> {
        let _ = (field, value);
        None
    }
}

pub struct KeyedExtractor<E> {
    key: KeyBuf,
    _extractor: PhantomData<E>,
}

impl<E> KeyedExtractor<E>
where
    E: Extractor,
{
    pub const fn new(key: KeyBuf) -> Self {
        Self {
            key,
            _extractor: PhantomData,
        }
    }

    pub const fn from_static(s: &'static str) -> Self {
        Self::new(KeyBuf::from_static(s))
    }
}

impl<E> fmt::Debug for KeyedExtractor<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyedExtractor")
            .field("key", &self.key)
            .field("_extractor", &self._extractor)
            .finish()
    }
}

impl<E> fmt::Display for KeyedExtractor<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.key.fmt(f)
    }
}

impl<E> Clone for KeyedExtractor<E> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            _extractor: self._extractor,
        }
    }
}

impl<E> PartialEq for KeyedExtractor<E> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<E> PartialEq<Key> for KeyedExtractor<E> {
    fn eq(&self, other: &Key) -> bool {
        self.key.as_key() == *other
    }
}

impl<E> PartialEq<KeyedExtractor<E>> for Key {
    fn eq(&self, other: &KeyedExtractor<E>) -> bool {
        self == other.key.as_key()
    }
}
impl<E> PartialEq<KeyedExtractor<E>> for &'_ Key {
    fn eq(&self, other: &KeyedExtractor<E>) -> bool {
        *self == other.key.as_key()
    }
}

impl<E> Eq for KeyedExtractor<E> {}

impl<E> PartialOrd for KeyedExtractor<E> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<E> Ord for KeyedExtractor<E> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl<E> AsRef<Key> for KeyedExtractor<E> {
    fn as_ref(&self) -> &Key {
        &self.key
    }
}

impl<E> From<KeyedExtractor<E>> for KeyBuf {
    fn from(value: KeyedExtractor<E>) -> Self {
        value.key
    }
}

pub struct Extract<E: Extractor> {
    _extractor: PhantomData<E>,
    value: Option<E::Value>,
}

impl<E> Extract<E>
where
    E: Extractor,
{
    pub fn new() -> Self {
        Self {
            value: None,
            _extractor: PhantomData,
        }
    }

    pub fn take(&mut self) -> Option<E::Value> {
        self.value.take()
    }
}

impl<E: Extractor> Default for Extract<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[forbid(clippy::missing_trait_methods)]
impl<E> Visit for Extract<E>
where
    E: Extractor,
{
    fn record_bool(&mut self, key: &Key, value: bool) {
        self.value = E::from_bool(key, value);
    }

    fn record_i8(&mut self, key: &Key, value: i8) {
        self.value = E::from_i8(key, value);
    }

    fn record_i16(&mut self, key: &Key, value: i16) {
        self.value = E::from_i16(key, value);
    }

    fn record_i32(&mut self, key: &Key, value: i32) {
        self.value = E::from_i32(key, value);
    }

    fn record_i64(&mut self, key: &Key, value: i64) {
        self.value = E::from_i64(key, value);
    }

    fn record_i128(&mut self, key: &Key, value: i128) {
        self.value = E::from_i128(key, value);
    }

    fn record_u8(&mut self, key: &Key, value: u8) {
        self.value = E::from_u8(key, value);
    }

    fn record_u16(&mut self, key: &Key, value: u16) {
        self.value = E::from_u16(key, value);
    }

    fn record_u32(&mut self, key: &Key, value: u32) {
        self.value = E::from_u32(key, value);
    }

    fn record_u64(&mut self, key: &Key, value: u64) {
        self.value = E::from_u64(key, value);
    }

    fn record_u128(&mut self, key: &Key, value: u128) {
        self.value = E::from_u128(key, value);
    }

    fn record_f32(&mut self, key: &Key, value: f32) {
        self.value = E::from_f32(key, value);
    }

    fn record_f64(&mut self, key: &Key, value: f64) {
        self.value = E::from_f64(key, value);
    }

    fn record_str(&mut self, key: &Key, value: &str) {
        self.value = E::from_str(key, value);
    }

    fn record_byte_slice(&mut self, key: &Key, value: &[u8]) {
        self.value = E::from_byte_slice(key, value);
    }

    fn record_error(&mut self, key: &Key, value: &(dyn std::error::Error + 'static)) {
        self.value = E::from_error(key, value);
    }

    fn record_debug(&mut self, key: &Key, value: &dyn fmt::Debug) {
        self.value = E::from_debug(key, value);
    }
}

impl Extractor for f32 {
    type Value = f32;

    fn from_f32(_field: &Key, value: f32) -> Option<Self::Value> {
        Some(value)
    }

    fn from_str(_field: &Key, value: &str) -> Option<Self::Value> {
        value.parse().ok()
    }
}

impl Extractor for f64 {
    type Value = f64;

    fn from_f32(_field: &Key, value: f32) -> Option<Self::Value> {
        Some(value.into())
    }

    fn from_f64(_field: &Key, value: f64) -> Option<Self::Value> {
        Some(value)
    }

    fn from_str(_field: &Key, value: &str) -> Option<Self::Value> {
        value.parse().ok()
    }
}

impl Extractor for bool {
    type Value = bool;

    fn from_bool(_field: &Key, value: bool) -> Option<Self::Value> {
        Some(value)
    }

    fn from_str(_field: &Key, value: &str) -> Option<Self::Value> {
        value.parse().ok()
    }
}
macro_rules! extract_number {
    ($t:ty) => {
        impl Extractor for $t {
            type Value = $t;

            fn from_i8(_field: &Key, value: i8) -> Option<Self::Value> {
                value.try_into().ok()
            }

            fn from_i16(_field: &Key, value: i16) -> Option<Self::Value> {
                value.try_into().ok()
            }

            fn from_i32(_field: &Key, value: i32) -> Option<Self::Value> {
                value.try_into().ok()
            }

            fn from_i64(_field: &Key, value: i64) -> Option<Self::Value> {
                value.try_into().ok()
            }

            fn from_i128(_field: &Key, value: i128) -> Option<Self::Value> {
                value.try_into().ok()
            }

            fn from_u8(_field: &Key, value: u8) -> Option<Self::Value> {
                value.try_into().ok()
            }

            fn from_u16(_field: &Key, value: u16) -> Option<Self::Value> {
                value.try_into().ok()
            }

            fn from_u32(_field: &Key, value: u32) -> Option<Self::Value> {
                value.try_into().ok()
            }

            fn from_u64(_field: &Key, value: u64) -> Option<Self::Value> {
                value.try_into().ok()
            }

            fn from_u128(_field: &Key, value: u128) -> Option<Self::Value> {
                value.try_into().ok()
            }

            fn from_str(_field: &Key, value: &str) -> Option<Self::Value> {
                value.parse().ok()
            }
        }
    };
}
extract_number!(i8);
extract_number!(i16);
extract_number!(i32);
extract_number!(i64);
extract_number!(i128);
extract_number!(u8);
extract_number!(u16);
extract_number!(u32);
extract_number!(u64);
extract_number!(u128);

impl Extractor for String {
    type Value = String;

    fn from_str(_field: &Key, value: &str) -> Option<Self::Value> {
        Some(value.to_string())
    }

    fn from_byte_slice(_field: &Key, value: &[u8]) -> Option<Self::Value> {
        std::str::from_utf8(value).ok().map(ToString::to_string)
    }

    fn from_error(_field: &Key, value: &(dyn std::error::Error + 'static)) -> Option<Self::Value> {
        Some(value.to_string())
    }

    fn from_debug(_field: &Key, value: &dyn fmt::Debug) -> Option<Self::Value> {
        Some(format!("{value:?}"))
    }
}
impl Extractor for Vec<u8> {
    type Value = Vec<u8>;

    fn from_str(_field: &Key, value: &str) -> Option<Self::Value> {
        Some(value.as_bytes().to_vec())
    }

    fn from_byte_slice(_field: &Key, value: &[u8]) -> Option<Self::Value> {
        Some(value.to_vec())
    }
}
pub struct Parse<T>(PhantomData<T>);
impl<T: std::str::FromStr> Extractor for Parse<T> {
    type Value = T;

    fn from_str(_field: &Key, value: &str) -> Option<Self::Value> {
        value.parse().ok()
    }

    fn from_byte_slice(field: &Key, value: &[u8]) -> Option<Self::Value> {
        let s = std::str::from_utf8(value).ok()?;
        Self::from_str(field, s)
    }

    fn from_error(field: &Key, value: &(dyn std::error::Error + 'static)) -> Option<Self::Value> {
        let s = value.to_string();
        Self::from_str(field, &s)
    }

    fn from_debug(field: &Key, value: &dyn fmt::Debug) -> Option<Self::Value> {
        let s = format!("{value:?}");
        Self::from_str(field, &s)
    }
}

pub struct Convert<F, T>(PhantomData<(F, T)>);

#[forbid(clippy::missing_trait_methods)]
impl<F, T> Extractor for Convert<F, T>
where
    F: Extractor,
    T: TryFrom<F::Value>,
{
    type Value = T;

    fn from_bool(field: &Key, value: bool) -> Option<Self::Value> {
        trace!(%field, value, "Convert::from_bool");
        F::from_bool(field, value).and_then(|v| v.try_into().ok())
    }

    fn from_i8(field: &Key, value: i8) -> Option<Self::Value> {
        trace!(%field, value, "Convert::from_i8");
        F::from_i8(field, value).and_then(|v| v.try_into().ok())
    }

    fn from_i16(field: &Key, value: i16) -> Option<Self::Value> {
        trace!(%field, value, "Convert::from_i16");
        F::from_i16(field, value).and_then(|v| v.try_into().ok())
    }

    fn from_i32(field: &Key, value: i32) -> Option<Self::Value> {
        trace!(%field, value, "Convert::from_i32");
        F::from_i32(field, value).and_then(|v| v.try_into().ok())
    }

    fn from_i64(field: &Key, value: i64) -> Option<Self::Value> {
        trace!(%field, value, "Convert::from_i64");
        F::from_i64(field, value).and_then(|v| v.try_into().ok())
    }

    fn from_i128(field: &Key, value: i128) -> Option<Self::Value> {
        trace!(%field, value, "Convert::from_i128");
        F::from_i128(field, value).and_then(|v| v.try_into().ok())
    }

    fn from_u8(field: &Key, value: u8) -> Option<Self::Value> {
        trace!(%field, value, "Convert::from_u8");
        F::from_u8(field, value).and_then(|v| v.try_into().ok())
    }

    fn from_u16(field: &Key, value: u16) -> Option<Self::Value> {
        trace!(%field, value, "Convert::from_u16");
        F::from_u16(field, value).and_then(|v| v.try_into().ok())
    }

    fn from_u32(field: &Key, value: u32) -> Option<Self::Value> {
        trace!(%field, value, "Convert::from_u32");
        F::from_u32(field, value).and_then(|v| v.try_into().ok())
    }

    fn from_u64(field: &Key, value: u64) -> Option<Self::Value> {
        trace!(%field, value, "Convert::from_u64");
        F::from_u64(field, value).and_then(|v| v.try_into().ok())
    }

    fn from_u128(field: &Key, value: u128) -> Option<Self::Value> {
        trace!(%field, value, "Convert::from_u128");
        F::from_u128(field, value).and_then(|v| v.try_into().ok())
    }

    fn from_f32(field: &Key, value: f32) -> Option<Self::Value> {
        trace!(%field, value, "Convert::from_f32");
        F::from_f32(field, value).and_then(|v| v.try_into().ok())
    }

    fn from_f64(field: &Key, value: f64) -> Option<Self::Value> {
        trace!(%field, value, "Convert::from_f64");
        F::from_f64(field, value).and_then(|v| v.try_into().ok())
    }

    fn from_str(field: &Key, value: &str) -> Option<Self::Value> {
        trace!(%field, value, "Convert::from_str");
        F::from_str(field, value).and_then(|v| v.try_into().ok())
    }

    fn from_byte_slice(field: &Key, value: &[u8]) -> Option<Self::Value> {
        trace!(%field, value, "Convert::from_byte_slice");
        F::from_byte_slice(field, value).and_then(|v| v.try_into().ok())
    }

    fn from_error(field: &Key, value: &(dyn std::error::Error + 'static)) -> Option<Self::Value> {
        trace!(%field, value, "Convert::from_error");
        F::from_error(field, value).and_then(|v| v.try_into().ok())
    }

    fn from_debug(field: &Key, value: &dyn fmt::Debug) -> Option<Self::Value> {
        trace!(%field, ?value, "Convert::from_debug");
        F::from_debug(field, value).and_then(|v| v.try_into().ok())
    }
}
