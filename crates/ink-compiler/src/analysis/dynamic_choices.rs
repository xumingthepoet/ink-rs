use std::collections::HashSet;

use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_story, ParsedVisitor, VisitContext},
        Choice, Object, Story,
    },
};

use super::{expression_types::infer_expression_type, indexes::AnalysisIndexes};

pub(super) fn dynamic_choice_diagnostics_with_indexes(
    story: &Story,
    indexes: &AnalysisIndexes<'_>,
) -> Vec<Diagnostic> {
    let mut checker = DynamicChoiceChecker {
        indexes,
        diagnostics: Vec::new(),
    };
    walk_story(story, &mut checker);
    checker.diagnostics
}

struct DynamicChoiceChecker<'a> {
    indexes: &'a AnalysisIndexes<'a>,
    diagnostics: Vec<Diagnostic>,
}

impl ParsedVisitor for DynamicChoiceChecker<'_> {
    fn visit_object(&mut self, object: &Object, context: &VisitContext) {
        let Object::Choice(choice) = object else {
            return;
        };
        self.check_choice(choice, context);
    }
}

impl DynamicChoiceChecker<'_> {
    fn check_choice(&mut self, choice: &Choice, context: &VisitContext) {
        let Some(binding) = choice.dynamic_binding() else {
            return;
        };

        if !(1..=2).contains(&binding.variables().len()) {
            self.diagnostics.push(Diagnostic::error(
                choice.span().clone(),
                format!(
                    "Dynamic choice binding expects one item variable or `index, item` variables but got {}",
                    binding.variables().len()
                ),
            ));
        }

        let mut seen = HashSet::new();
        if binding
            .variables()
            .iter()
            .any(|variable| !seen.insert(variable.source_name()))
        {
            self.diagnostics.push(Diagnostic::error(
                choice.span().clone(),
                "Dynamic choice binding variables must be unique",
            ));
        }

        let iterable_type = match infer_expression_type(
            binding.iterable(),
            &self.indexes.variable_scopes,
            &self.indexes.struct_types,
            &self.indexes.enum_types,
            &self.indexes.target_symbols,
            &self.indexes.interface_members,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) {
            Ok(iterable_type) => iterable_type,
            Err(error) => {
                self.diagnostics.push(Diagnostic::error(
                    choice.span().clone(),
                    format!(
                        "Cannot type-check dynamic choice iterable: {}",
                        error.message()
                    ),
                ));
                return;
            }
        };

        if iterable_type.array_element_type().is_none() {
            self.diagnostics.push(Diagnostic::error(
                choice.span().clone(),
                format!(
                    "Dynamic choice iterable has type {} but expected array",
                    iterable_type.display_name()
                ),
            ));
        }
    }
}
