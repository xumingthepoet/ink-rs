use std::fmt;

use super::DebugMetadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub name: String,
    pub debug_metadata: Option<DebugMetadata>,
}

impl Identifier {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            debug_metadata: None,
        }
    }

    pub fn done() -> Self {
        Self::new("DONE")
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}
