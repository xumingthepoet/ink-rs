use crate::{
    parsed::{Conditional, ConditionalBranch, ContentList, Expression, Object, Text},
    source::SourceLine,
};

use super::{
    gather, is_identifier_continue, parse_initial_expression, parser::Parser, scan, text,
    weave::weave_from_objects,
};

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

    let (condition_source, inline_content) = split_top_level_once(after_dash, ':')?;
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

fn parse_default_branch_content(trimmed: &str) -> Option<&str> {
    if trimmed.starts_with("->") {
        return None;
    }
    let content = trimmed.strip_prefix('-')?.trim_start();
    Some(content)
}

fn classify_branches(has_initial_condition: bool, branches: &mut [ConditionalBranchBuilder]) {
    if has_initial_condition {
        let mut earlier_branches_have_own_condition = false;
        let last_index = branches.len().saturating_sub(1);

        for (index, branch) in branches.iter_mut().enumerate() {
            let is_last = index == last_index;
            branch.is_true_branch = false;

            if branch.own_condition.is_some() {
                branch.is_else = false;
                earlier_branches_have_own_condition = true;
            } else if branch.explicit_else || (earlier_branches_have_own_condition && is_last) {
                branch.is_else = true;
            } else if index == 0 {
                branch.is_true_branch = true;
                branch.is_else = false;
            } else {
                branch.is_else = true;
            }
        }
    } else {
        let last_index = branches.len().saturating_sub(1);
        for (index, branch) in branches.iter_mut().enumerate() {
            branch.is_true_branch = false;
            if branch.explicit_else || (branch.own_condition.is_none() && index == last_index) {
                branch.is_else = true;
            }
        }
    }
}

fn non_empty_trimmed(source: &str) -> Option<&str> {
    let trimmed = source.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

fn split_top_level_once(source: &str, needle: char) -> Option<(&str, &str)> {
    scan::split_top_level_once_with_options(source, needle, scan::ScanOptions::inline_text())
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
                classify_branches(initial_condition.is_some(), &mut branches);
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
    use crate::{parsed::Object, source::SourceInput, syntax::parse};

    #[test]
    fn parses_multiline_conditional_into_content_list() {
        let output = parse(SourceInput::new("{ true:\n- yes\n}"));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("expected story");
        let has_conditional = story.root_weave().content().iter().any(|object| {
            matches!(
                object,
                Object::ContentList(content)
                    if matches!(content.objects().first(), Some(Object::Conditional(_)))
            )
        });
        assert!(has_conditional);
    }
}
