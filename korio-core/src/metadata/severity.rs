/// Severity of the event or span.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    num_enum::FromPrimitive,
    num_enum::IntoPrimitive,
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[repr(u16)]
#[allow(missing_docs)]
pub enum Severity {
    #[default]
    Unspecified = 0,
    Trace = 1,
    Trace2 = 2,
    Trace3 = 3,
    Trace4 = 4,
    Debug = 5,
    Debug2 = 6,
    Debug3 = 7,
    Debug4 = 8,
    Info = 9,
    Info2 = 10,
    Info3 = 11,
    Info4 = 12,
    Warn = 13,
    Warn2 = 14,
    Warn3 = 15,
    Warn4 = 16,
    Error = 17,
    Error2 = 18,
    Error3 = 19,
    Error4 = 20,
    Fatal = 21,
    Fatal2 = 22,
    Fatal3 = 23,
    Fatal4 = 24,
}

impl Severity {
    /// Return string representation of this severity
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Unspecified => "UNSPECIFIED",
            Severity::Trace => "TRACE",
            Severity::Trace2 => "TRACE2",
            Severity::Trace3 => "TRACE3",
            Severity::Trace4 => "TRACE4",
            Severity::Debug => "DEBUG",
            Severity::Debug2 => "DEBUG2",
            Severity::Debug3 => "DEBUG3",
            Severity::Debug4 => "DEBUG4",
            Severity::Info => "INFO",
            Severity::Info2 => "INFO2",
            Severity::Info3 => "INFO3",
            Severity::Info4 => "INFO4",
            Severity::Warn => "WARN",
            Severity::Warn2 => "WARN2",
            Severity::Warn3 => "WARN3",
            Severity::Warn4 => "WARN4",
            Severity::Error => "ERROR",
            Severity::Error2 => "ERROR2",
            Severity::Error3 => "ERROR3",
            Severity::Error4 => "ERROR4",
            Severity::Fatal => "FATAL",
            Severity::Fatal2 => "FATAL2",
            Severity::Fatal3 => "FATAL3",
            Severity::Fatal4 => "FATAL4",
        }
    }
}
