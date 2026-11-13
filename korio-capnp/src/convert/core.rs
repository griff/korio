use capnp_convert::{BuildFrom, ReadFrom, ReadInto, SetInto};
use korio::Severity;
use korio::metadata::{AnyValue, Key, KeyBuf, KeyValue};

use crate::capnp::common_capnp;

impl From<common_capnp::Severity> for Severity {
    fn from(value: common_capnp::Severity) -> Self {
        match value {
            common_capnp::Severity::Unspecified => Severity::Info,
            common_capnp::Severity::Trace => Severity::Trace,
            common_capnp::Severity::Trace2 => Severity::Trace2,
            common_capnp::Severity::Trace3 => Severity::Trace3,
            common_capnp::Severity::Trace4 => Severity::Trace4,
            common_capnp::Severity::Debug => Severity::Debug,
            common_capnp::Severity::Debug2 => Severity::Debug2,
            common_capnp::Severity::Debug3 => Severity::Debug3,
            common_capnp::Severity::Debug4 => Severity::Debug4,
            common_capnp::Severity::Info => Severity::Info,
            common_capnp::Severity::Info2 => Severity::Info2,
            common_capnp::Severity::Info3 => Severity::Info3,
            common_capnp::Severity::Info4 => Severity::Info4,
            common_capnp::Severity::Warn => Severity::Warn,
            common_capnp::Severity::Warn2 => Severity::Warn2,
            common_capnp::Severity::Warn3 => Severity::Warn3,
            common_capnp::Severity::Warn4 => Severity::Warn4,
            common_capnp::Severity::Error => Severity::Error,
            common_capnp::Severity::Error2 => Severity::Error2,
            common_capnp::Severity::Error3 => Severity::Error3,
            common_capnp::Severity::Error4 => Severity::Error4,
            common_capnp::Severity::Fatal => Severity::Fatal,
            common_capnp::Severity::Fatal2 => Severity::Fatal2,
            common_capnp::Severity::Fatal3 => Severity::Fatal3,
            common_capnp::Severity::Fatal4 => Severity::Fatal4,
        }
    }
}

impl From<Severity> for common_capnp::Severity {
    fn from(value: Severity) -> Self {
        match value {
            Severity::Unspecified => common_capnp::Severity::Unspecified,
            Severity::Trace => common_capnp::Severity::Trace,
            Severity::Trace2 => common_capnp::Severity::Trace2,
            Severity::Trace3 => common_capnp::Severity::Trace3,
            Severity::Trace4 => common_capnp::Severity::Trace4,
            Severity::Debug => common_capnp::Severity::Debug,
            Severity::Debug2 => common_capnp::Severity::Debug2,
            Severity::Debug3 => common_capnp::Severity::Debug3,
            Severity::Debug4 => common_capnp::Severity::Debug4,
            Severity::Info => common_capnp::Severity::Info,
            Severity::Info2 => common_capnp::Severity::Info2,
            Severity::Info3 => common_capnp::Severity::Info3,
            Severity::Info4 => common_capnp::Severity::Info4,
            Severity::Warn => common_capnp::Severity::Warn,
            Severity::Warn2 => common_capnp::Severity::Warn2,
            Severity::Warn3 => common_capnp::Severity::Warn3,
            Severity::Warn4 => common_capnp::Severity::Warn4,
            Severity::Error => common_capnp::Severity::Error,
            Severity::Error2 => common_capnp::Severity::Error2,
            Severity::Error3 => common_capnp::Severity::Error3,
            Severity::Error4 => common_capnp::Severity::Error4,
            Severity::Fatal => common_capnp::Severity::Fatal,
            Severity::Fatal2 => common_capnp::Severity::Fatal2,
            Severity::Fatal3 => common_capnp::Severity::Fatal3,
            Severity::Fatal4 => common_capnp::Severity::Fatal4,
        }
    }
}

impl SetInto<common_capnp::key_value::Builder<'_>> for (&Key, &AnyValue) {
    fn set_into(&self, builder: &mut common_capnp::key_value::Builder<'_>) -> capnp::Result<()> {
        builder.set_key(self.0.as_str());
        builder.reborrow().init_value().build_from(self.1)?;
        Ok(())
    }
}

impl SetInto<common_capnp::key_value::Builder<'_>> for (&KeyBuf, &AnyValue) {
    fn set_into(&self, builder: &mut common_capnp::key_value::Builder<'_>) -> capnp::Result<()> {
        let (k, v) = self;
        (k.as_ref(), *v).set_into(builder)
    }
}

impl SetInto<common_capnp::key_value::Builder<'_>> for KeyValue {
    fn set_into(&self, builder: &mut common_capnp::key_value::Builder<'_>) -> capnp::Result<()> {
        (&self.key, &self.value).set_into(builder)
    }
}

impl ReadFrom<common_capnp::key_value::Reader<'_>> for KeyValue {
    fn read_from(reader: common_capnp::key_value::Reader<'_>) -> Result<Self, capnp::Error> {
        let key = reader.get_key()?.to_string()?.into();
        let value = reader.get_value()?.read_into()?;
        Ok(KeyValue { key, value })
    }
}

impl SetInto<common_capnp::any_value::Builder<'_>> for AnyValue {
    fn set_into(&self, builder: &mut common_capnp::any_value::Builder<'_>) -> capnp::Result<()> {
        match self {
            AnyValue::Empty => builder.reborrow().set_empty(()),
            AnyValue::Bool(value) => builder.reborrow().set_bool_value(*value),
            AnyValue::U8(value) => builder.reborrow().set_uint8_value(*value),
            AnyValue::U16(value) => builder.reborrow().set_uint16_value(*value),
            AnyValue::U32(value) => builder.reborrow().set_uint32_value(*value),
            AnyValue::U64(value) => builder.reborrow().set_uint64_value(*value),
            AnyValue::U128(value) => builder.reborrow().init_uint128_value().build_from(value)?,
            AnyValue::I8(value) => builder.reborrow().set_int8_value(*value),
            AnyValue::I16(value) => builder.reborrow().set_int16_value(*value),
            AnyValue::I32(value) => builder.reborrow().set_int32_value(*value),
            AnyValue::I64(value) => builder.reborrow().set_int64_value(*value),
            AnyValue::I128(value) => builder.reborrow().init_int128_value().build_from(value)?,
            AnyValue::F32(value) => builder.reborrow().set_float32_value(*value),
            AnyValue::F64(value) => builder.reborrow().set_float64_value(*value),
            AnyValue::String(value) => builder.reborrow().set_string_value(value),
            AnyValue::Bytes(value) => builder.reborrow().set_bytes_value(&value[..]),
            AnyValue::Array(values) => {
                let mut b = builder.reborrow().init_array_value(values.len() as u32);
                for (index, value) in values.iter().enumerate() {
                    b.reborrow().get(index as u32).build_from(value)?;
                }
            }
            AnyValue::Map(values) => {
                let mut b = builder.reborrow().init_kvlist_value(values.len() as u32);
                for (index, value) in values.iter().enumerate() {
                    b.reborrow().get(index as u32).build_from(value)?;
                }
            }
        }
        Ok(())
    }
}

impl ReadFrom<common_capnp::any_value::Reader<'_>> for AnyValue {
    fn read_from(reader: common_capnp::any_value::Reader<'_>) -> Result<Self, capnp::Error> {
        match reader.which()? {
            common_capnp::any_value::Which::Empty(_) => Ok(AnyValue::Empty),
            common_capnp::any_value::Which::StringValue(value) => {
                Ok(AnyValue::String(value?.to_string()?.into()))
            }
            common_capnp::any_value::Which::BoolValue(value) => Ok(AnyValue::Bool(value)),
            common_capnp::any_value::Which::Int8Value(value) => Ok(AnyValue::I8(value)),
            common_capnp::any_value::Which::Int16Value(value) => Ok(AnyValue::I16(value)),
            common_capnp::any_value::Which::Int32Value(value) => Ok(AnyValue::I32(value)),
            common_capnp::any_value::Which::Int64Value(value) => Ok(AnyValue::I64(value)),
            common_capnp::any_value::Which::Int128Value(value) => {
                Ok(AnyValue::I128(value?.read_into()?))
            }
            common_capnp::any_value::Which::Uint8Value(value) => Ok(AnyValue::U8(value)),
            common_capnp::any_value::Which::Uint16Value(value) => Ok(AnyValue::U16(value)),
            common_capnp::any_value::Which::Uint32Value(value) => Ok(AnyValue::U32(value)),
            common_capnp::any_value::Which::Uint64Value(value) => Ok(AnyValue::U64(value)),
            common_capnp::any_value::Which::Uint128Value(value) => {
                Ok(AnyValue::U128(value?.read_into()?))
            }
            common_capnp::any_value::Which::Float32Value(value) => Ok(AnyValue::F32(value)),
            common_capnp::any_value::Which::Float64Value(value) => Ok(AnyValue::F64(value)),
            common_capnp::any_value::Which::ArrayValue(value) => {
                let values = value?
                    .iter()
                    .map(AnyValue::read_from)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(AnyValue::Array(values))
            }
            common_capnp::any_value::Which::KvlistValue(value) => {
                let values = value?
                    .iter()
                    .map(KeyValue::read_from)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(AnyValue::Map(values))
            }
            common_capnp::any_value::Which::BytesValue(value) => {
                Ok(AnyValue::Bytes(value?.to_vec().into()))
            }
        }
    }
}
