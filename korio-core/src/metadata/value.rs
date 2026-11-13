use std::borrow::Cow;
use std::fmt;
use std::num::Wrapping;

use auto_impl::auto_impl;

use super::{Key, Visit};

/// Value for [`Key`]
#[auto_impl(&, Box)]
pub trait Value {
    /// Record this value using the given visitor.
    fn record(&self, key: &Key, visitor: &mut dyn Visit);
}

/// Use the [`fmt::Display`] for the given value.
pub fn display<T: fmt::Display + ?Sized>(value: &T) -> &DisplayValue<T> {
    // SAFETY: `T` and `DisplayValue<T>` have the same ABI
    unsafe { &*(value as *const T as *const DisplayValue<T>) }
}

/// A value that that uses [`fmt::Display`].
#[derive(Clone)]
#[repr(transparent)]
pub struct DisplayValue<T: fmt::Display + ?Sized>(T);

impl<T: fmt::Display + ?Sized> fmt::Debug for DisplayValue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Display + ?Sized> fmt::Display for DisplayValue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Display + ?Sized> Value for DisplayValue<T> {
    fn record(&self, key: &Key, visitor: &mut dyn Visit) {
        visitor.record_debug(key, &self);
    }
}

/// Use the [`fmt::Debug`] for the given value.
pub fn debug<T: fmt::Debug>(value: T) -> DebugValue<T> {
    DebugValue(value)
}

/// A value that that uses [`fmt::Display`].
#[derive(Clone)]
#[repr(transparent)]
pub struct DebugValue<T: fmt::Debug>(T);

impl<T: fmt::Debug> fmt::Debug for DebugValue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Debug> Value for DebugValue<T> {
    fn record(&self, key: &Key, visitor: &mut dyn Visit) {
        visitor.record_debug(key, self);
    }
}

/// Wraps a type implementing [`Valuable`] as a `Value` that
/// can be recorded using its `Valuable` implementation.
///
/// [`Valuable`]: valuable::Valuable
#[cfg(feature = "valuable")]
pub fn valuable<T>(t: &T) -> valuable::Value<'_>
where
    T: valuable::Valuable,
{
    t.as_value()
}

impl Visit for fmt::DebugStruct<'_, '_> {
    fn record_debug(&mut self, key: &Key, value: &dyn std::fmt::Debug) {
        self.field(key.as_str(), value);
    }
}

impl Visit for fmt::DebugMap<'_, '_> {
    fn record_debug(&mut self, key: &Key, value: &dyn std::fmt::Debug) {
        self.entry(&DisplayValue(key), value);
    }
}

macro_rules! to_nonzero {
    (u8) => {
        std::num::NonZeroU8
    };
    (u16) => {
        std::num::NonZeroU16
    };
    (u32) => {
        std::num::NonZeroU32
    };
    (u64) => {
        std::num::NonZeroU64
    };
    (u128) => {
        std::num::NonZeroU128
    };
    (usize) => {
        std::num::NonZeroUsize
    };
    (i8) => {
        std::num::NonZeroI8
    };
    (i16) => {
        std::num::NonZeroI16
    };
    (i32) => {
        std::num::NonZeroI32
    };
    (i64) => {
        std::num::NonZeroI64
    };
    (i128) => {
        std::num::NonZeroI128
    };
    (isize) => {
        std::num::NonZeroIsize
    };
}

macro_rules! impl_one_value {
    ($type:ty, |$self:ident| $op:expr, $record:ident) => {
        impl Value for $type {
            fn record(&self, key: &Key, visitor: &mut dyn Visit) {
                let $self = self;
                let value = $op;
                visitor.$record(key, value);
            }
        }
    };
}

macro_rules! impl_value {
    ( #[nonzero] $record:ident, $( $type:tt ),+ ) => {
        $(
            impl_one_value!($type, |this| *this, $record);
            impl_one_value!(to_nonzero!($type), |this| this.get(), $record);
        )+
    };
    ( #[nonzero] $record:ident, $( $type:tt ),+ as $as_type:ty ) => {
        $(
            impl_one_value!($type, |this| *this as $as_type, $record);
            impl_one_value!(to_nonzero!($type), |this| this.get() as $as_type, $record);
        )+
    };
    ( $record:ident, $( $type:ty ),+ ) => {
        $( impl_one_value!($type, |this| *this, $record); )+
    };
    ( $record:ident, $( $type:ty ),+ as ref ) => {
        $( impl_one_value!($type, |this| this, $record); )+
    };
    ( $record:ident, $( $type:ty ),+ as $as_type:ty ) => {
        $( impl_one_value!($type, |this| *this as $as_type, $record); )+
    };
}

macro_rules! impl_values {
    ( $( $record:ident( $($types:tt)+ ) ),+ $(,)? ) => {
        $(
            impl_value!($record, $($types)+ );
        )+
    };
}

macro_rules! impl_nonzero_values {
    ( $( $record:ident( $($types:tt)+ ) ),+ $(,)? ) => {
        $(
            impl_value!(#[nonzero] $record, $($types)+ );
        )+
    };
}

impl_nonzero_values! {
    record_u8(u8),
    record_u16(u16),
    record_u32(u32),
    record_u64(u64),
    record_u64(usize as u64),
    record_u128(u128),
    record_i8(i8),
    record_i16(i16),
    record_i32(i32),
    record_i64(i64),
    record_i64(isize as i64),
    record_i128(i128),
}

impl_values! {
    record_bool(bool),
    record_f32(f32),
    record_f64(f64),
    record_byte_slice([u8], Vec<u8>, Cow<'_, [u8]> as ref),
    record_str(str, String, Cow<'_, str> as ref),
    record_debug(fmt::Arguments<'_> as ref),
}

#[cfg(feature = "valuable")]
impl Value for valuable::Value<'_> {
    fn record(&self, key: &Key, visitor: &mut dyn Visit) {
        visitor.record_value(key, *self)
    }
}

#[cfg(feature = "valuable")]
impl Value for &'_ dyn valuable::Valuable {
    fn record(&self, key: &Key, visitor: &mut dyn Visit) {
        visitor.record_value(key, self.as_value())
    }
}

impl<T: Value> Value for Wrapping<T> {
    fn record(&self, key: &Key, visitor: &mut dyn Visit) {
        self.0.record(key, visitor);
    }
}

impl<T: Value> Value for Option<T> {
    fn record(&self, key: &Key, visitor: &mut dyn Visit) {
        if let Some(v) = &self {
            v.record(key, visitor)
        }
    }
}
