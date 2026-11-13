//! Time related helpers
//! 
//! A lot of the formats serializes `SystemTime` and `Duration` as a nanoseconds
//! encoded in a `u64`. This module helps with dealing with this.
//! 

use std::ops::Add;
use std::time::{Duration, SystemTime, UNIX_EPOCH as SYS_UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// The `UNIX_EPOCH` encoded as a [`UnixTimeNano`]
pub const UNIX_EPOCH : UnixTimeNano = UnixTimeNano(0);

/// A unix time in nanoseconds since "1970-01-01 00:00:00 UTC".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct UnixTimeNano(u64);

impl UnixTimeNano {
    /// Returns the system time corresponding to "now".
    pub fn now() -> Self {
        SystemTime::now().into()
    }

    /// Returns the time corresponding to `UNIX_EPOCH + nanos`.
    pub fn from_nanos_since_epoch(nanos: u64) -> Self {
        UNIX_EPOCH + Duration::from_nanos(nanos)
    }

    /// Returns the nanoseconds since `UNIX_EPOCH` that this represents.
    pub fn as_nanos_since_epoch(&self) -> u64 {
        self.0
    }
}

impl From<SystemTime> for UnixTimeNano {
    fn from(value: SystemTime) -> Self {
        UnixTimeNano(value.duration_since(SYS_UNIX_EPOCH).unwrap_or_default().as_nanos() as u64)
    }
}

impl From<UnixTimeNano> for SystemTime {
    fn from(value: UnixTimeNano) -> Self {
        SYS_UNIX_EPOCH + Duration::from_nanos(value.0)
    }
}

impl Add<Duration> for UnixTimeNano {
    type Output = UnixTimeNano;

    fn add(self, rhs: Duration) -> Self::Output {
        UnixTimeNano(self.0 + (rhs.as_nanos() as u64))
    }
}

/// Encoding a `SystemTime` as nanoseconds since `UNIX_EPOCH` with `serde`.
pub mod system_time_serde {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Deserialize `SystemTime` from nanoseconds.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where D: Deserializer<'de>,
    {
        let time_nanos : u64 = u64::deserialize(deserializer)?;
        let time = Duration::from_nanos(time_nanos);
        Ok(UNIX_EPOCH + time)
    }

    /// Serialize `SystemTime` as nanoseconds.
    pub fn serialize<S>(value: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let time : super::UnixTimeNano = (*value).into();
        time.serialize(serializer)
    }
}

/// Encoding a `Option<SystemTime>` as nanoseconds since `UNIX_EPOCH` with `serde`.
/// This will (de-)serialize the value as a `Option<u64>`.
pub mod option_system_time_serde {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Deserialize `Option<SystemTime>` from nanoseconds.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<SystemTime>, D::Error>
    where D: Deserializer<'de>,
    {
        let Some(time_nanos) = Option::<u64>::deserialize(deserializer)? else {
            return Ok(None);
        };
        let time = Duration::from_nanos(time_nanos);
        Ok(Some(UNIX_EPOCH + time))
    }

    /// Serialize `Option<SystemTime>` as nanoseconds.
    pub fn serialize<S>(value: &Option<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let time : Option<super::UnixTimeNano> = (*value).map(From::from);
        time.serialize(serializer)
    }
}

/// Encoding a `Range<SystemTime>` as pair of nanoseconds since `UNIX_EPOCH` with `serde`.
/// This will (de-)serialize the value as a `Range<u64>`.
pub mod system_time_range_serde {
    use std::ops::Range;
    use std::time::SystemTime;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::UnixTimeNano;

    /// Deserialize `Range<SystemTime>` from pair of nanoseconds.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Range<SystemTime>, D::Error>
    where D: Deserializer<'de>,
    {
        let dur_nanos : Range<UnixTimeNano> = Range::<UnixTimeNano>::deserialize(deserializer)?;
        Ok(dur_nanos.start.into()..dur_nanos.end.into())
    }

    /// Serialize `Range<Duration>` as pair of nanoseconds.
    pub fn serialize<S>(value: &Range<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let start : UnixTimeNano = value.start.into();
        let end : UnixTimeNano = value.end.into();
        (start..end).serialize(serializer)
    }
}

/// Encoding a `Duration` as nanoseconds with `serde`.
pub mod duration_serde {
    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Deserialize `Duration` from nanoseconds.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where D: Deserializer<'de>,
    {
        let time_nanos : u64 = u64::deserialize(deserializer)?;
        Ok(Duration::from_nanos(time_nanos))
    }

    /// Serialize `Duration` as nanoseconds.
    pub fn serialize<S>(value: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let time = (*value).as_nanos() as u64;
        time.serialize(serializer)
    }
}

/// Encoding a `Range<Duration>` as pair of nanoseconds with `serde`.
/// This will (de-)serialize the value as a `Range<u64>`.
pub mod duration_range_serde {
    use std::ops::Range;
    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Deserialize `Range<Duration>` from pair of nanoseconds.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Range<Duration>, D::Error>
    where D: Deserializer<'de>,
    {
        let dur_nanos : Range<u64> = Range::<u64>::deserialize(deserializer)?;
        Ok(Duration::from_nanos(dur_nanos.start)..Duration::from_nanos(dur_nanos.end))
    }

    /// Serialize `Range<Duration>` as pair of nanoseconds.
    pub fn serialize<S>(value: &Range<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let start = value.start.as_nanos() as u64;
        let end = value.end.as_nanos() as u64;
        (start..end).serialize(serializer)
    }
}

#[doc(hidden)]
#[cfg(any(
    not(target_arch = "wasm32"),
    all(target_arch = "wasm32", target_os = "wasi")
))]
pub fn now() -> SystemTime {
    SystemTime::now()
}

#[doc(hidden)]
#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub fn now() -> SystemTime {
    SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(js_sys::Date::now() as u64)
}