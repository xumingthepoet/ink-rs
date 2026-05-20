mod array_literals;
mod assignments;
mod constants;
mod context;
mod dict_literals;
mod enums;
mod expression_types;
mod field_access;
mod flow;
mod index_access;
mod indexes;
mod initializers;
mod interface_values;
mod interfaces;
mod modules;
mod names;
mod span;
mod struct_literals;
mod structs;
mod target_symbols;
mod targets;
#[cfg(test)]
mod test_support;
mod type_names;
mod variables;
mod warnings;

use crate::{compiler::StageOutput, diagnostic::Diagnostic, parsed::Story};

use array_literals::array_literal_diagnostics_with_indexes;
use assignments::variable_assignment_diagnostics_with_indexes;
use constants::constant_redefinition_diagnostics;
use dict_literals::dict_literal_diagnostics_with_indexes;
use enums::enum_type_diagnostics;
use field_access::field_access_diagnostics_with_indexes;
use flow::flow_diagnostics_with_indexes;
use index_access::index_access_diagnostics_with_indexes;
use initializers::variable_initializer_diagnostics_with_indexes;
use interfaces::{interface_diagnostics, interface_implementation_diagnostics};
use names::naming_diagnostics;
use struct_literals::struct_literal_diagnostics_with_indexes;
use structs::struct_type_diagnostics;
use targets::call_target_diagnostics_with_indexes;
use warnings::author_warning_diagnostics;

pub use modules::{
    ModuleDependencyGraph, ModuleImportIndex, ModuleReachability, ModuleSymbolIndex,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedStory {
    // Keep analysis indexes pass-local until an index has stable semantics
    // outside diagnostics. Lowering currently builds runtime-path indexes that
    // are tied to JSON container layout rather than the analysis symbol model.
    pub parsed: Story,
    pub entry_point: Option<ModuleEntryPoint>,
    pub module_symbols: ModuleSymbolIndex,
    pub module_dependencies: ModuleDependencyGraph,
    pub module_imports: ModuleImportIndex,
    pub module_reachability: ModuleReachability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleEntryPoint {
    pub module: String,
    pub knot: String,
}

impl ModuleEntryPoint {
    pub fn new(module: impl Into<String>, knot: impl Into<String>) -> Self {
        Self {
            module: module.into(),
            knot: knot.into(),
        }
    }

    pub fn qualified_name(&self) -> String {
        format!("{}::{}", self.module, self.knot)
    }
}

pub(crate) fn analyze(parsed: Story) -> StageOutput<CheckedStory> {
    let module_analysis = modules::ModuleAnalysis::build(&parsed);
    let diagnostics = run_analysis_passes_with_modules(&parsed, &module_analysis);
    let (entry_point, module_symbols, module_dependencies, module_imports, module_reachability) =
        module_analysis.into_checked_parts();
    StageOutput {
        artifact: Some(CheckedStory {
            parsed,
            entry_point,
            module_symbols,
            module_dependencies,
            module_imports,
            module_reachability,
        }),
        diagnostics,
    }
}

#[cfg(test)]
fn run_analysis_passes(story: &Story) -> Vec<Diagnostic> {
    let module_analysis = modules::ModuleAnalysis::build(story);
    run_analysis_passes_with_modules(story, &module_analysis)
}

fn run_analysis_passes_with_modules(
    story: &Story,
    module_analysis: &modules::ModuleAnalysis,
) -> Vec<Diagnostic> {
    let analysis_indexes = indexes::AnalysisIndexes::build(story, module_analysis);
    analysis_indexes.mark_ready_for_incremental_migration();
    let mut diagnostics = Vec::new();

    // Constants and author warnings are story-wide discovery passes. They do
    // not depend on symbol or variable indexes.
    diagnostics.extend(constant_redefinition_diagnostics(story));
    diagnostics.extend(author_warning_diagnostics(story));
    diagnostics.extend(enum_type_diagnostics(story));
    diagnostics.extend(struct_type_diagnostics(story));
    diagnostics.extend(interface_diagnostics(story));
    diagnostics.extend(modules::module_symbol_diagnostics(
        story,
        &module_analysis.symbols,
    ));
    diagnostics.extend(interface_implementation_diagnostics(
        story,
        &module_analysis.symbols,
    ));
    diagnostics.extend(modules::mixed_root_module_diagnostics(story));
    diagnostics.extend(modules::module_entry_point_diagnostics(
        story,
        &module_analysis.entry_points,
    ));
    diagnostics.extend(modules::module_dependency_diagnostics(
        &module_analysis.dependencies,
    ));
    diagnostics.extend(modules::module_import_diagnostics(
        story,
        &module_analysis.symbols,
        &module_analysis.imports,
    ));
    diagnostics.extend(modules::unreachable_module_diagnostics(
        story,
        &module_analysis.reachability,
    ));

    // Naming must run before target checks so name collisions are reported
    // independently from downstream target/variable resolution.
    diagnostics.extend(naming_diagnostics(story));

    // Flow checks are order-sensitive and should stay before target checks:
    // loose ends, illegal returns, and function body restrictions describe
    // control-flow shape rather than target availability.
    diagnostics.extend(flow_diagnostics_with_indexes(story, &analysis_indexes));

    // Target checks build symbol and variable-scope indexes.
    // Keep this after naming/flow diagnostics so resolution errors do not hide
    // more local structural problems.
    diagnostics.extend(call_target_diagnostics_with_indexes(
        story,
        &analysis_indexes,
    ));

    diagnostics.extend(variable_initializer_diagnostics_with_indexes(
        story,
        &analysis_indexes,
    ));
    diagnostics.extend(variable_assignment_diagnostics_with_indexes(
        story,
        &analysis_indexes,
    ));
    diagnostics.extend(dict_literal_diagnostics_with_indexes(
        story,
        &analysis_indexes,
    ));
    diagnostics.extend(struct_literal_diagnostics_with_indexes(
        story,
        &analysis_indexes,
    ));
    diagnostics.extend(array_literal_diagnostics_with_indexes(
        story,
        &analysis_indexes,
    ));
    diagnostics.extend(field_access_diagnostics_with_indexes(
        story,
        &analysis_indexes,
    ));
    diagnostics.extend(index_access_diagnostics_with_indexes(
        story,
        &analysis_indexes,
    ));

    diagnostics
}

#[cfg(test)]
mod tests {
    const ANALYSIS_SOURCES: &[(&str, &str)] = &[
        ("mod.rs", include_str!("mod.rs")),
        ("array_literals.rs", include_str!("array_literals.rs")),
        ("assignments.rs", include_str!("assignments.rs")),
        ("constants.rs", include_str!("constants.rs")),
        ("context.rs", include_str!("context.rs")),
        ("dict_literals.rs", include_str!("dict_literals.rs")),
        ("enums.rs", include_str!("enums.rs")),
        ("expression_types.rs", include_str!("expression_types.rs")),
        ("field_access.rs", include_str!("field_access.rs")),
        ("flow.rs", include_str!("flow.rs")),
        ("index_access.rs", include_str!("index_access.rs")),
        ("indexes.rs", include_str!("indexes.rs")),
        ("initializers.rs", include_str!("initializers.rs")),
        ("interface_values.rs", include_str!("interface_values.rs")),
        ("interfaces.rs", include_str!("interfaces.rs")),
        ("names.rs", include_str!("names.rs")),
        ("span.rs", include_str!("span.rs")),
        ("struct_literals.rs", include_str!("struct_literals.rs")),
        ("structs.rs", include_str!("structs.rs")),
        ("target_symbols.rs", include_str!("target_symbols.rs")),
        ("targets.rs", include_str!("targets.rs")),
        ("test_support.rs", include_str!("test_support.rs")),
        ("variables.rs", include_str!("variables.rs")),
        ("warnings.rs", include_str!("warnings.rs")),
    ];

    #[test]
    fn analysis_sources_do_not_import_lower_or_emit_modules() {
        let forbidden_fragments = [
            concat!("crate", "::", "lower"),
            concat!("crate", "::", "emit"),
            concat!("lower", "::"),
            concat!("emit", "::"),
            concat!("lower", ","),
            concat!("emit", ","),
            concat!("lower", "}"),
            concat!("emit", "}"),
        ];

        for (path, source) in ANALYSIS_SOURCES {
            for (line_index, line) in source.lines().enumerate() {
                let code = line.split("//").next().unwrap_or_default();
                let compact = code
                    .chars()
                    .filter(|ch| !ch.is_whitespace())
                    .collect::<String>();
                for fragment in forbidden_fragments {
                    assert!(
                        !compact.contains(fragment),
                        "analysis source {path}:{} must not import lower/emit module APIs",
                        line_index + 1
                    );
                }
            }
        }
    }
}
