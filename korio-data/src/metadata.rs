use korio_core::metadata::{Metadata, Severity};

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OwnedMetadata {
    /// Target of the span or event.
    pub target: String,

    /// Name of the span or event.
    pub name: String,

    /// The original severity string from the source
    pub severity_text: String,

    /// The corresponding severity value, normalized
    pub severity_number: Severity,
}

impl OwnedMetadata {
    pub fn from_data(data: &dyn Metadata) -> Self {
        let target = data.target().to_string();
        let name = data.name().to_string();
        let severity_text = data.severity_text().to_string();
        let severity_number = data.severity_number();
        Self {
            target,
            name,
            severity_text,
            severity_number,
        }
    }
}

impl Metadata for OwnedMetadata {
    fn name(&self) -> &str {
        &self.name
    }

    fn target(&self) -> &str {
        &self.target
    }

    fn severity_number(&self) -> Severity {
        self.severity_number
    }

    fn severity_text(&self) -> &str {
        &self.severity_text
    }
}
