use crate::{
    compiler::StageOutput,
    diagnostic::Diagnostic,
    parsed::{Conditional, ContentList, Flow, Object, Sequence, Story, Text},
    source::{SourceFile, SourceInput, SourceLine},
};

use super::rule::RuleParser;
use super::{
    author_warning_statement, choice_statement, classify_conditional_branches,
    constant_declaration_statement, divert_statement, external_declaration_statement,
    gather_statement, group_weave_content, is_choice_continuation_boundary,
    is_multiline_sequence_element_start, knot, leading_whitespace_count, logic_line_statement,
    parse_choice_from_line, parse_conditional_branch_header,
    parse_default_conditional_branch_content, parse_initial_expression,
    parse_multiline_conditional_prefix, return_statement, temp_declaration_statement, text,
    text_statement, variable_assignment_statement, variable_declaration_statement,
    ConditionalBranchBuilder,
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

    fn parse_statement(&mut self, line: &SourceLine) -> Vec<Object> {
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
        let mut objects = line_parser.parse_rule(gather_statement)?;
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

    fn parse_compound_statement(
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

    fn parse_multiline_rule<T>(
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

    fn parse_multiline_conditional(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<Vec<Object>> {
        let line = &lines[*index];
        let (prefix, conditional_source) = parse_multiline_conditional_prefix(line)?;
        let trimmed = conditional_source.trim();
        if !trimmed.starts_with('{') {
            return None;
        }

        let after_open = trimmed.strip_prefix('{')?.trim();
        let initial_condition = if after_open.is_empty() {
            None
        } else {
            let condition_source = after_open.strip_suffix(':')?.trim();
            Some(parse_initial_expression(condition_source)?)
        };

        *index += 1;
        let mut branches = Vec::new();
        let mut current_branch = ConditionalBranchBuilder::true_branch();

        while *index < lines.len() {
            let current_line = &lines[*index];
            let current_trimmed = current_line.text.trim();

            if let Some(after_close) = current_trimmed.strip_prefix('}') {
                branches.push(current_branch);
                classify_conditional_branches(initial_condition.is_some(), &mut branches);
                let branches = branches
                    .into_iter()
                    .map(ConditionalBranchBuilder::finish)
                    .collect();
                let conditional = Conditional::new(initial_condition, branches);
                let mut objects = prefix;
                objects.push(Object::ContentList(ContentList::new(vec![
                    Object::Conditional(conditional),
                ])));
                let suffix = after_close.trim_start();
                if suffix.is_empty() {
                    *index += 1;
                    objects.push(Object::Text(Text::new("\n", current_line.span.clone())));
                    return Some(objects);
                }
                if let Some(mut suffix_objects) =
                    self.parse_multiline_conditional_suffix(suffix, current_line, lines, index)
                {
                    objects.append(&mut suffix_objects);
                    return Some(objects);
                }
                let suffix_line = SourceLine {
                    text: suffix.to_string(),
                    span: current_line.span.clone(),
                };
                objects.extend(self.parse_statement(&suffix_line));
                *index += 1;
                return Some(objects);
            }

            if let Some(content) = parse_default_conditional_branch_content(current_trimmed) {
                if let Some(nested_objects) =
                    self.parse_nested_conditional_branch_content(lines, index, content)
                {
                    if current_branch.has_content() || !branches.is_empty() {
                        branches.push(current_branch);
                    }
                    current_branch = ConditionalBranchBuilder::content_branch();
                    current_branch.objects.extend(nested_objects);
                    continue;
                }
            }

            if let Some(parsed_branch) = parse_conditional_branch_header(current_trimmed) {
                if current_branch.has_content() || !branches.is_empty() {
                    branches.push(current_branch);
                }
                current_branch = parsed_branch.builder;
                if let Some(content) = parsed_branch.inline_content {
                    self.append_conditional_branch_inline_content(
                        &mut current_branch,
                        content,
                        current_line,
                    );
                }
                *index += 1;
                continue;
            }

            if let Some(content) = parse_default_conditional_branch_content(current_trimmed) {
                if current_branch.has_content() || !branches.is_empty() {
                    branches.push(current_branch);
                }
                current_branch = ConditionalBranchBuilder::content_branch();
                self.append_conditional_branch_inline_content(
                    &mut current_branch,
                    content,
                    current_line,
                );
                *index += 1;
                continue;
            }

            if current_trimmed.starts_with('{') {
                if let Some(nested_objects) = self.parse_compound_statement(lines, index) {
                    current_branch.objects.extend(nested_objects);
                    continue;
                }
            }

            current_branch
                .objects
                .extend(self.parse_statement(current_line));
            *index += 1;
        }

        None
    }

    fn parse_multiline_conditional_suffix(
        &mut self,
        suffix: &str,
        source_line: &SourceLine,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<Vec<Object>> {
        for (brace_index, _) in suffix.match_indices('{') {
            let prefix = &suffix[..brace_index];
            let nested_source = suffix[brace_index..].trim_start();
            let mut nested_lines = Vec::with_capacity(lines.len() - *index);
            nested_lines.push(SourceLine {
                text: nested_source.to_string(),
                span: source_line.span.clone(),
            });
            nested_lines.extend(lines[*index + 1..].iter().cloned());

            let mut nested_index = 0;
            let nested_objects = self
                .parse_multiline_rule(&mut nested_index, |parser, index| {
                    parser.parse_multiline_conditional(&nested_lines, index)
                })?;
            if nested_index == 0 {
                continue;
            }

            let mut objects = Vec::new();
            if !prefix.trim().is_empty() {
                objects.extend(text::parse_inline_content(
                    prefix.trim_start(),
                    &source_line.span,
                )?);
            }
            objects.extend(nested_objects);
            *index += nested_index;
            return Some(objects);
        }

        None
    }

    fn parse_nested_conditional_branch_content(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
        content: &str,
    ) -> Option<Vec<Object>> {
        if !content.trim_start().starts_with('{') {
            return None;
        }

        let mut nested_lines = Vec::with_capacity(lines.len() - *index);
        nested_lines.push(SourceLine {
            text: content.trim_start().to_string(),
            span: lines[*index].span.clone(),
        });
        nested_lines.extend(lines[*index + 1..].iter().cloned());

        let mut nested_index = 0;
        let nested_objects = self.parse_multiline_rule(&mut nested_index, |parser, index| {
            parser.parse_multiline_conditional(&nested_lines, index)
        })?;
        if nested_index == 0 {
            return None;
        }

        *index += nested_index;
        Some(nested_objects)
    }

    fn append_conditional_branch_inline_content(
        &mut self,
        branch: &mut ConditionalBranchBuilder,
        content: &str,
        source_line: &SourceLine,
    ) {
        if content.trim().is_empty() {
            return;
        }

        let content_line = SourceLine {
            text: content.trim_start().to_string(),
            span: source_line.span.clone(),
        };
        branch.objects.extend(self.parse_statement(&content_line));
    }

    fn parse_multiline_sequence(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<Vec<Object>> {
        let line = &lines[*index];
        let trimmed = line.text.trim();
        let after_open = trimmed.strip_prefix('{')?.trim();
        let (sequence_type, rest) = text::parse_sequence_type_annotation(after_open)?;
        if !rest.trim().is_empty() {
            return None;
        }

        *index += 1;
        let mut elements = Vec::new();

        while *index < lines.len() {
            let current_line = &lines[*index];
            let current_trimmed = current_line.text.trim();

            if current_trimmed == "}" {
                *index += 1;
                return Some(vec![
                    Object::ContentList(ContentList::new(vec![Object::Sequence(Sequence::new(
                        sequence_type,
                        elements,
                    ))])),
                    Object::Text(Text::new("\n", current_line.span.clone())),
                ]);
            }

            if current_trimmed.is_empty() {
                *index += 1;
                continue;
            }

            elements.push(self.parse_multiline_sequence_element(lines, index)?);
        }

        None
    }

    fn parse_multiline_sequence_element(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<ContentList> {
        let line = &lines[*index];
        let trimmed = line.text.trim_start();
        if trimmed.starts_with("->") {
            return None;
        }

        let remainder = trimmed.strip_prefix('-')?.trim_start();
        let mut objects = Vec::new();
        if !remainder.is_empty() {
            objects.push(Object::Text(Text::new("\n", line.span.clone())));
            let content_line = SourceLine {
                text: remainder.to_string(),
                span: line.span.clone(),
            };
            objects.extend(self.parse_statement(&content_line));
        }

        *index += 1;
        while *index < lines.len() {
            let next_line = &lines[*index];
            let next_trimmed = next_line.text.trim();
            if next_trimmed == "}" || is_multiline_sequence_element_start(next_line) {
                break;
            }
            if !next_trimmed.is_empty() {
                if objects.is_empty() {
                    objects.push(Object::Text(Text::new("\n", next_line.span.clone())));
                }
                objects.extend(self.parse_statement(next_line));
            }
            *index += 1;
        }

        Some(ContentList::new(objects))
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
