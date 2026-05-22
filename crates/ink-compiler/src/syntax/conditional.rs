use crate::{
    parsed::{
        Conditional, ConditionalBranch, ConditionalKind, ContentList, Expression, Object, Text,
    },
    source::SourceLine,
};

use super::{
    gather, is_identifier_continue, parse_initial_expression, parser::Parser, scan, text,
    weave::weave_from_objects,
};

mod if_block;
mod switch_block;

struct ConditionalBranchBuilder {
    is_true_branch: bool,
    is_else: bool,
    explicit_else: bool,
    own_condition: Option<Expression>,
    objects: Vec<Object>,
}

impl ConditionalBranchBuilder {
    fn true_branch() -> Self {
        Self {
            is_true_branch: true,
            is_else: false,
            explicit_else: false,
            own_condition: None,
            objects: Vec::new(),
        }
    }

    fn content_branch() -> Self {
        Self {
            is_true_branch: false,
            is_else: false,
            explicit_else: false,
            own_condition: None,
            objects: Vec::new(),
        }
    }

    fn has_content(&self) -> bool {
        !self.objects.is_empty() || self.own_condition.is_some() || self.is_else
    }

    fn finish(self) -> ConditionalBranch {
        ConditionalBranch::new(
            self.is_true_branch,
            self.is_else,
            false,
            weave_from_objects(self.objects),
            self.own_condition,
        )
    }
}

struct ParsedConditionalBranchHeader<'a> {
    builder: ConditionalBranchBuilder,
    inline_content: Option<&'a str>,
}

struct ParsedConditionalHeader {
    kind: ConditionalKind,
    initial_condition: Option<Expression>,
}

fn parse_conditional_header(source: &str) -> Option<ParsedConditionalHeader> {
    let header = source.strip_suffix(':')?.trim();
    if_block::parse_header(header).or_else(|| switch_block::parse_header(header))
}

fn parse_branch_header(trimmed: &str) -> Option<ParsedConditionalBranchHeader<'_>> {
    let after_dash = trimmed.strip_prefix('-')?.trim_start();
    if trimmed.starts_with("->") {
        return None;
    }

    if let Some(after_else) = after_dash.strip_prefix("else") {
        if after_else
            .chars()
            .next()
            .is_some_and(is_identifier_continue)
        {
            return None;
        }

        let inline_content = after_else.trim_start().strip_prefix(':')?;
        return Some(ParsedConditionalBranchHeader {
            builder: ConditionalBranchBuilder {
                is_true_branch: false,
                is_else: true,
                explicit_else: true,
                own_condition: None,
                objects: Vec::new(),
            },
            inline_content: non_empty_trimmed(inline_content),
        });
    }

    let (condition_source, inline_content) = split_branch_header_separator(after_dash)?;
    Some(ParsedConditionalBranchHeader {
        builder: ConditionalBranchBuilder {
            is_true_branch: false,
            is_else: false,
            explicit_else: false,
            own_condition: Some(parse_initial_expression(condition_source.trim())?),
            objects: Vec::new(),
        },
        inline_content: non_empty_trimmed(inline_content),
    })
}

fn split_branch_header_separator(source: &str) -> Option<(&str, &str)> {
    for (index, _) in
        scan::top_level_token_matches_with_options(source, &[":"], scan::ScanOptions::inline_text())
    {
        let next_index = index + ':'.len_utf8();
        let previous_is_colon = source[..index].chars().next_back() == Some(':');
        let next_is_colon = source[next_index..].starts_with(':');
        if !previous_is_colon && !next_is_colon {
            return Some((&source[..index], &source[next_index..]));
        }
    }
    None
}

fn parse_default_branch_content(trimmed: &str) -> Option<&str> {
    if trimmed.starts_with("->") {
        return None;
    }
    let content = trimmed.strip_prefix('-')?.trim_start();
    Some(content)
}

fn classify_branches(
    kind: ConditionalKind,
    has_initial_condition: bool,
    branches: &mut [ConditionalBranchBuilder],
) {
    for branch in branches.iter_mut() {
        branch.is_true_branch = false;
        branch.is_else = branch.explicit_else;
    }

    if kind == ConditionalKind::If && has_initial_condition {
        if let Some(first_branch) = branches.first_mut() {
            if !first_branch.explicit_else {
                first_branch.is_true_branch = true;
            }
        }
    }
}

fn non_empty_trimmed(source: &str) -> Option<&str> {
    let trimmed = source.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

impl Parser {
    pub(super) fn parse_multiline_conditional(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<Vec<Object>> {
        let line = &lines[*index];
        let (prefix, conditional_source) = gather::parse_multiline_conditional_prefix(line)?;
        let trimmed = conditional_source.trim();
        if !trimmed.starts_with('{') {
            return None;
        }

        let after_open = trimmed.strip_prefix('{')?.trim();
        let header = parse_conditional_header(after_open)?;

        *index += 1;
        let mut branches = Vec::new();
        let mut current_branch = ConditionalBranchBuilder::true_branch();

        while *index < lines.len() {
            let current_line = &lines[*index];
            let current_trimmed = current_line.text.trim();

            if let Some(after_close) = current_trimmed.strip_prefix('}') {
                branches.push(current_branch);
                classify_branches(
                    header.kind,
                    header.initial_condition.is_some(),
                    &mut branches,
                );
                let branches = branches
                    .into_iter()
                    .map(ConditionalBranchBuilder::finish)
                    .collect();
                let conditional = Conditional::new(header.kind, header.initial_condition, branches);
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

            if let Some(content) = parse_default_branch_content(current_trimmed) {
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

            if let Some(parsed_branch) = parse_branch_header(current_trimmed) {
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

            if let Some(content) = parse_default_branch_content(current_trimmed) {
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
}

#[cfg(test)]
mod tests {
    use crate::{
        parsed::{ConditionalKind, Object},
        source::SourceInput,
        syntax::parse,
    };

    #[test]
    fn parses_multiline_conditional_into_content_list() {
        let output = parse(SourceInput::new("{ if true:\nyes\n}"));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("expected story");
        let has_if = story.root_weave().content().iter().any(|object| {
            matches!(
                object,
                Object::ContentList(content)
                    if matches!(
                        content.objects().first(),
                        Some(Object::Conditional(conditional))
                            if conditional.kind() == ConditionalKind::If
                    )
            )
        });
        assert!(has_if);
    }

    #[test]
    fn parses_multiline_conditional_after_struct_literal_expression_support() {
        let output = parse(SourceInput::new("{ if score > 0:\nyes\n- else: no\n}"));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("expected story");
        let has_if = story.root_weave().content().iter().any(|object| {
            matches!(
                object,
                Object::ContentList(content)
                    if matches!(
                        content.objects().first(),
                        Some(Object::Conditional(conditional))
                            if conditional.kind() == ConditionalKind::If
                    )
            )
        });
        assert!(has_if);
    }

    #[test]
    fn parses_explicit_switch_conditional() {
        let output = parse(SourceInput::new(
            "{ switch score:\n- 0: zero\n- else: many\n}",
        ));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("expected story");
        let has_switch = story.root_weave().content().iter().any(|object| {
            matches!(
                object,
                Object::ContentList(content)
                    if matches!(
                        content.objects().first(),
                        Some(Object::Conditional(conditional))
                            if conditional.kind() == ConditionalKind::Switch
                    )
            )
        });
        assert!(has_switch);
    }

    #[test]
    fn switch_case_header_keeps_qualified_constant_separator() {
        let output = parse(SourceInput::new(
            "=== module game ===\n\
             VAR point: int = save::SAVE_POINT_VILLAGE\n\
             == main ==\n\
             { switch point:\n\
             - save::SAVE_POINT_VILLAGE:\n\
                 village\n\
             - else:\n\
                 other\n\
             }",
        ));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("expected story");
        let game = story.modules().first().expect("game module");
        let main = game.flows().first().expect("main flow");
        let Object::ContentList(content) = &main.weave().content()[0] else {
            panic!("expected conditional content list");
        };
        let Some(Object::Conditional(conditional)) = content.objects().first() else {
            panic!("expected switch");
        };
        let first_case = conditional.branches()[0]
            .own_condition()
            .expect("case condition");

        assert_eq!(first_case.to_source_string(), "save::SAVE_POINT_VILLAGE");
    }

    #[test]
    fn legacy_multiline_conditional_without_keyword_is_not_control_syntax() {
        let output = parse(SourceInput::new("{ score > 0:\nyes\n}"));

        assert!(output.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("expected closing `}` for inline expression before end of line")));
    }
}
