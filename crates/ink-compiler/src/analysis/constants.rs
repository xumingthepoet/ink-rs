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
