use std::collections::HashMap;

use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_story, ParsedVisitor, VisitContext},
        Expression, Object, Story,
    },
};

pub(super) fn constant_redefinition_diagnostics(story: &Story) -> Vec<Diagnostic> {
    #[derive(Default)]
    struct ConstantRedefinitionVisitor {
        constants: HashMap<String, Expression>,
        diagnostics: Vec<Diagnostic>,
    }

    impl ParsedVisitor for ConstantRedefinitionVisitor {
        fn visit_object(&mut self, object: &Object, _context: &VisitContext) {
            if let Object::ConstantDeclaration(declaration) = object {
                if let Some(existing) = self.constants.get(declaration.name()) {
                    if existing != declaration.expression() {
                        self.diagnostics.push(Diagnostic::error(
                            declaration.span().clone(),
                            format!(
                                "CONST '{}' has been redefined with a different value",
                                declaration.name()
                            ),
                        ));
                    }
                }
                self.constants.insert(
                    declaration.name().to_string(),
                    declaration.expression().clone(),
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
        let story = parse_story("CONST score = 1\nCONST score = 2");
        let diagnostics = constant_redefinition_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "CONST 'score' has been redefined with a different value",
        );
    }

    #[test]
    fn allows_same_value_constant_redefinition() {
        let story = parse_story("CONST score = 1\nCONST score = 1");

        assert!(constant_redefinition_diagnostics(&story).is_empty());
    }
}
