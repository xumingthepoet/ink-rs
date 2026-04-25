mod constants;
mod flow;
mod names;
mod span;
mod targets;
mod variables;
mod warnings;

use crate::{compiler::StageOutput, parsed::Story};

use constants::constant_redefinition_diagnostics;
use flow::flow_diagnostics;
use names::naming_diagnostics;
use targets::call_target_diagnostics;
use warnings::author_warning_diagnostics;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedStory {
    pub parsed: Story,
}

pub(crate) fn analyze(parsed: Story) -> StageOutput<CheckedStory> {
    let mut diagnostics = constant_redefinition_diagnostics(&parsed);
    diagnostics.extend(author_warning_diagnostics(&parsed));
    diagnostics.extend(naming_diagnostics(&parsed));
    diagnostics.extend(flow_diagnostics(&parsed));
    diagnostics.extend(call_target_diagnostics(&parsed));
    StageOutput {
        artifact: Some(CheckedStory { parsed }),
        diagnostics,
    }
}
