use crate::{ast::ParsedStory, compiler::StageOutput};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedStory {
    pub parsed: ParsedStory,
}

pub(crate) fn analyze(parsed: ParsedStory) -> StageOutput<CheckedStory> {
    StageOutput {
        artifact: Some(CheckedStory { parsed }),
        diagnostics: Vec::new(),
    }
}
