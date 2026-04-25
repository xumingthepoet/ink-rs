use crate::{
    diagnostic::{Diagnostic, DiagnosticSeverity},
    parsed::Story,
    source::SourceInput,
    syntax,
};

pub(super) fn parse_story(source: &str) -> Story {
    let output = syntax::parse(SourceInput::new(source));
    assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
    output.artifact.expect("parser should return a story")
}

pub(super) fn assert_single_diagnostic(
    diagnostics: &[Diagnostic],
    severity: DiagnosticSeverity,
    message: &str,
) {
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(diagnostics[0].severity, severity);
    assert_eq!(diagnostics[0].message, message);
}
