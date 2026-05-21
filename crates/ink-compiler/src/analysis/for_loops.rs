use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_story, ParsedVisitor, VisitContext},
        Conditional, ContentList, DictKeyType, ForLoop, Object, Story, TypeName, Weave,
    },
};

use super::{expression_types::infer_expression_type, indexes::AnalysisIndexes, span::object_span};

pub(super) fn for_loop_diagnostics_with_indexes(
    story: &Story,
    indexes: &AnalysisIndexes<'_>,
) -> Vec<Diagnostic> {
    let mut checker = ForLoopChecker {
        indexes,
        diagnostics: Vec::new(),
    };
    walk_story(story, &mut checker);
    checker.diagnostics
}

pub(super) fn loop_variable_types(
    iterable_type: &TypeName,
    variable_count: usize,
) -> Option<Vec<TypeName>> {
    if let Some(element_type) = iterable_type.array_element_type() {
        return match variable_count {
            1 => Some(vec![element_type.clone()]),
            2 => Some(vec![TypeName::int(), element_type.clone()]),
            _ => None,
        };
    }

    if let Some((key_type, value_type)) = iterable_type.dict_key_value_types() {
        return (variable_count == 2)
            .then(|| vec![dict_key_type_name(key_type), value_type.clone()]);
    }

    None
}

pub(super) fn dict_key_type_name(key_type: DictKeyType) -> TypeName {
    match key_type {
        DictKeyType::String => TypeName::string(),
        DictKeyType::Int => TypeName::int(),
    }
}

struct ForLoopChecker<'a> {
    indexes: &'a AnalysisIndexes<'a>,
    diagnostics: Vec<Diagnostic>,
}

impl ParsedVisitor for ForLoopChecker<'_> {
    fn visit_object(&mut self, object: &Object, context: &VisitContext) {
        let Object::ForLoop(for_loop) = object else {
            return;
        };
        self.check_header(for_loop, context);
        self.check_body(for_loop);
    }
}

impl ForLoopChecker<'_> {
    fn check_header(&mut self, for_loop: &ForLoop, context: &VisitContext) {
        let iterable_type = match infer_expression_type(
            for_loop.iterable(),
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
                    for_loop.span().clone(),
                    format!("Cannot type-check for loop iterable: {}", error.message()),
                ));
                return;
            }
        };

        if iterable_type.array_element_type().is_some() {
            if !(1..=2).contains(&for_loop.variables().len()) {
                self.diagnostics.push(Diagnostic::error(
                    for_loop.span().clone(),
                    format!(
                        "Array for loops expect one item variable or index, item variables but got {}",
                        for_loop.variables().len()
                    ),
                ));
            }
            return;
        }

        if iterable_type.dict_key_value_types().is_some() {
            if for_loop.variables().len() != 2 {
                self.diagnostics.push(Diagnostic::error(
                    for_loop.span().clone(),
                    format!(
                        "Dict for loops require key and value variables but got {}",
                        for_loop.variables().len()
                    ),
                ));
            }
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            for_loop.span().clone(),
            format!(
                "For loop iterable has type {} but expected array or Dict",
                iterable_type.display_name()
            ),
        ));
    }

    fn check_body(&mut self, for_loop: &ForLoop) {
        check_for_body_weave(for_loop.body(), &mut self.diagnostics);
    }
}

fn check_for_body_weave(weave: &Weave, diagnostics: &mut Vec<Diagnostic>) {
    for object in weave.content() {
        check_for_body_object(object, diagnostics);
    }
}

fn check_for_body_content_list(content: &ContentList, diagnostics: &mut Vec<Diagnostic>) {
    for object in content.objects() {
        check_for_body_object(object, diagnostics);
    }
}

fn check_for_body_conditional(conditional: &Conditional, diagnostics: &mut Vec<Diagnostic>) {
    for branch in conditional.branches() {
        check_for_body_weave(branch.content(), diagnostics);
    }
}

fn check_for_body_object(object: &Object, diagnostics: &mut Vec<Diagnostic>) {
    match object {
        Object::ContentList(content) => check_for_body_content_list(content, diagnostics),
        Object::Conditional(conditional) => check_for_body_conditional(conditional, diagnostics),
        Object::ForLoop(_) => {}
        Object::VariableAssignment(assignment) if assignment.is_global() => {
            diagnostics.push(Diagnostic::error(
                assignment.span().clone(),
                "Global VAR declarations are not allowed inside for loops.",
            ));
        }
        Object::Return(ret) => {
            diagnostics.push(Diagnostic::error(
                ret.span().clone(),
                "`~ return` is not allowed inside for loops.",
            ));
        }
        Object::Choice(_) => push_unsupported_body_diagnostic(object, "Choices", diagnostics),
        Object::Divert(_) => push_unsupported_body_diagnostic(object, "Diverts", diagnostics),
        Object::TunnelOnwards(_) => {
            push_unsupported_body_diagnostic(object, "Tunnel onwards", diagnostics);
        }
        Object::Gather(_) => push_unsupported_body_diagnostic(object, "Gathers", diagnostics),
        Object::Weave(_) => push_unsupported_body_diagnostic(object, "Nested weave", diagnostics),
        Object::AuthorWarning(_) => {
            push_unsupported_body_diagnostic(object, "Author warnings", diagnostics);
        }
        Object::ConstantDeclaration(_) => {
            push_unsupported_body_diagnostic(object, "CONST declarations", diagnostics);
        }
        Object::EnumDeclaration(_) => {
            push_unsupported_body_diagnostic(object, "ENUM declarations", diagnostics);
        }
        Object::ExternalDeclaration(_) => {
            push_unsupported_body_diagnostic(object, "EXTERNAL declarations", diagnostics);
        }
        Object::StructDeclaration(_) => {
            push_unsupported_body_diagnostic(object, "STRUCT declarations", diagnostics);
        }
        Object::Tag(_) => push_unsupported_body_diagnostic(object, "Tags", diagnostics),
        Object::Text(_)
        | Object::Expression(_)
        | Object::Glue(_)
        | Object::IncDec(_)
        | Object::LogicLine(_)
        | Object::VariableAssignment(_) => {}
    }
}

fn push_unsupported_body_diagnostic(
    object: &Object,
    object_name: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    diagnostics.push(Diagnostic::error(
        object_span(object),
        format!("{object_name} are not allowed inside for loops."),
    ));
}

#[cfg(test)]
mod tests {
    use crate::diagnostic::DiagnosticSeverity;

    use super::{
        super::{
            indexes::AnalysisIndexes,
            modules::ModuleAnalysis,
            run_analysis_passes,
            test_support::{assert_single_diagnostic, parse_story},
        },
        *,
    };

    fn diagnostics(source: &str) -> Vec<Diagnostic> {
        let story = parse_story(source);
        let module_analysis = ModuleAnalysis::build(&story);
        let indexes = AnalysisIndexes::build(&story, &module_analysis);
        for_loop_diagnostics_with_indexes(&story, &indexes)
    }

    #[test]
    fn accepts_array_dict_and_nested_for_loop_variable_types() {
        let diagnostics = diagnostics(
            "=== module game ===\n\
             STRUCT Row { values: int[] }\n\
             VAR rows: Row[] = [%Row{ values: [1, 2] }]\n\
             VAR scores: Dict<string, int> = %{\"ada\": 10}\n\
             == main ==\n\
             { for row in rows:\n\
                 { for value in row.values:\n\
                     ~ temp doubled: int = value + value\n\
                 }\n\
             }\n\
             { for key, value in scores:\n\
                 ~ temp label: string = key\n\
                 ~ temp score: int = value\n\
             }\n\
             -> DONE",
        );

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn rejects_non_iterable_expression() {
        let diagnostics = diagnostics(
            "=== module game ===\n\
             VAR score: int = 1\n\
             == main ==\n\
             { for item in score:\n\
                 item\n\
             }\n\
             -> DONE",
        );

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "For loop iterable has type int but expected array or Dict",
        );
    }

    #[test]
    fn rejects_wrong_array_and_dict_variable_counts() {
        let diagnostics = diagnostics(
            "=== module game ===\n\
             VAR values: int[] = [1]\n\
             VAR scores: Dict<string, int> = %{\"ada\": 10}\n\
             == main ==\n\
             { for a, b, c in values:\n\
                 value\n\
             }\n\
             { for key in scores:\n\
                 value\n\
             }\n\
             -> DONE",
        );

        assert_eq!(diagnostics.len(), 2, "{diagnostics:#?}");
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.message
            == "Array for loops expect one item variable or index, item variables but got 3"));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.message
            == "Dict for loops require key and value variables but got 1"));
    }

    #[test]
    fn rejects_unsupported_for_body_objects() {
        let diagnostics = diagnostics(
            "=== module game ===\n\
             VAR values: int[] = [1]\n\
             == main ==\n\
             { for value in values:\n\
                 -> DONE\n\
             }\n\
             -> DONE",
        );

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Diverts are not allowed inside for loops.",
        );
    }

    #[test]
    fn loop_variables_are_not_visible_after_the_loop() {
        let story = parse_story(
            "=== module game ===\n\
             VAR values: int[] = [1]\n\
             == main ==\n\
             { for value in values:\n\
                 {value}\n\
             }\n\
             ~ temp after: int = value\n\
             -> DONE",
        );
        let diagnostics = run_analysis_passes(&story);

        assert!(
            diagnostics.iter().any(
                |diagnostic| diagnostic.severity == DiagnosticSeverity::Error
                    && diagnostic.message == "Unresolved variable: value"
            ),
            "{diagnostics:#?}"
        );
    }
}
