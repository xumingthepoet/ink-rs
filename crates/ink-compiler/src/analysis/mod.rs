mod constants;
mod context;
mod flow;
mod names;
mod span;
mod targets;
#[cfg(test)]
mod test_support;
mod variables;
mod warnings;

use crate::{compiler::StageOutput, diagnostic::Diagnostic, parsed::Story};

use constants::constant_redefinition_diagnostics;
use flow::flow_diagnostics;
use names::naming_diagnostics;
use targets::call_target_diagnostics;
use warnings::author_warning_diagnostics;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedStory {
    // Keep analysis indexes pass-local until an index has stable semantics
    // outside diagnostics. Lowering currently builds runtime-path indexes that
    // are tied to JSON container layout rather than the analysis symbol model.
    pub parsed: Story,
}

pub(crate) fn analyze(parsed: Story) -> StageOutput<CheckedStory> {
    let diagnostics = run_analysis_passes(&parsed);
    StageOutput {
        artifact: Some(CheckedStory { parsed }),
        diagnostics,
    }
}

fn run_analysis_passes(story: &Story) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Constants and author warnings are story-wide discovery passes. They do
    // not depend on symbol or variable indexes.
    diagnostics.extend(constant_redefinition_diagnostics(story));
    diagnostics.extend(author_warning_diagnostics(story));

    // Naming must run before target checks so name collisions are reported
    // independently from downstream target/variable resolution.
    diagnostics.extend(naming_diagnostics(story));

    // Flow checks are order-sensitive and should stay before target checks:
    // loose ends, illegal returns, and function body restrictions describe
    // control-flow shape rather than target availability.
    diagnostics.extend(flow_diagnostics(story));

    // Target checks build symbol, variable-target, and variable-scope indexes.
    // Keep this after naming/flow diagnostics so resolution errors do not hide
    // more local structural problems.
    diagnostics.extend(call_target_diagnostics(story));

    diagnostics
}
