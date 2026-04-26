use std::collections::HashMap;

use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_story, ParsedVisitor, VisitContext},
        Expression, Object, Story, TypeName,
    },
};

pub(super) fn constant_redefinition_diagnostics(story: &Story) -> Vec<Diagnostic> {
    #[derive(Default)]
    struct ConstantRedefinitionVisitor {
        constants: HashMap<String, (TypeName, Expression)>,
        diagnostics: Vec<Diagnostic>,
    }

    impl ParsedVisitor for ConstantRedefinitionVisitor {
        fn visit_object(&mut self, object: &Object, context: &VisitContext) {
            if context.current_module.is_some() {
                return;
            }

            if let Object::ConstantDeclaration(declaration) = object {
                if let Some(existing) = self.constants.get(declaration.name()) {
                    if existing.0 != *declaration.declared_type()
                        || existing.1 != *declaration.expression()
                    {
                        self.diagnostics.push(Diagnostic::error(
                            declaration.span().clone(),
                            format!(
                                "CONST '{}' has been redefined with a different type or value",
                                declaration.name()
                            ),
                        ));
                    }
                }
                self.constants.insert(
                    declaration.name().to_string(),
                    (
                        declaration.declared_type().clone(),
                        declaration.expression().clone(),
                    ),
                );
            }
        }
    }

    let mut visitor = ConstantRedefinitionVisitor::default();
    walk_story(story, &mut visitor);
    visitor.diagnostics
}

#[cfg(test)]
mod tests {
    use crate::diagnostic::DiagnosticSeverity;

    use super::{
        super::test_support::{assert_single_diagnostic, parse_story},
        *,
    };

    #[test]
    fn reports_changed_constant_redefinition() {
        let story = parse_story("CONST score: int = 1\nCONST score: int = 2");
        let diagnostics = constant_redefinition_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "CONST 'score' has been redefined with a different type or value",
        );
    }

    #[test]
    fn allows_same_value_constant_redefinition() {
        let story = parse_story("CONST score: int = 1\nCONST score: int = 1");

        assert!(constant_redefinition_diagnostics(&story).is_empty());
    }

    #[test]
    fn reports_changed_constant_redefinition_type() {
        let story = parse_story("CONST score: int = 1\nCONST score: string = \"1\"");
        let diagnostics = constant_redefinition_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "CONST 'score' has been redefined with a different type or value",
        );
    }

    #[test]
    fn leaves_module_constants_to_module_namespace_analysis() {
        let story = parse_story(
            "=== module first ===\n\
             CONST score: int = 1\n\
             === module second ===\n\
             CONST score: string = \"1\"",
        );

        assert!(constant_redefinition_diagnostics(&story).is_empty());
    }
}
