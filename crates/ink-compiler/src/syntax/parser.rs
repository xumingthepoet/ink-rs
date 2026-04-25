use crate::{
    compiler::StageOutput,
    diagnostic::Diagnostic,
    parsed::{Flow, Object, Story},
    source::{SourceFile, SourceInput, SourceLine},
};

use super::rule::RuleParser;
use super::weave::group_weave_content;
use super::{
    author_warning_statement, choice_statement, constant_declaration_statement, divert_statement,
    external_declaration_statement, gather, is_choice_continuation_boundary, knot,
    leading_whitespace_count, logic_line_statement, parse_choice_from_line, return_statement,
    temp_declaration_statement, text_statement, variable_assignment_statement,
    variable_declaration_statement,
};

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

pub(super) struct Parser {
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
        let lines = self.source.lines.clone();
        let mut objects = Vec::new();
        let mut flows = Vec::new();
        let mut index = 0;

        while index < lines.len() {
            let line = &lines[index];

            if line.text.trim().is_empty() {
                index += 1;
                continue;
            }

            if line.text.trim() == "}" {
                index += 1;
                continue;
            }

            if knot::is_knot_declaration_line(&line.text) {
                if let Some(flow) = self.parse_flow(&lines, &mut index) {
                    flows.push(flow);
                } else {
                    index += 1;
                }
                continue;
            }

            if knot::is_stitch_declaration_line(&line.text) {
                if let Some(flow) = self.parse_stitch(&lines, &mut index) {
                    flows.push(flow);
                    continue;
                }
            }

            if let Some(parsed) = self.parse_compound_statement(&lines, &mut index) {
                objects.extend(parsed);
                continue;
            }

            objects.extend(self.parse_statement(line));
            index += 1;
        }

        Story::new(group_weave_content(objects), flows)
    }

    pub(super) fn parse_statement(&mut self, line: &SourceLine) -> Vec<Object> {
        if line.text.trim().is_empty() {
            return Vec::new();
        }

        if let Some(objects) = self.parse_gather_line(line) {
            return objects;
        }

        let mut line_parser = RuleParser::new(line);
        let statement_rules: &[StatementRule] = &[
            variable_declaration_statement,
            constant_declaration_statement,
            external_declaration_statement,
            return_statement,
            temp_declaration_statement,
            variable_assignment_statement,
            logic_line_statement,
            choice_statement,
            author_warning_statement,
            divert_statement,
            text_statement,
        ];

        for rule in statement_rules {
            if let Some(rule_match) = line_parser.parse_rule_with_metadata(*rule) {
                debug_assert!(rule_match.metadata.is_forward());
                self.diagnostics.extend(line_parser.finish());
                return rule_match.value;
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

    fn parse_gather_line(&mut self, line: &SourceLine) -> Option<Vec<Object>> {
        let mut line_parser = RuleParser::new(line);
        let mut objects = line_parser.parse_rule(gather::parse_statement)?;
        let had_error = line_parser.had_error();
        line_parser.skip_horizontal_whitespace();

        let remaining = line_parser.line_remainder().to_string();
        let remaining_span = line_parser.current_span();
        self.diagnostics.extend(line_parser.finish());

        if had_error {
            return Some(Vec::new());
        }

        if !remaining.trim().is_empty() {
            let remaining_line = SourceLine {
                text: remaining,
                span: remaining_span,
            };
            objects.extend(self.parse_statement(&remaining_line));
        }

        Some(objects)
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
        } else if knot::is_knot_declaration_line(trimmed) {
            Some("knot declaration")
        } else if trimmed.starts_with('*') || trimmed.starts_with('+') {
            Some("choice")
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

    fn parse_flow(&mut self, lines: &[SourceLine], index: &mut usize) -> Option<Flow> {
        let line = &lines[*index];
        let mut line_parser = RuleParser::new(line);
        let declaration = line_parser.parse_rule(knot::parse_knot_declaration);
        let had_error = line_parser.had_error();
        self.diagnostics.extend(line_parser.finish());

        let Some(declaration) = declaration else {
            return None;
        };

        if had_error {
            return None;
        }

        *index += 1;
        let mut content = Vec::new();
        let mut child_flows = Vec::new();

        while *index < lines.len() {
            let next_line = &lines[*index];
            if next_line.text.trim().is_empty() {
                *index += 1;
                continue;
            }

            if next_line.text.trim() == "}" {
                *index += 1;
                continue;
            }

            if knot::is_knot_declaration_line(&next_line.text) {
                break;
            }

            if knot::is_stitch_declaration_line(&next_line.text) {
                if let Some(stitch) = self.parse_stitch(lines, index) {
                    child_flows.push(stitch);
                } else {
                    *index += 1;
                }
                continue;
            }

            if let Some(parsed) = self.parse_compound_statement(lines, index) {
                content.extend(parsed);
                continue;
            }

            content.extend(self.parse_statement(next_line));
            *index += 1;
        }

        Some(Flow::new(
            declaration.level,
            declaration.name,
            group_weave_content(content),
            child_flows,
            declaration.arguments,
            declaration.is_function,
        ))
    }

    fn parse_stitch(&mut self, lines: &[SourceLine], index: &mut usize) -> Option<Flow> {
        let line = &lines[*index];
        let mut line_parser = RuleParser::new(line);
        let declaration = line_parser.parse_rule(knot::parse_stitch_declaration);
        let had_error = line_parser.had_error();
        self.diagnostics.extend(line_parser.finish());

        let Some(declaration) = declaration else {
            return None;
        };

        if had_error {
            return None;
        }

        *index += 1;
        let mut content = Vec::new();

        while *index < lines.len() {
            let next_line = &lines[*index];
            if next_line.text.trim().is_empty() {
                *index += 1;
                continue;
            }

            if next_line.text.trim() == "}" {
                *index += 1;
                continue;
            }

            if knot::is_knot_declaration_line(&next_line.text)
                || knot::is_stitch_declaration_line(&next_line.text)
            {
                break;
            }

            if let Some(parsed) = self.parse_compound_statement(lines, index) {
                content.extend(parsed);
                continue;
            }

            content.extend(self.parse_statement(next_line));
            *index += 1;
        }

        Some(Flow::new(
            declaration.level,
            declaration.name,
            group_weave_content(content),
            Vec::new(),
            declaration.arguments,
            declaration.is_function,
        ))
    }

    pub(super) fn parse_compound_statement(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<Vec<Object>> {
        if let Some(parsed) = self.parse_multiline_rule(index, |parser, index| {
            parser.parse_choice_with_continuation(lines, index)
        }) {
            return Some(parsed);
        }

        if let Some(parsed) = self.parse_multiline_rule(index, |parser, index| {
            parser.parse_multiline_sequence(lines, index)
        }) {
            return Some(parsed);
        }

        self.parse_multiline_rule(index, |parser, index| {
            parser.parse_multiline_conditional(lines, index)
        })
    }

    pub(super) fn parse_multiline_rule<T>(
        &mut self,
        index: &mut usize,
        rule: impl FnOnce(&mut Self, &mut usize) -> Option<T>,
    ) -> Option<T> {
        let start_index = *index;
        let diagnostic_count = self.diagnostics.len();

        let result = rule(self, index);
        if result.is_none() {
            *index = start_index;
            self.diagnostics.truncate(diagnostic_count);
        }

        result
    }

    fn parse_choice_with_continuation(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<Vec<Object>> {
        let line = &lines[*index];
        if !line
            .text
            .trim_start()
            .starts_with(|ch| matches!(ch, '*' | '+'))
        {
            return None;
        }

        let initial_choice = parse_choice_from_line(line)?;
        if !initial_choice.is_invisible_default() || initial_choice.condition().is_none() {
            return None;
        }

        let current_indent = leading_whitespace_count(&line.text);
        let mut next_index = *index + 1;
        let mut continuation_parts = Vec::new();
        while next_index < lines.len() {
            let next_line = &lines[next_index];
            let trimmed = next_line.text.trim();
            if trimmed.is_empty()
                || trimmed == "}"
                || leading_whitespace_count(&next_line.text) <= current_indent
                || is_choice_continuation_boundary(trimmed)
            {
                break;
            }

            continuation_parts.push(trimmed.to_string());
            next_index += 1;
        }

        if continuation_parts.is_empty() {
            return None;
        }

        let combined = format!("{} {}", line.text.trim_end(), continuation_parts.join(" "));
        let combined_line = SourceLine {
            text: combined,
            span: line.span.clone(),
        };
        let combined_choice = parse_choice_from_line(&combined_line)?;
        if combined_choice.is_invisible_default() {
            return None;
        }

        *index = next_index;
        Some(vec![Object::Choice(combined_choice)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_multiline_compound_rule_rewinds_index() {
        let source = SourceFile::from_input(SourceInput::new("{ once\n- A"));
        let lines = source.lines.clone();
        let mut parser = Parser::new(source);
        let mut index = 0;

        assert!(parser
            .parse_compound_statement(&lines, &mut index)
            .is_none());

        assert_eq!(index, 0);
        assert!(parser.diagnostics.is_empty(), "{:#?}", parser.diagnostics);
    }
}
