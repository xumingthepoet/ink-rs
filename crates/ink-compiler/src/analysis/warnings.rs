use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_story, ParsedVisitor, VisitContext},
        Object, Story,
    },
};

pub(super) fn author_warning_diagnostics(story: &Story) -> Vec<Diagnostic> {
    #[derive(Default)]
    struct AuthorWarningVisitor {
        diagnostics: Vec<Diagnostic>,
    }

    impl ParsedVisitor for AuthorWarningVisitor {
        fn visit_object(&mut self, object: &Object, _context: &VisitContext) {
            if let Object::AuthorWarning(author_warning) = object {
                self.diagnostics.push(Diagnostic::author(
                    author_warning.span().clone(),
                    author_warning.message().to_string(),
                ));
            }
        }
    }

    let mut visitor = AuthorWarningVisitor::default();
    walk_story(story, &mut visitor);
    visitor.diagnostics
}
