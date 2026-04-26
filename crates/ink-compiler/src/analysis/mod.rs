mod array_literals;
mod assignments;
mod constants;
mod context;
mod expression_types;
mod field_access;
mod flow;
mod index_access;
mod initializers;
mod names;
mod span;
mod struct_literals;
mod structs;
mod target_symbols;
mod targets;
#[cfg(test)]
mod test_support;
mod variable_targets;
mod variables;
mod warnings;

use crate::{compiler::StageOutput, diagnostic::Diagnostic, parsed::Story};

use array_literals::array_literal_diagnostics;
use assignments::variable_assignment_diagnostics;
use constants::constant_redefinition_diagnostics;
use field_access::field_access_diagnostics;
use flow::flow_diagnostics;
use index_access::index_access_diagnostics;
use initializers::variable_initializer_diagnostics;
use names::naming_diagnostics;
use struct_literals::struct_literal_diagnostics;
use structs::struct_type_diagnostics;
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
    diagnostics.extend(struct_type_diagnostics(story));

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

    diagnostics.extend(variable_initializer_diagnostics(story));
    diagnostics.extend(variable_assignment_diagnostics(story));
    diagnostics.extend(struct_literal_diagnostics(story));
    diagnostics.extend(array_literal_diagnostics(story));
    diagnostics.extend(field_access_diagnostics(story));
    diagnostics.extend(index_access_diagnostics(story));

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
        ("expression_types.rs", include_str!("expression_types.rs")),
        ("field_access.rs", include_str!("field_access.rs")),
        ("flow.rs", include_str!("flow.rs")),
        ("index_access.rs", include_str!("index_access.rs")),
        ("initializers.rs", include_str!("initializers.rs")),
        ("names.rs", include_str!("names.rs")),
        ("span.rs", include_str!("span.rs")),
        ("struct_literals.rs", include_str!("struct_literals.rs")),
        ("structs.rs", include_str!("structs.rs")),
        ("target_symbols.rs", include_str!("target_symbols.rs")),
        ("targets.rs", include_str!("targets.rs")),
        ("test_support.rs", include_str!("test_support.rs")),
        ("variable_targets.rs", include_str!("variable_targets.rs")),
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
