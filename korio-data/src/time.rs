use std::ops::Add;
use std::time::{Duration, SystemTime, UNIX_EPOCH as SYS_UNIX_EPOCH};

pub const UNIX_EPOCH: UnixTimeNano = UnixTimeNano(DurationNano::ZERO);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct UnixTimeNano(DurationNano);

impl UnixTimeNano {
    pub fn now() -> Self {
        SystemTime::now().into()
    }

    pub fn from_nanos_since_epoch(nanos: u64) -> Self {
        UNIX_EPOCH + DurationNano::from_nanos(nanos)
    }

    pub fn as_nanos_since_epoch(&self) -> u64 {
        self.0.as_nanos()
    }
}

impl From<SystemTime> for UnixTimeNano {
    fn from(value: SystemTime) -> Self {
        UnixTimeNano(
            value
                .duration_since(SYS_UNIX_EPOCH)
                .unwrap_or_default()
                .into(),
        )
    }
}

impl From<UnixTimeNano> for SystemTime {
    fn from(value: UnixTimeNano) -> Self {
        SYS_UNIX_EPOCH + value.0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DurationNano(u64);

impl DurationNano {
    pub const ZERO: DurationNano = DurationNano(0);

    pub fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    pub fn as_nanos(&self) -> u64 {
        self.0
    }
}

impl From<Duration> for DurationNano {
    fn from(value: Duration) -> Self {
        let nanos: u64 = value.as_nanos().try_into().unwrap_or(u64::MAX);
        DurationNano(nanos)
    }
}

impl From<DurationNano> for Duration {
    fn from(value: DurationNano) -> Self {
        Duration::from_nanos(value.0)
    }
}

impl Add for DurationNano {
    type Output = DurationNano;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<Duration> for DurationNano {
    type Output = DurationNano;

    fn add(self, rhs: Duration) -> Self::Output {
        Self(
            (self.0 as u128 + rhs.as_nanos())
                .try_into()
                .unwrap_or(u64::MAX),
        )
    }
}
impl Add<DurationNano> for SystemTime {
    type Output = SystemTime;

    fn add(self, rhs: DurationNano) -> Self::Output {
        self + Duration::from(rhs)
    }
}

impl Add<DurationNano> for UnixTimeNano {
    type Output = UnixTimeNano;

    fn add(self, rhs: DurationNano) -> Self::Output {
        UnixTimeNano(self.0 + rhs)
    }
}

impl Add<Duration> for UnixTimeNano {
    type Output = UnixTimeNano;

    fn add(self, rhs: Duration) -> Self::Output {
        UnixTimeNano(self.0 + rhs)
    }
}

impl PartialEq<Duration> for DurationNano {
    fn eq(&self, other: &Duration) -> bool {
        self.0 as u128 == other.as_nanos()
    }
}

impl PartialOrd<Duration> for DurationNano {
    fn partial_cmp(&self, other: &Duration) -> Option<std::cmp::Ordering> {
        (self.0 as u128).partial_cmp(&other.as_nanos())
    }
}

#[cfg(feature = "serde")]
pub mod system_time_serde {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let time_nanos: u64 = u64::deserialize(deserializer)?;
        let time = Duration::from_nanos(time_nanos);
        Ok(UNIX_EPOCH + time)
    }

    pub fn serialize<S>(value: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let time: super::UnixTimeNano = (*value).into();
        time.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
pub mod option_system_time_serde {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<SystemTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Some(time_nanos) = Option::<u64>::deserialize(deserializer)? else {
            return Ok(None);
        };
        let time = Duration::from_nanos(time_nanos);
        Ok(Some(UNIX_EPOCH + time))
    }

    pub fn serialize<S>(value: &Option<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let time: Option<super::UnixTimeNano> = (*value).map(From::from);
        time.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
pub mod system_time_range_serde {
    use std::ops::Range;
    use std::time::SystemTime;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::UnixTimeNano;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Range<SystemTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let dur_nanos: Range<UnixTimeNano> = Range::<UnixTimeNano>::deserialize(deserializer)?;
        Ok(dur_nanos.start.into()..dur_nanos.end.into())
    }

    pub fn serialize<S>(value: &Range<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let start: UnixTimeNano = value.start.into();
        let end: UnixTimeNano = value.end.into();
        (start..end).serialize(serializer)
    }
}

#[cfg(feature = "serde")]
pub mod duration_serde {
    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::DurationNano;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let time_nanos: u64 = u64::deserialize(deserializer)?;
        Ok(Duration::from_nanos(time_nanos))
    }

    pub fn serialize<S>(value: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let time: DurationNano = (*value).into();
        time.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
pub mod duration_range_serde {
    use std::ops::Range;
    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::DurationNano;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Range<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let dur_nanos: Range<DurationNano> = Range::<DurationNano>::deserialize(deserializer)?;
        Ok(dur_nanos.start.into()..dur_nanos.end.into())
    }

    pub fn serialize<S>(value: &Range<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let start: DurationNano = value.start.into();
        let end: DurationNano = value.end.into();
        (start..end).serialize(serializer)
    }
}
