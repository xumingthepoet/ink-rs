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

#[cfg(test)]
mod tests {
    use crate::diagnostic::DiagnosticSeverity;

    use super::{
        super::test_support::{assert_single_diagnostic, parse_story},
        *,
    };

    #[test]
    fn reports_author_warning_nodes() {
        let story = parse_story("TODO: check this branch\nLine.");
        let diagnostics = author_warning_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Author,
            "check this branch",
        );
    }
}
