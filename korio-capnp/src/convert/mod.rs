mod core;
mod data;

use std::num::NonZeroU128;

use byteorder::{ByteOrder as _, LittleEndian, NetworkEndian};
use capnp_convert::{ReadFrom, SetInto};
use korio::span::TraceId;

use crate::capnp::common_capnp;

impl SetInto<common_capnp::trace_id::Builder<'_>> for TraceId {
    fn set_into(&self, builder: &mut common_capnp::trace_id::Builder<'_>) -> capnp::Result<()> {
        let mut ul = [0u64; 2];
        NetworkEndian::read_u64_into(&self.into_u128().to_ne_bytes(), &mut ul);
        builder.set_upper64(ul[0]);
        builder.set_lower64(ul[1]);
        Ok(())
    }
}

impl ReadFrom<common_capnp::trace_id::Reader<'_>> for TraceId {
    fn read_from(reader: common_capnp::trace_id::Reader<'_>) -> Result<Self, capnp::Error> {
        let mut ul = [0u64; 2];
        ul[0] = reader.get_upper64();
        ul[1] = reader.get_lower64();
        let mut octets = [0u8; 16];
        NetworkEndian::write_u64_into(&ul, &mut octets);
        let value = NonZeroU128::new(u128::from_ne_bytes(octets))
            .ok_or(capnp::Error::failed("zero trace id".into()))?;
        Ok(TraceId::from_non_zero_u128(value))
    }
}

impl ReadFrom<common_capnp::trace_id::Reader<'_>> for Option<TraceId> {
    fn read_from(reader: common_capnp::trace_id::Reader<'_>) -> Result<Self, capnp::Error> {
        let mut ul = [0u64; 2];
        ul[0] = reader.get_upper64();
        ul[1] = reader.get_lower64();
        let mut octets = [0u8; 16];
        NetworkEndian::write_u64_into(&ul, &mut octets);
        if let Some(id) = NonZeroU128::new(u128::from_ne_bytes(octets)) {
            Ok(Some(TraceId::from_non_zero_u128(id)))
        } else {
            Ok(None)
        }
    }
}

impl SetInto<common_capnp::int128::Builder<'_>> for i128 {
    fn set_into(&self, builder: &mut common_capnp::int128::Builder<'_>) -> capnp::Result<()> {
        let mut ul = [0u64; 2];
        LittleEndian::read_u64_into(&self.to_le_bytes(), &mut ul);
        builder.set_upper64(ul[0]);
        builder.set_lower64(ul[1]);
        Ok(())
    }
}

impl ReadFrom<common_capnp::int128::Reader<'_>> for i128 {
    fn read_from(reader: common_capnp::int128::Reader<'_>) -> Result<Self, capnp::Error> {
        let mut ul = [0u64; 2];
        ul[0] = reader.get_upper64();
        ul[1] = reader.get_lower64();
        let mut octets = [0u8; 16];
        LittleEndian::write_u64_into(&ul, &mut octets);
        Ok(i128::from_le_bytes(octets))
    }
}

impl SetInto<common_capnp::u_int128::Builder<'_>> for u128 {
    fn set_into(&self, builder: &mut common_capnp::u_int128::Builder<'_>) -> capnp::Result<()> {
        let mut ul = [0u64; 2];
        LittleEndian::read_u64_into(&self.to_le_bytes(), &mut ul);
        builder.set_upper64(ul[0]);
        builder.set_lower64(ul[1]);
        Ok(())
    }
}

impl ReadFrom<common_capnp::u_int128::Reader<'_>> for u128 {
    fn read_from(reader: common_capnp::u_int128::Reader<'_>) -> Result<Self, capnp::Error> {
        let mut ul = [0u64; 2];
        ul[0] = reader.get_upper64();
        ul[1] = reader.get_lower64();
        let mut octets = [0u8; 16];
        LittleEndian::write_u64_into(&ul, &mut octets);
        Ok(u128::from_le_bytes(octets))
    }
}
