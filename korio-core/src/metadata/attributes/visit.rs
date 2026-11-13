use std::fmt::{self, Write as _};

use crate::metadata::{Key, display};

/// Visitor for getting values of fields.
pub trait Visit {
    /// Visit a value implementing [`Valuable`].
    #[cfg(feature = "valuable")]
    fn record_value(&mut self, key: &Key, value: valuable::Value<'_>) {
        self.record_debug(key, &value);
    }

    /// Visit a boolean value.
    fn record_bool(&mut self, key: &Key, value: bool) {
        self.record_debug(key, &value);
    }

    /// Visit an i8 value.
    fn record_i8(&mut self, key: &Key, value: i8) {
        self.record_debug(key, &value);
    }

    /// Visit an i16 value.
    fn record_i16(&mut self, key: &Key, value: i16) {
        self.record_debug(key, &value);
    }

    /// Visit an i32 value.
    fn record_i32(&mut self, key: &Key, value: i32) {
        self.record_debug(key, &value);
    }

    /// Visit an i64 value.
    fn record_i64(&mut self, key: &Key, value: i64) {
        self.record_debug(key, &value);
    }

    /// Visit an i128 value.
    fn record_i128(&mut self, key: &Key, value: i128) {
        self.record_debug(key, &value);
    }

    /// Visit an u8 value.
    fn record_u8(&mut self, key: &Key, value: u8) {
        self.record_debug(key, &value);
    }

    /// Visit an u16 value.
    fn record_u16(&mut self, key: &Key, value: u16) {
        self.record_debug(key, &value);
    }

    /// Visit an u32 value.
    fn record_u32(&mut self, key: &Key, value: u32) {
        self.record_debug(key, &value);
    }

    /// Visit an u64 value.
    fn record_u64(&mut self, key: &Key, value: u64) {
        self.record_debug(key, &value);
    }

    /// Visit an u128 value.
    fn record_u128(&mut self, key: &Key, value: u128) {
        self.record_debug(key, &value);
    }

    /// Visit a f32 value.
    fn record_f32(&mut self, key: &Key, value: f32) {
        self.record_debug(key, &value);
    }

    /// Visit a f64 value.
    fn record_f64(&mut self, key: &Key, value: f64) {
        self.record_debug(key, &value);
    }

    /// Visit a string value.
    fn record_str(&mut self, key: &Key, value: &str) {
        self.record_debug(key, &value);
    }

    /// Visit a byte slice value.
    fn record_byte_slice(&mut self, key: &Key, value: &[u8]) {
        self.record_debug(key, &HexBytes(value));
    }

    /// Visit an error value.
    fn record_error(&mut self, key: &Key, value: &(dyn std::error::Error + 'static)) {
        self.record_debug(key, &display(value));
    }

    /// Visit a debug value.
    fn record_debug(&mut self, key: &Key, value: &dyn std::fmt::Debug);
}

struct HexBytes<'b>(&'b [u8]);
impl fmt::Debug for HexBytes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('[')?;

        let mut bytes = self.0.iter();

        if let Some(byte) = bytes.next() {
            f.write_fmt(format_args!("{byte:02x}"))?;
        }

        for byte in bytes {
            f.write_fmt(format_args!(" {byte:02x}"))?;
        }

        f.write_char(']')
    }
}
