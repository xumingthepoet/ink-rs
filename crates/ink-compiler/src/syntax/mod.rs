mod choice;
mod divert;
mod error;
mod rule;
mod state;
mod text;

use crate::{
    compiler::StageOutput,
    diagnostic::Diagnostic,
    parsed::{Object, Story},
    source::{SourceFile, SourceInput, SourceLine},
};

use self::rule::RuleParser;

type StatementRule = for<'source> fn(&mut RuleParser<'source>) -> Option<Vec<Object>>;

pub(crate) fn parse(input: SourceInput) -> StageOutput<Story> {
    let source = SourceFile::from_input(input);
    let mut parser = Parser::new(source);
    let story = parser.parse_story();

    StageOutput {
        artifact: Some(story),
        diagnostics: parser.diagnostics,
    }
}

struct Parser {
    source: SourceFile,
    diagnostics: Vec<Diagnostic>,
}

impl Parser {
    fn new(source: SourceFile) -> Self {
        Self {
            source,
            diagnostics: Vec::new(),
        }
    }

    fn parse_story(&mut self) -> Story {
        let mut objects = Vec::new();

        for line in self.source.lines.clone() {
            objects.extend(self.parse_statement(&line));
        }

        Story::new(objects)
    }

    fn parse_statement(&mut self, line: &SourceLine) -> Vec<Object> {
        if line.text.trim().is_empty() {
            return Vec::new();
        }

        let mut line_parser = RuleParser::new(line);
        let statement_rules: &[StatementRule] =
            &[choice_statement, divert_statement, text_statement];

        for rule in statement_rules {
            if let Some(objects) = line_parser.parse_rule(*rule) {
                self.diagnostics.extend(line_parser.finish());
                return objects;
            }

            if line_parser.had_error() {
                self.diagnostics.extend(line_parser.finish());
                return Vec::new();
            }
        }

        let diagnostics = line_parser.finish();
        self.diagnostics.extend(diagnostics);

        if let Some(diagnostic) = self.try_unsupported_statement(line) {
            self.diagnostics.push(diagnostic);
        }
        Vec::new()
    }

    fn try_unsupported_statement(&self, line: &SourceLine) -> Option<Diagnostic> {
        let trimmed = line.text.trim_start();

        let feature = if trimmed.starts_with("INCLUDE ") {
            Some("include")
        } else if trimmed.starts_with("VAR ") {
            Some("global variable declaration")
        } else if trimmed.starts_with("LIST ") {
            Some("list declaration")
        } else if trimmed.starts_with("CONST ") {
            Some("constant declaration")
        } else if trimmed.starts_with("EXTERNAL ") {
            Some("external declaration")
        } else if trimmed.starts_with("===") {
            Some("knot declaration")
        } else if trimmed.starts_with('=') {
            Some("stitch declaration")
        } else if trimmed.starts_with('*') || trimmed.starts_with('+') {
            Some("choice")
        } else if trimmed.starts_with('-') && !trimmed.starts_with("->") {
            Some("gather")
        } else if trimmed.starts_with('~') {
            Some("logic line")
        } else if trimmed.contains('{') || trimmed.contains('}') {
            Some("inline logic")
        } else if trimmed.contains("<>") {
            Some("glue")
        } else if trimmed.contains('#') {
            Some("tag")
        } else {
            None
        };

        feature.map(|feature| Diagnostic::unsupported(line.span.clone(), feature))
    }
}

fn choice_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    choice::parse_choice(parser).map(|choice| vec![Object::Choice(choice)])
}

fn divert_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    divert::parse_divert(parser).map(|divert| vec![Object::Divert(divert)])
}

fn text_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    text::parse_text_line(parser).map(|text| text.into_iter().map(Object::Text).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_text_lines() {
        let output = parse(SourceInput::new("Line.\nOther line."));
        assert!(output.diagnostics.is_empty());
        let story = output.artifact.unwrap();
        assert_eq!(story.root_weave().content().len(), 4);
    }

    #[test]
    fn parses_choice() {
        let output = parse(SourceInput::new("* Choice"));
        assert!(output.diagnostics.is_empty());
        let story = output.artifact.unwrap();
        assert_eq!(story.root_weave().content().len(), 1);
    }

    #[test]
    fn reports_unsupported_sticky_choice() {
        let output = parse(SourceInput::new("+ Choice"));
        assert_eq!(output.diagnostics.len(), 1);
        assert!(output.artifact.unwrap().root_weave().content().is_empty());
    }
}
