use crate::{
    diagnostic::Diagnostic,
    parsed::{visit::walk_story, Story},
};

use super::indexes::AnalysisIndexes;

#[cfg(test)]
use super::modules::ModuleAnalysis;

mod builtins;
mod calls;
mod checker;
mod diverts;

use checker::CallTargetChecker;

#[cfg(test)]
pub(super) fn call_target_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let module_analysis = ModuleAnalysis::build(story);
    let indexes = AnalysisIndexes::build(story, &module_analysis);
    call_target_diagnostics_with_indexes(story, &indexes)
}

pub(super) fn call_target_diagnostics_with_indexes(
    story: &Story,
    indexes: &AnalysisIndexes<'_>,
) -> Vec<Diagnostic> {
    let mut checker = CallTargetChecker::new(
        &indexes.target_symbols,
        &indexes.variable_scopes,
        &indexes.struct_types,
        &indexes.enum_types,
        &indexes.interface_members,
        &indexes.module_implementations,
        indexes.module_imports,
        &indexes.interface_module_literal_uses,
    );
    walk_story(story, &mut checker);
    checker.diagnostics
}
