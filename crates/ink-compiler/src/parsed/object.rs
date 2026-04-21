#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugMetadata {
    pub source_name: Option<String>,
    pub start_line_number: usize,
    pub end_line_number: usize,
    pub start_character_number: usize,
    pub end_character_number: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object {
    pub debug_metadata: Option<DebugMetadata>,
}

impl Object {
    pub fn new() -> Self {
        Self {
            debug_metadata: None,
        }
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::new()
    }
}
