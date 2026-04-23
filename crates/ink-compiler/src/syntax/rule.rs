use crate::{
    diagnostic::Diagnostic,
    source::{SourceLine, SourceSpan},
};

use super::{error, state::ParserState};

pub(super) struct RuleParser<'source> {
    input: &'source str,
    source_name: Option<String>,
    line: usize,
    column_base: usize,
    state: ParserState,
    diagnostics: Vec<Diagnostic>,
}

impl<'source> RuleParser<'source> {
    pub(super) fn new(line: &'source SourceLine) -> Self {
        Self {
            input: &line.text,
            source_name: line.span.source_name.clone(),
            line: line.span.line,
            column_base: line.span.column,
            state: ParserState::new(),
            diagnostics: Vec::new(),
        }
    }

    pub(super) fn parse_rule<T>(&mut self, rule: impl FnOnce(&mut Self) -> Option<T>) -> Option<T> {
        let rule_id = self.state.begin_rule();
        let result = rule(self);

        match result {
            Some(result) => {
                self.state.succeed_rule(rule_id);
                Some(result)
            }
            None => {
                self.state.fail_rule(rule_id);
                None
            }
        }
    }

    pub(super) fn expect<T>(
        &mut self,
        expected: &str,
        rule: impl FnOnce(&mut Self) -> Option<T>,
        recovery: impl FnOnce(&mut Self),
    ) -> Option<T> {
        let result = self.parse_rule(rule);
        if result.is_none() {
            self.error(error::expected_message(expected, self.line_remainder()));
            recovery(self);
        }
        result
    }

    pub(super) fn error(&mut self, message: impl Into<String>) {
        if self.state.error_reported_in_scope() {
            return;
        }

        self.diagnostics
            .push(Diagnostic::error(self.current_span(), message.into()));
        self.state.note_error_reported();
    }

    pub(super) fn had_error(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    pub(super) fn finish(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub(super) fn current_span(&self) -> SourceSpan {
        SourceSpan::new(
            self.source_name.clone(),
            self.line,
            self.column_base + self.state.character_in_line(),
        )
    }

    pub(super) fn current_char(&self) -> Option<char> {
        self.input[self.state.byte_index()..].chars().next()
    }

    pub(super) fn skip_horizontal_whitespace(&mut self) {
        while matches!(self.current_char(), Some(' ' | '\t')) {
            self.advance_char();
        }
    }

    pub(super) fn match_string(&mut self, expected: &str) -> Option<String> {
        if self.line_remainder().starts_with(expected) {
            self.advance_string(expected);
            Some(expected.to_string())
        } else {
            None
        }
    }

    pub(super) fn take_to_end_trimmed(&mut self) -> Option<String> {
        let text = self.line_remainder().trim().to_string();
        self.skip_to_end();

        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }

    pub(super) fn skip_to_end(&mut self) {
        let character_count = self.line_remainder().chars().count();
        self.state.set_position(
            self.input.len(),
            self.state.character_in_line() + character_count,
        );
    }

    pub(super) fn line_remainder(&self) -> &str {
        &self.input[self.state.byte_index()..]
    }

    #[cfg(test)]
    pub(super) fn peek<T>(&mut self, rule: impl FnOnce(&mut Self) -> Option<T>) -> Option<T> {
        let rule_id = self.state.begin_rule();
        let result = rule(self);
        self.state.fail_rule(rule_id);
        result
    }

    fn advance_string(&mut self, text: &str) {
        for ch in text.chars() {
            self.state.advance(ch.len_utf8());
        }
    }

    fn advance_char(&mut self) {
        if let Some(ch) = self.current_char() {
            self.state.advance(ch.len_utf8());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser_for(text: &str) -> RuleParser<'_> {
        let line = SourceLine {
            text: text.to_string(),
            span: SourceSpan::new(None, 1, 1),
        };
        RuleParser::new(Box::leak(Box::new(line)))
    }

    #[test]
    fn failed_rule_rewinds_cursor() {
        let mut parser = parser_for("* choice");

        assert!(parser
            .parse_rule(|parser| parser.match_string("+"))
            .is_none());
        assert_eq!(parser.line_remainder(), "* choice");
    }

    #[test]
    fn successful_rule_commits_cursor() {
        let mut parser = parser_for("* choice");

        assert_eq!(
            parser.parse_rule(|parser| parser.match_string("*")),
            Some("*".to_string())
        );
        assert_eq!(parser.line_remainder(), " choice");
    }

    #[test]
    fn expect_reports_once_per_scope_and_recovers() {
        let mut parser = parser_for("*");

        parser.match_string("*");
        assert!(parser
            .expect(
                "choice text",
                |parser| parser.take_to_end_trimmed(),
                |parser| {
                    parser.skip_to_end();
                }
            )
            .is_none());

        let diagnostics = parser.finish();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "Expected choice text but saw end of line".to_string()
        );
    }

    #[test]
    fn peek_rewinds_after_success() {
        let mut parser = parser_for("-> DONE");

        assert_eq!(
            parser.peek(|parser| parser.match_string("->")),
            Some("->".to_string())
        );
        assert_eq!(parser.line_remainder(), "-> DONE");
    }
}
