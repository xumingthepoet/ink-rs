use crate::{compiler::StageOutput, parsed::Story};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedStory {
    pub parsed: Story,
}

pub(crate) fn analyze(parsed: Story) -> StageOutput<CheckedStory> {
    StageOutput {
        artifact: Some(CheckedStory { parsed }),
        diagnostics: Vec::new(),
    }
}
