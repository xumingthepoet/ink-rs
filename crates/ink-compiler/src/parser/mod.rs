pub mod character_range;
pub mod character_set;
mod comment_eliminator;
pub mod expression;
pub mod string_parser;
mod whitespace;

pub use comment_eliminator::CommentEliminator;
pub use expression::ExpressionParser;
pub use string_parser::{Element, ParseSuccessStruct, StringParser, StringParserState};
pub use whitespace::{
    any_whitespace, end_of_file, end_of_line, multi_spaced, multiline_whitespace, newline, spaced,
    whitespace,
};

use std::collections::HashSet;
use std::sync::Arc;

use crate::{
    error::{Diagnostic, DiagnosticSeverity},
    parsed::{
        Conditional, ConditionalSingleBranch, ConstantDeclaration, ContentList, Divert,
        ExternalDeclaration, FlowLevel, Identifier, Knot, ListDefinition, ListElementDefinition,
        Object, ObjectKind, ObjectRef, Path, Return, Sequence, SequenceType, Stitch,
        Story as ParsedStory, Text, VariableAssignment,
    },
    results::{DefaultFileHandler, FileHandler, ParseResult},
};

#[derive(Debug)]
pub struct InkParser<'source> {
    input_string: String,
    source_filename: Option<&'source str>,
    file_handler: Option<Arc<dyn FileHandler>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssignmentOperator {
    Assign,
    Increment,
    Decrement,
}

#[derive(Debug, Default)]
struct ConditionalBranchBuilder {
    content: Vec<ObjectRef>,
    own_expression: Option<ObjectRef>,
    is_true_branch: bool,
    is_else: bool,
    matching_equality: bool,
}

impl<'source> InkParser<'source> {
    pub fn new(
        input_string: &'source str,
        source_filename: Option<&'source str>,
        file_handler: Option<Arc<dyn FileHandler>>,
    ) -> Self {
        let input_string = CommentEliminator::process(input_string).unwrap_or_default();

        Self {
            input_string,
            source_filename,
            file_handler,
        }
    }

    pub fn input_string(&self) -> &str {
        &self.input_string
    }

    pub fn source_filename(&self) -> Option<&'source str> {
        self.source_filename
    }

    pub fn file_handler(&self) -> Option<&dyn FileHandler> {
        self.file_handler.as_deref()
    }

    pub fn parse(&mut self) -> ParseResult {
        match self.parse_plain_text_story() {
            Ok(parsed_story) => ParseResult::success(parsed_story),
            Err(diagnostic) => ParseResult::failure(diagnostic),
        }
    }

    fn parse_plain_text_story(&self) -> std::result::Result<ParsedStory, Diagnostic> {
        let mut open_files = HashSet::new();
        self.parse_plain_text_story_with_open_files(&mut open_files)
    }

    fn parse_plain_text_story_with_open_files(
        &self,
        open_files: &mut HashSet<std::path::PathBuf>,
    ) -> std::result::Result<ParsedStory, Diagnostic> {
        if let Some((line, column, marker)) = self.find_unsupported_syntax() {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                self.source_filename.map(str::to_string),
                line,
                column,
                format!(
                    "InkParser currently supports plain text lines only; found unsupported syntax starting with {marker:?}"
                ),
            ));
        }

        let mut top_level_content = Vec::new();
        let mut current_flow: Option<ObjectRef> = None;
        let mut appended_flows = Vec::new();
        let source_filename = self.source_filename.map(str::to_string);
        let segments: Vec<&str> = self.input_string.split_inclusive('\n').collect();

        if segments.is_empty() {
            return Ok(ParsedStory::new(top_level_content, false));
        }

        let mut line_index = 0;
        while line_index < segments.len() {
            let segment = segments[line_index];
            let had_newline = segment.ends_with('\n');
            let line_text = segment.strip_suffix('\n').unwrap_or(segment);

            if let Some((mut include_content, include_flows)) = Self::parse_include_line(
                line_text,
                line_index + 1,
                source_filename.clone(),
                self.file_handler.clone(),
                open_files,
            )? {
                if let Some(parent) = current_flow.as_ref() {
                    for object in include_content.drain(..) {
                        Object::add_content(parent, object);
                    }
                } else {
                    top_level_content.append(&mut include_content);
                }

                appended_flows.extend(include_flows);
                line_index += 1;
                continue;
            }

            if let Some((logic, consumed_lines)) = Self::parse_brace_logic_block(
                &segments,
                line_index,
                line_index + 1,
                source_filename.clone(),
            )? {
                if let Some(parent) = current_flow.as_ref() {
                    Object::add_content(parent, logic);
                } else {
                    top_level_content.push(logic);
                }
                line_index += consumed_lines;
                continue;
            }

            if let Some(statement) =
                Self::parse_statement_line(line_text, line_index + 1, source_filename.clone())?
            {
                if let Some(parent) = current_flow.as_ref() {
                    Object::add_content(parent, statement);
                } else {
                    top_level_content.push(statement);
                }
                line_index += 1;
                continue;
            }

            if let Some(flow) =
                Self::parse_flow_header(line_text, line_index + 1, source_filename.clone())?
            {
                top_level_content.push(flow.clone());
                current_flow = Some(flow);
                line_index += 1;
                continue;
            }

            if let Some(divert) =
                Self::parse_simple_divert_line(line_text, line_index + 1, source_filename.clone())?
            {
                if let Some(parent) = current_flow.as_ref() {
                    Object::add_content(parent, divert);
                } else {
                    top_level_content.push(divert);
                }
                line_index += 1;
                continue;
            }

            if let Some(sequence) =
                Self::parse_sequence_line(line_text, line_index + 1, source_filename.clone())?
            {
                if let Some(parent) = current_flow.as_ref() {
                    Object::add_content(parent, sequence);
                } else {
                    top_level_content.push(sequence);
                }
                line_index += 1;
                continue;
            }

            if let Some(conditional) =
                Self::parse_conditional_line(line_text, line_index + 1, source_filename.clone())?
            {
                if let Some(parent) = current_flow.as_ref() {
                    Object::add_content(parent, conditional);
                } else {
                    top_level_content.push(conditional);
                }
                line_index += 1;
                continue;
            }

            let line = Self::build_content_line(line_text, had_newline);
            if let Some(parent) = current_flow.as_ref() {
                Object::add_content(parent, line);
            } else {
                top_level_content.push(line);
            }
            line_index += 1;
        }

        top_level_content.extend(appended_flows);

        Ok(ParsedStory::new(top_level_content, false))
    }

    fn parse_include_line(
        line_text: &str,
        line_number: usize,
        source_filename: Option<String>,
        file_handler: Option<Arc<dyn FileHandler>>,
        open_files: &mut HashSet<std::path::PathBuf>,
    ) -> std::result::Result<Option<(Vec<ObjectRef>, Vec<ObjectRef>)>, Diagnostic> {
        let trimmed_start = line_text.trim_start();
        let Some(remainder) = Self::strip_keyword(trimmed_start, "INCLUDE") else {
            return Ok(None);
        };

        let include_name = remainder.trim();
        if include_name.is_empty() {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line_number,
                1,
                "Expected filename for include statement",
            ));
        }

        let file_handler: Arc<dyn FileHandler> =
            file_handler.unwrap_or_else(|| Arc::new(DefaultFileHandler));
        let full_filename = file_handler.resolve_ink_filename(include_name);
        if !open_files.insert(full_filename.clone()) {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line_number,
                1,
                format!(
                    "Recursive INCLUDE detected: '{}' is already open.",
                    full_filename.display()
                ),
            ));
        }

        let included_result = match file_handler.load_ink_file_contents(&full_filename) {
            Ok(included_string) => {
                let included_filename = full_filename.to_string_lossy().into_owned();
                let included_parser = InkParser::new(
                    &included_string,
                    Some(included_filename.as_str()),
                    Some(file_handler.clone()),
                );
                included_parser.parse_plain_text_story_with_open_files(open_files)
            }
            Err(_) => Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line_number,
                1,
                format!("Failed to load: '{include_name}'"),
            )),
        };

        open_files.remove(&full_filename);

        let included_story = included_result?;
        let mut non_flow_content = Vec::new();
        let mut flows_from_other_files = Vec::new();

        for sub_story_obj in included_story.content() {
            if matches!(sub_story_obj.borrow().kind(), ObjectKind::Flow { .. }) {
                flows_from_other_files.push(sub_story_obj);
            } else {
                non_flow_content.push(sub_story_obj);
            }
        }

        if !non_flow_content.is_empty() {
            non_flow_content.push(Text::new("\n").object());
        }

        Ok(Some((non_flow_content, flows_from_other_files)))
    }

    fn build_content_line(line_text: &str, had_newline: bool) -> ObjectRef {
        let line = ContentList::new();
        if !line_text.is_empty() {
            line.add_content(Text::new(line_text).object());
        }
        line.trim_trailing_whitespace();

        if had_newline {
            line.add_content(Text::new("\n").object());
        }

        line.object()
    }

    fn parse_flow_header(
        line_text: &str,
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<Option<ObjectRef>, Diagnostic> {
        let trimmed_start = line_text.trim_start();
        if trimmed_start.is_empty() {
            return Ok(None);
        }

        let (flow_level, equals_count) = if trimmed_start.starts_with("==") {
            (
                FlowLevel::Knot,
                trimmed_start
                    .chars()
                    .take_while(|character| *character == '=')
                    .count(),
            )
        } else if trimmed_start.starts_with('=') {
            (FlowLevel::Stitch, 1)
        } else {
            return Ok(None);
        };

        if flow_level == FlowLevel::Stitch && trimmed_start.chars().nth(1) == Some('=') {
            return Ok(None);
        }

        let mut remainder = trimmed_start[equals_count..].trim();
        if remainder.ends_with('=') {
            remainder = remainder.trim_end_matches('=').trim_end();
        }

        let mut is_function = false;
        if let Some(function_remainder) = remainder.strip_prefix("function") {
            if function_remainder.is_empty()
                || function_remainder
                    .chars()
                    .next()
                    .map(char::is_whitespace)
                    .unwrap_or(false)
            {
                is_function = true;
                remainder = function_remainder.trim_start();
            }
        }

        let name = remainder
            .split_whitespace()
            .next()
            .filter(|name| !name.is_empty())
            .ok_or_else(|| {
                Diagnostic::new(
                    DiagnosticSeverity::Error,
                    source_filename.clone(),
                    line_number,
                    1,
                    "Expected knot or stitch name",
                )
            })?
            .to_string();

        let identifier = Identifier::new(name);
        let flow = match flow_level {
            FlowLevel::Knot => Knot::new(identifier, Vec::new(), Vec::new(), is_function).object(),
            FlowLevel::Stitch => {
                Stitch::new(identifier, Vec::new(), Vec::new(), is_function).object()
            }
            FlowLevel::Story | FlowLevel::WeavePoint => {
                return Err(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    source_filename,
                    line_number,
                    1,
                    "Expected knot or stitch name",
                ));
            }
        };

        Ok(Some(flow))
    }

    fn parse_simple_divert_line(
        line_text: &str,
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<Option<ObjectRef>, Diagnostic> {
        let trimmed_start = line_text.trim_start();
        if !trimmed_start.starts_with("->") {
            return Ok(None);
        }

        let is_tunnel = trimmed_start.starts_with("->->");
        let divert_prefix = if is_tunnel { "->->" } else { "->" };

        let remainder = trimmed_start[divert_prefix.len()..].trim_start();
        let line_indent = line_text.len().saturating_sub(trimmed_start.len()) + 1;

        if remainder.is_empty() {
            return Ok(Some(if is_tunnel {
                Divert::tunnel(None).object()
            } else {
                Divert::empty().object()
            }));
        }

        let mut target_tokens = remainder.split_whitespace();
        let target_text = target_tokens.next().unwrap_or_default();
        if target_text.is_empty() {
            return Ok(Some(if is_tunnel {
                Divert::tunnel(None).object()
            } else {
                Divert::empty().object()
            }));
        }

        if target_tokens.next().is_some() {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line_number,
                line_indent + 2,
                "Simple divert targets must be a single knot or stitch name",
            ));
        }

        let target = Self::parse_simple_divert_target(target_text).ok_or_else(|| {
            Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename.clone(),
                line_number,
                line_indent + 3,
                "Expected a valid divert target",
            )
        })?;

        Ok(Some(if is_tunnel {
            Divert::tunnel(Some(target)).object()
        } else {
            Divert::new(Some(target)).object()
        }))
    }

    fn parse_sequence_line(
        line_text: &str,
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<Option<ObjectRef>, Diagnostic> {
        let trimmed_start = line_text.trim_start();
        let (sequence_type, remainder) =
            if let Some(remainder) = trimmed_start.strip_prefix("once:") {
                (SequenceType::Once, remainder)
            } else if let Some(remainder) = trimmed_start.strip_prefix("cycle:") {
                (SequenceType::Cycle, remainder)
            } else if let Some(remainder) = trimmed_start.strip_prefix("shuffle:") {
                (SequenceType::Shuffle, remainder)
            } else if let Some(remainder) = trimmed_start.strip_prefix("stopping:") {
                (SequenceType::Stopping, remainder)
            } else {
                return Ok(None);
            };

        let mut branches = Vec::new();
        for branch_text in remainder.split('|') {
            let branch = ContentList::new();
            let branch_text = branch_text.trim();
            if !branch_text.is_empty() {
                branch.add_content(Text::new(branch_text).object());
            }
            branch.trim_trailing_whitespace();
            branches.push(branch);
        }

        if branches.len() < 2 {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line_number,
                line_text.len().saturating_sub(trimmed_start.len()) + 1,
                "Sequence lines must contain at least two branches separated by '|'",
            ));
        }

        Ok(Some(Sequence::new(branches, sequence_type).object()))
    }

    fn parse_conditional_line(
        line_text: &str,
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<Option<ObjectRef>, Diagnostic> {
        let trimmed_start = line_text.trim_start();
        if !trimmed_start.starts_with('{') || !trimmed_start.ends_with('}') {
            return Ok(None);
        }

        let inner = trimmed_start[1..trimmed_start.len() - 1].trim();
        let Some((condition_text, branches_text)) = inner.split_once(':') else {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line_number,
                line_text.len().saturating_sub(trimmed_start.len()) + 1,
                "Expected ':' in inline conditional",
            ));
        };

        let condition = Self::parse_expression_fragment(
            condition_text.trim(),
            line_number,
            source_filename.clone(),
        )?;

        let mut branches = Vec::new();
        let branch_texts = branches_text.split('|').map(str::trim).collect::<Vec<_>>();
        if branch_texts.is_empty() || branch_texts.len() > 2 {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line_number,
                line_text.len().saturating_sub(trimmed_start.len()) + 1,
                "Inline conditionals must have one or two branches separated by '|'",
            ));
        }

        for (branch_index, branch_text) in branch_texts.into_iter().enumerate() {
            let branch_content = ContentList::new();
            if !branch_text.is_empty() {
                branch_content.add_content(Text::new(branch_text).object());
            }
            let mut branch = ConditionalSingleBranch::new(vec![branch_content.object()]);
            branch.set_is_true_branch(branch_index == 0);
            branch.set_is_else(branch_index == 1);
            branch.set_is_inline(true);
            branches.push(branch);
        }

        Ok(Some(Conditional::new(Some(condition), branches).object()))
    }

    fn parse_brace_logic_block(
        segments: &[&str],
        start_index: usize,
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<Option<(ObjectRef, usize)>, Diagnostic> {
        let first_segment = segments[start_index];
        let line_text = first_segment.strip_suffix('\n').unwrap_or(first_segment);
        let trimmed_start = line_text.trim_start();

        if !trimmed_start.starts_with('{') || trimmed_start.ends_with('}') {
            return Ok(None);
        }

        let header_text = trimmed_start[1..].trim_start();
        let mut body_lines: Vec<(String, bool, usize)> = Vec::new();
        let mut consumed_lines = 1;
        let mut closed = false;

        while start_index + consumed_lines < segments.len() {
            let segment = segments[start_index + consumed_lines];
            let inner_line_text = segment.strip_suffix('\n').unwrap_or(segment);
            let trimmed = inner_line_text.trim_start();

            if trimmed == "}" {
                closed = true;
                consumed_lines += 1;
                break;
            }

            body_lines.push((
                inner_line_text.to_string(),
                segment.ends_with('\n'),
                line_number + consumed_lines,
            ));
            consumed_lines += 1;
        }

        if !closed {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line_number,
                1,
                "Expected closing '}' for brace logic block",
            ));
        }

        if let Some(sequence_type) = Self::parse_brace_sequence_header(header_text) {
            let sequence = Self::parse_brace_sequence_body(
                sequence_type,
                &body_lines,
                source_filename.clone(),
            )?;
            return Ok(Some((sequence, consumed_lines)));
        }

        let initial_condition = if header_text.trim().is_empty() {
            None
        } else {
            let Some(condition_text) = header_text.trim().strip_suffix(':') else {
                return Err(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    source_filename,
                    line_number,
                    1,
                    "Expected ':' after conditional header",
                ));
            };

            let condition_text = condition_text.trim();
            if condition_text.is_empty() {
                None
            } else {
                Some(Self::parse_expression_fragment(
                    condition_text,
                    line_number,
                    source_filename.clone(),
                )?)
            }
        };

        let conditional = Self::parse_brace_conditional_body(
            initial_condition,
            &body_lines,
            line_number,
            source_filename,
        )?;
        Ok(Some((conditional, consumed_lines)))
    }

    fn parse_brace_sequence_header(header_text: &str) -> Option<SequenceType> {
        let header_text = header_text.trim();
        let header_text = header_text.strip_suffix(':')?.trim();

        match header_text {
            "once" => Some(SequenceType::Once),
            "cycle" => Some(SequenceType::Cycle),
            "shuffle" => Some(SequenceType::Shuffle),
            "stopping" => Some(SequenceType::Stopping),
            _ => None,
        }
    }

    fn parse_brace_sequence_body(
        sequence_type: SequenceType,
        body_lines: &[(String, bool, usize)],
        source_filename: Option<String>,
    ) -> std::result::Result<ObjectRef, Diagnostic> {
        let mut branches = Vec::new();
        let mut current_branch: Option<Vec<ObjectRef>> = None;
        let mut saw_branch = false;

        for (line_text, had_newline, line_number) in body_lines {
            let trimmed_start = line_text.trim_start();
            if let Some(after_dash) = trimmed_start.strip_prefix('-') {
                if let Some(branch_content) = current_branch.take() {
                    branches.push(Self::build_content_list(branch_content));
                }

                saw_branch = true;
                let mut branch_content = Vec::new();
                let content_text = after_dash.trim_start();
                if !content_text.is_empty() {
                    branch_content.extend(Self::parse_branch_content_objects(
                        content_text,
                        *line_number,
                        source_filename.clone(),
                        false,
                    )?);
                }
                current_branch = Some(branch_content);
                continue;
            }

            if trimmed_start.is_empty() {
                if let Some(branch_content) = current_branch.as_mut() {
                    branch_content.extend(Self::parse_branch_content_objects(
                        line_text,
                        *line_number,
                        source_filename.clone(),
                        *had_newline,
                    )?);
                }
                continue;
            }

            if !saw_branch {
                return Err(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    source_filename.clone(),
                    *line_number,
                    1,
                    "Sequence branches must start with '-'",
                ));
            }

            if let Some(branch_content) = current_branch.as_mut() {
                branch_content.extend(Self::parse_branch_content_objects(
                    line_text,
                    *line_number,
                    source_filename.clone(),
                    *had_newline,
                )?);
            }
        }

        if let Some(branch_content) = current_branch.take() {
            branches.push(Self::build_content_list(branch_content));
        }

        if branches.len() < 2 {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                body_lines
                    .first()
                    .map(|(_, _, line_number)| *line_number)
                    .unwrap_or(1),
                1,
                "Sequence lines must contain at least two branches separated by '|'",
            ));
        }

        Ok(Sequence::new(branches, sequence_type).object())
    }

    fn parse_brace_conditional_body(
        initial_condition: Option<ObjectRef>,
        body_lines: &[(String, bool, usize)],
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<ObjectRef, Diagnostic> {
        let mut branches = Vec::new();
        let mut current_branch: Option<ConditionalBranchBuilder> = None;
        let mut saw_plain_branch = false;

        for (line_text, had_newline, line_number) in body_lines {
            let trimmed_start = line_text.trim_start();

            if let Some(after_dash) = trimmed_start.strip_prefix('-') {
                if let Some(branch) = current_branch.take() {
                    branches.push(Self::build_conditional_branch(branch));
                }

                let after_dash = after_dash.trim_start();
                let mut branch = ConditionalBranchBuilder::default();

                if after_dash.starts_with("else:") {
                    branch.is_else = true;
                    let content_text = after_dash["else:".len()..].trim_start();
                    if !content_text.is_empty() {
                        branch.content.extend(Self::parse_branch_content_objects(
                            content_text,
                            *line_number,
                            source_filename.clone(),
                            false,
                        )?);
                    }
                } else if let Some((condition_text, content_text)) = after_dash.split_once(':') {
                    let condition_text = condition_text.trim();
                    if condition_text.is_empty() {
                        if initial_condition.is_none() {
                            return Err(Diagnostic::new(
                                DiagnosticSeverity::Error,
                                source_filename.clone(),
                                *line_number,
                                1,
                                "Expected a condition before ':' in conditional branch",
                            ));
                        }
                        branch.is_true_branch = !saw_plain_branch;
                        saw_plain_branch = true;
                        if !content_text.trim().is_empty() {
                            branch.content.extend(Self::parse_branch_content_objects(
                                content_text.trim_start(),
                                *line_number,
                                source_filename.clone(),
                                false,
                            )?);
                        }
                    } else if let Some(condition) = Self::parse_expression_fragment(
                        condition_text,
                        *line_number,
                        source_filename.clone(),
                    )
                    .ok()
                    {
                        branch.own_expression = Some(condition);
                        branch.matching_equality = initial_condition.is_some();
                        if !content_text.trim().is_empty() {
                            branch.content.extend(Self::parse_branch_content_objects(
                                content_text.trim_start(),
                                *line_number,
                                source_filename.clone(),
                                false,
                            )?);
                        }
                    } else {
                        branch.is_true_branch = !saw_plain_branch;
                        saw_plain_branch = true;
                        branch.content.extend(Self::parse_branch_content_objects(
                            after_dash,
                            *line_number,
                            source_filename.clone(),
                            false,
                        )?);
                    }
                } else if initial_condition.is_some() {
                    branch.is_true_branch = !saw_plain_branch;
                    saw_plain_branch = true;
                    if !after_dash.is_empty() {
                        branch.content.extend(Self::parse_branch_content_objects(
                            after_dash,
                            *line_number,
                            source_filename.clone(),
                            false,
                        )?);
                    }
                } else {
                    return Err(Diagnostic::new(
                        DiagnosticSeverity::Error,
                        source_filename.clone(),
                        *line_number,
                        1,
                        "Expected a conditional branch expression or 'else:' after '-'",
                    ));
                }

                current_branch = Some(branch);
                continue;
            }

            if trimmed_start.is_empty() {
                if let Some(branch) = current_branch.as_mut() {
                    branch.content.extend(Self::parse_branch_content_objects(
                        line_text,
                        *line_number,
                        source_filename.clone(),
                        *had_newline,
                    )?);
                }
                continue;
            }

            if current_branch.is_none() {
                if initial_condition.is_some() {
                    let mut branch = ConditionalBranchBuilder::default();
                    branch.is_true_branch = true;
                    branch.content.extend(Self::parse_branch_content_objects(
                        line_text,
                        *line_number,
                        source_filename.clone(),
                        *had_newline,
                    )?);
                    current_branch = Some(branch);
                    continue;
                }

                return Err(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    source_filename.clone(),
                    *line_number,
                    1,
                    "Expected a conditional branch starting with '-'",
                ));
            }

            if let Some(branch) = current_branch.as_mut() {
                branch.content.extend(Self::parse_branch_content_objects(
                    line_text,
                    *line_number,
                    source_filename.clone(),
                    *had_newline,
                )?);
            }
        }

        if let Some(branch) = current_branch.take() {
            branches.push(Self::build_conditional_branch(branch));
        }

        if branches.is_empty() {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line_number,
                1,
                "Expected conditional branches inside brace block",
            ));
        }

        let conditional = Conditional::new(initial_condition, branches);
        Ok(conditional.object())
    }

    fn build_conditional_branch(branch: ConditionalBranchBuilder) -> ConditionalSingleBranch {
        let mut result = ConditionalSingleBranch::new(branch.content);
        result.set_is_true_branch(branch.is_true_branch);
        result.set_is_else(branch.is_else);
        result.set_is_inline(false);
        result.set_matching_equality(branch.matching_equality);
        result.set_own_expression(branch.own_expression);
        result
    }

    fn build_content_list(content: Vec<ObjectRef>) -> ContentList {
        let list = ContentList::new();
        for child in content {
            list.add_content(child);
        }
        list.trim_trailing_whitespace();
        list
    }

    fn parse_branch_content_objects(
        line_text: &str,
        line_number: usize,
        source_filename: Option<String>,
        had_newline: bool,
    ) -> std::result::Result<Vec<ObjectRef>, Diagnostic> {
        if let Some(statement) =
            Self::parse_statement_line(line_text, line_number, source_filename.clone())?
        {
            return Ok(vec![statement]);
        }

        if let Some(divert) =
            Self::parse_simple_divert_line(line_text, line_number, source_filename.clone())?
        {
            return Ok(vec![divert]);
        }

        if let Some(sequence) =
            Self::parse_sequence_line(line_text, line_number, source_filename.clone())?
        {
            return Ok(vec![sequence]);
        }

        if let Some(conditional) =
            Self::parse_conditional_line(line_text, line_number, source_filename)?
        {
            return Ok(vec![conditional]);
        }

        let mut result = Vec::new();
        let content_text = line_text.trim_start();
        if !content_text.is_empty() {
            result.push(Text::new(content_text).object());
        }
        if had_newline {
            result.push(Text::new("\n").object());
        }
        Ok(result)
    }

    fn parse_simple_divert_target(target_text: &str) -> Option<Path> {
        let mut components = Vec::new();

        for raw_component in target_text.split('.') {
            let component = raw_component.trim();
            if component.is_empty() {
                return None;
            }

            if !component
                .chars()
                .all(|character| character.is_alphanumeric() || character == '_')
            {
                return None;
            }

            components.push(Identifier::new(component));
        }

        if components.is_empty() {
            None
        } else {
            Some(Path::new(components))
        }
    }

    fn parse_statement_line(
        line_text: &str,
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<Option<ObjectRef>, Diagnostic> {
        let trimmed_start = line_text.trim_start();
        if trimmed_start.is_empty() {
            return Ok(None);
        }

        if let Some(statement) =
            Self::parse_logic_statement(trimmed_start, line_number, source_filename.clone())?
        {
            return Ok(Some(statement));
        }

        if let Some(statement) =
            Self::parse_variable_declaration(trimmed_start, line_number, source_filename.clone())?
        {
            return Ok(Some(statement));
        }

        if let Some(statement) =
            Self::parse_list_declaration(trimmed_start, line_number, source_filename.clone())?
        {
            return Ok(Some(statement));
        }

        if let Some(statement) =
            Self::parse_constant_declaration(trimmed_start, line_number, source_filename.clone())?
        {
            return Ok(Some(statement));
        }

        if let Some(statement) =
            Self::parse_external_declaration(trimmed_start, line_number, source_filename)?
        {
            return Ok(Some(statement));
        }

        Ok(None)
    }

    fn parse_logic_statement(
        line_text: &str,
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<Option<ObjectRef>, Diagnostic> {
        let Some(body) = line_text.strip_prefix('~') else {
            return Ok(None);
        };

        let body = body.trim_start();
        if body.is_empty() {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line_number,
                1,
                "Expected logic after '~'",
            ));
        }

        if let Some(statement) =
            Self::parse_return_statement(body, line_number, source_filename.clone())?
        {
            return Ok(Some(statement));
        }

        if let Some(statement) =
            Self::parse_temp_assignment(body, line_number, source_filename.clone())?
        {
            return Ok(Some(statement));
        }

        if let Some(statement) =
            Self::parse_assignment_or_expression(body, line_number, source_filename)?
        {
            return Ok(Some(statement));
        }

        Ok(None)
    }

    fn parse_return_statement(
        body: &str,
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<Option<ObjectRef>, Diagnostic> {
        let Some(remainder) = Self::strip_keyword(body, "return") else {
            return Ok(None);
        };

        let expression_text = remainder.trim_start();
        if expression_text.is_empty() {
            return Ok(Some(Return::new(None).object()));
        }

        let expression =
            Self::parse_expression_fragment(expression_text, line_number, source_filename)?;
        Ok(Some(Return::new(Some(expression)).object()))
    }

    fn parse_temp_assignment(
        body: &str,
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<Option<ObjectRef>, Diagnostic> {
        let Some(remainder) = Self::strip_keyword(body, "temp") else {
            return Ok(None);
        };

        let remainder = remainder.trim_start();
        let (identifier, after_identifier) =
            Self::parse_identifier_prefix(remainder).ok_or_else(|| {
                Diagnostic::new(
                    DiagnosticSeverity::Error,
                    source_filename.clone(),
                    line_number,
                    1,
                    "Expected temporary variable name",
                )
            })?;

        let after_identifier = after_identifier.trim_start();
        if after_identifier.is_empty() {
            return Ok(Some(
                VariableAssignment::new(identifier, None, false, true).object(),
            ));
        }

        if let Some((operator, rhs)) = Self::parse_assignment_operator(after_identifier) {
            let rhs = rhs.trim_start();
            if rhs.is_empty() {
                return Err(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    source_filename,
                    line_number,
                    1,
                    "Expected value after assignment operator",
                ));
            }

            let expression = Self::parse_expression_fragment(rhs, line_number, source_filename)?;
            return Ok(Some(match operator {
                AssignmentOperator::Assign => {
                    VariableAssignment::new(identifier, Some(expression), false, true).object()
                }
                AssignmentOperator::Increment => {
                    crate::parsed::IncDecExpression::new(identifier, true, Some(expression))
                        .object()
                }
                AssignmentOperator::Decrement => {
                    crate::parsed::IncDecExpression::new(identifier, false, Some(expression))
                        .object()
                }
            }));
        }

        Ok(Some(
            VariableAssignment::new(identifier, None, false, true).object(),
        ))
    }

    fn parse_assignment_or_expression(
        body: &str,
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<Option<ObjectRef>, Diagnostic> {
        if let Some((identifier, after_identifier)) = Self::parse_identifier_prefix(body) {
            if let Some((operator, rhs)) = Self::parse_assignment_operator(after_identifier) {
                let rhs = rhs.trim_start();
                if rhs.is_empty() {
                    return Err(Diagnostic::new(
                        DiagnosticSeverity::Error,
                        source_filename,
                        line_number,
                        1,
                        "Expected value after assignment operator",
                    ));
                }

                let expression =
                    Self::parse_expression_fragment(rhs, line_number, source_filename.clone())?;
                return Ok(Some(match operator {
                    AssignmentOperator::Assign => {
                        VariableAssignment::new(identifier, Some(expression), false, false).object()
                    }
                    AssignmentOperator::Increment => {
                        crate::parsed::IncDecExpression::new(identifier, true, Some(expression))
                            .object()
                    }
                    AssignmentOperator::Decrement => {
                        crate::parsed::IncDecExpression::new(identifier, false, Some(expression))
                            .object()
                    }
                }));
            }
        }

        let expression = Self::parse_expression_fragment(body, line_number, source_filename)?;
        Ok(Some(expression))
    }

    fn parse_variable_declaration(
        line_text: &str,
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<Option<ObjectRef>, Diagnostic> {
        let Some(remainder) = Self::strip_keyword(line_text, "VAR") else {
            return Ok(None);
        };

        let remainder = remainder.trim_start();
        let (identifier, after_identifier) =
            Self::parse_identifier_prefix(remainder).ok_or_else(|| {
                Diagnostic::new(
                    DiagnosticSeverity::Error,
                    source_filename.clone(),
                    line_number,
                    1,
                    "Expected variable name",
                )
            })?;

        let after_identifier = after_identifier.trim_start();
        let Some(rhs) = after_identifier.strip_prefix('=') else {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line_number,
                1,
                "Expected '=' after variable name",
            ));
        };

        let expression =
            Self::parse_expression_fragment(rhs.trim_start(), line_number, source_filename)?;
        Ok(Some(
            VariableAssignment::new(identifier, Some(expression), true, false).object(),
        ))
    }

    fn parse_list_declaration(
        line_text: &str,
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<Option<ObjectRef>, Diagnostic> {
        let Some(remainder) = Self::strip_keyword(line_text, "LIST") else {
            return Ok(None);
        };

        let remainder = remainder.trim_start();
        let (identifier, after_identifier) =
            Self::parse_identifier_prefix(remainder).ok_or_else(|| {
                Diagnostic::new(
                    DiagnosticSeverity::Error,
                    source_filename.clone(),
                    line_number,
                    1,
                    "Expected list name",
                )
            })?;

        let after_identifier = after_identifier.trim_start();
        let Some(rhs) = after_identifier.strip_prefix('=') else {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line_number,
                1,
                "Expected '=' after list name",
            ));
        };

        let list_definition = Self::parse_list_definition(
            identifier.clone(),
            rhs.trim_start(),
            line_number,
            source_filename,
        )?;
        Ok(Some(
            VariableAssignment::new_with_list_definition(identifier, list_definition, true)
                .object(),
        ))
    }

    fn parse_constant_declaration(
        line_text: &str,
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<Option<ObjectRef>, Diagnostic> {
        let Some(remainder) = Self::strip_keyword(line_text, "CONST") else {
            return Ok(None);
        };

        let remainder = remainder.trim_start();
        let (identifier, after_identifier) =
            Self::parse_identifier_prefix(remainder).ok_or_else(|| {
                Diagnostic::new(
                    DiagnosticSeverity::Error,
                    source_filename.clone(),
                    line_number,
                    1,
                    "Expected constant name",
                )
            })?;

        let after_identifier = after_identifier.trim_start();
        let Some(rhs) = after_identifier.strip_prefix('=') else {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line_number,
                1,
                "Expected '=' after constant name",
            ));
        };

        let expression =
            Self::parse_expression_fragment(rhs.trim_start(), line_number, source_filename)?;
        Ok(Some(
            ConstantDeclaration::new(identifier, Some(expression)).object(),
        ))
    }

    fn parse_list_definition(
        identifier: Identifier,
        definition_text: &str,
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<ListDefinition, Diagnostic> {
        let mut elements = Vec::new();
        let mut current_value = 1_i64;

        for raw_element in definition_text.split(',') {
            let element_text = raw_element.trim();
            if element_text.is_empty() {
                return Err(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    source_filename.clone(),
                    line_number,
                    1,
                    "Expected list item name",
                ));
            }

            let element = Self::parse_list_element_definition(
                element_text,
                current_value,
                line_number,
                source_filename.clone(),
            )?;
            let element_series_value = element.series_value().unwrap_or(current_value);
            current_value = element_series_value.saturating_add(1);
            elements.push(element);
        }

        if elements.is_empty() {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line_number,
                1,
                "Expected list item names",
            ));
        }

        Ok(ListDefinition::new(identifier, elements))
    }

    fn parse_list_element_definition(
        element_text: &str,
        current_value: i64,
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<ListElementDefinition, Diagnostic> {
        let mut remaining = element_text.trim_start();
        let in_initial_list = remaining.starts_with('(');
        if in_initial_list {
            remaining = remaining[1..].trim_start();
        }

        let (identifier, after_identifier) =
            Self::parse_identifier_prefix(remaining).ok_or_else(|| {
                Diagnostic::new(
                    DiagnosticSeverity::Error,
                    source_filename.clone(),
                    line_number,
                    1,
                    "Expected list item name",
                )
            })?;

        let mut tail = after_identifier.trim_start();
        let mut needs_close_paren = in_initial_list;

        if in_initial_list && tail.starts_with(')') {
            tail = tail[1..].trim_start();
            needs_close_paren = false;
        }

        let mut explicit_value = None;
        if tail.starts_with('=') {
            tail = tail[1..].trim_start();
            let (value, after_value) =
                Self::parse_signed_integer_prefix(tail).ok_or_else(|| {
                    Diagnostic::new(
                        DiagnosticSeverity::Error,
                        source_filename.clone(),
                        line_number,
                        1,
                        "Expected integer value for list item",
                    )
                })?;
            explicit_value = Some(value);
            tail = after_value.trim_start();

            if needs_close_paren && tail.starts_with(')') {
                tail = tail[1..].trim_start();
                needs_close_paren = false;
            }
        }

        if needs_close_paren {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line_number,
                1,
                "Expected closing ')' for list item",
            ));
        }

        if !tail.is_empty() {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line_number,
                1,
                "Unexpected trailing text in list item definition",
            ));
        }

        let series_value = explicit_value.unwrap_or(current_value);
        Ok(ListElementDefinition::new(
            identifier,
            in_initial_list,
            explicit_value,
            series_value,
        ))
    }

    fn parse_external_declaration(
        line_text: &str,
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<Option<ObjectRef>, Diagnostic> {
        let Some(remainder) = Self::strip_keyword(line_text, "EXTERNAL") else {
            return Ok(None);
        };

        let remainder = remainder.trim_start();
        let (identifier, after_identifier) =
            Self::parse_identifier_prefix(remainder).ok_or_else(|| {
                Diagnostic::new(
                    DiagnosticSeverity::Error,
                    source_filename.clone(),
                    line_number,
                    1,
                    "Expected external name",
                )
            })?;

        let mut argument_names = Vec::new();
        let after_identifier = after_identifier.trim_start();
        if after_identifier.starts_with('(') {
            let Some(close_paren_index) = after_identifier.find(')') else {
                return Err(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    source_filename,
                    line_number,
                    1,
                    "Expected closing ')' for external declaration",
                ));
            };

            let inside = &after_identifier[1..close_paren_index];
            for raw_arg in inside.split(',') {
                let arg = raw_arg.trim();
                if arg.is_empty() {
                    continue;
                }
                let (argument, tail) = Self::parse_identifier_prefix(arg).ok_or_else(|| {
                    Diagnostic::new(
                        DiagnosticSeverity::Error,
                        source_filename.clone(),
                        line_number,
                        1,
                        "Expected external argument name",
                    )
                })?;

                if !tail.trim().is_empty() {
                    return Err(Diagnostic::new(
                        DiagnosticSeverity::Error,
                        source_filename.clone(),
                        line_number,
                        1,
                        "Unexpected trailing text in external declaration",
                    ));
                }
                argument_names.push(argument.name);
            }
        }

        Ok(Some(
            ExternalDeclaration::new(identifier, argument_names).object(),
        ))
    }

    fn parse_expression_fragment(
        expression_text: &str,
        line_number: usize,
        source_filename: Option<String>,
    ) -> std::result::Result<ObjectRef, Diagnostic> {
        let mut parser = ExpressionParser::new(expression_text.to_string(), source_filename);
        parser.parse_expression().ok_or_else(|| {
            let diagnostic = parser.diagnostics().first().cloned().unwrap_or_else(|| {
                Diagnostic::new(
                    DiagnosticSeverity::Error,
                    None,
                    line_number,
                    1,
                    "Failed to parse expression",
                )
            });
            Diagnostic::new(
                diagnostic.severity,
                diagnostic.source_filename,
                line_number,
                diagnostic.column,
                diagnostic.message,
            )
        })
    }

    fn strip_keyword<'input>(input: &'input str, keyword: &str) -> Option<&'input str> {
        if !input.starts_with(keyword) {
            return None;
        }

        let remainder = &input[keyword.len()..];
        if remainder
            .chars()
            .next()
            .map(|character| character.is_alphanumeric() || character == '_')
            .unwrap_or(false)
        {
            return None;
        }

        Some(remainder)
    }

    fn parse_identifier_prefix(input: &str) -> Option<(Identifier, &str)> {
        let mut chars = input.chars();
        let first = chars.next()?;
        if !(first.is_alphabetic() || first == '_') {
            return None;
        }

        let mut identifier = String::new();
        identifier.push(first);

        let mut consumed = first.len_utf8();
        for character in chars {
            if character.is_alphanumeric() || character == '_' {
                identifier.push(character);
                consumed += character.len_utf8();
            } else {
                break;
            }
        }

        Some((Identifier::new(identifier), &input[consumed..]))
    }

    fn parse_signed_integer_prefix(input: &str) -> Option<(i64, &str)> {
        let trimmed = input.trim_start();
        let leading_whitespace = input.len() - trimmed.len();
        let remainder = &input[leading_whitespace..];

        let mut chars = remainder.chars();
        let mut token = String::new();

        if let Some(first) = chars.next() {
            if first == '+' || first == '-' {
                token.push(first);
            } else if first.is_ascii_digit() {
                token.push(first);
            } else {
                return None;
            }
        } else {
            return None;
        }

        let mut consumed = token.len();
        for character in chars {
            if character.is_ascii_digit() {
                token.push(character);
                consumed += character.len_utf8();
            } else {
                break;
            }
        }

        if token == "+" || token == "-" {
            return None;
        }

        let value = token.parse::<i64>().ok()?;
        Some((value, &remainder[consumed..]))
    }

    fn parse_assignment_operator(input: &str) -> Option<(AssignmentOperator, &str)> {
        let trimmed = input.trim_start();
        let leading_whitespace = input.len() - trimmed.len();
        let remainder = &input[leading_whitespace..];

        if let Some(after) = remainder.strip_prefix("+=") {
            return Some((AssignmentOperator::Increment, after));
        }

        if let Some(after) = remainder.strip_prefix("-=") {
            return Some((AssignmentOperator::Decrement, after));
        }

        if let Some(after) = remainder.strip_prefix('=') {
            if !after.starts_with('=') {
                return Some((AssignmentOperator::Assign, after));
            }
        }

        None
    }

    fn find_unsupported_syntax(&self) -> Option<(usize, usize, &'static str)> {
        let mut in_brace_block = false;

        for (line_index, line) in self.input_string.lines().enumerate() {
            let trimmed = line.trim_start();
            let column = line.len().saturating_sub(trimmed.len()) + 1;

            if in_brace_block {
                if trimmed == "}" {
                    in_brace_block = false;
                }
                continue;
            }

            if trimmed.starts_with('{') && !trimmed.ends_with('}') {
                in_brace_block = true;
                continue;
            }

            for marker in ["*", "-", "#", "{", "}"] {
                if marker == "-" && trimmed.starts_with("->") {
                    continue;
                }
                if marker == "{" && trimmed.starts_with('{') && trimmed.ends_with('}') {
                    continue;
                }
                if trimmed.starts_with(marker) {
                    return Some((line_index + 1, column, marker));
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        fs, io,
        path::{Path, PathBuf},
        sync::Arc,
    };

    use super::{CommentEliminator, InkParser};
    use crate::parsed::{
        ExpressionKind, NumberValue, ObjectKind, ObjectRef, SequenceType, Story as ParsedStory,
    };
    use crate::results::FileHandler;

    #[derive(Debug)]
    struct IncludeFileHandler {
        files: HashMap<PathBuf, String>,
    }

    impl IncludeFileHandler {
        fn new(files: HashMap<PathBuf, String>) -> Self {
            Self { files }
        }
    }

    impl FileHandler for IncludeFileHandler {
        fn resolve_ink_filename(&self, include_name: &str) -> PathBuf {
            workspace_root().join("ink-csharp/tests").join(include_name)
        }

        fn load_ink_file_contents(&self, full_filename: &Path) -> io::Result<String> {
            self.files.get(full_filename).cloned().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("missing include: {}", full_filename.display()),
                )
            })
        }
    }

    fn workspace_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf()
    }

    fn load_workspace_text(relative_path: &str) -> String {
        fs::read_to_string(workspace_root().join(relative_path))
            .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
    }

    fn render_story(story: &ParsedStory) -> String {
        let mut lines = vec!["Story".to_string()];

        for child in story.content() {
            render_object(&child, 1, &mut lines);
        }

        lines.join("\n")
    }

    fn render_object(object: &ObjectRef, indent: usize, lines: &mut Vec<String>) {
        let padding = "  ".repeat(indent);
        let borrowed = object.borrow();

        match borrowed.kind() {
            ObjectKind::ContentList { .. } => {
                lines.push(format!("{padding}ContentList"));
                for child in borrowed.content() {
                    render_object(child, indent + 1, lines);
                }
            }
            ObjectKind::Text { text } => {
                lines.push(format!("{padding}Text({text:?})"));
            }
            ObjectKind::Divert {
                target,
                is_empty,
                is_tunnel,
                is_thread,
            } => {
                let target = target
                    .as_ref()
                    .map(|path| path.to_string())
                    .unwrap_or_else(|| "->".to_string());
                lines.push(format!(
                    "{padding}Divert(target={target:?}, empty={is_empty}, tunnel={is_tunnel}, thread={is_thread})"
                ));
            }
            ObjectKind::Flow {
                flow_level,
                name,
                is_function,
            } => {
                lines.push(format!(
                    "{padding}Flow(level={flow_level:?}, name={:?}, function={is_function})",
                    name.as_deref().unwrap_or("<unnamed>")
                ));
                for child in borrowed.content() {
                    render_object(child, indent + 1, lines);
                }
            }
            ObjectKind::AuthorWarning { warning_message } => {
                lines.push(format!("{padding}AuthorWarning({warning_message:?})"));
            }
            ObjectKind::Tag {
                is_start,
                in_choice,
            } => {
                lines.push(format!(
                    "{padding}Tag(start={is_start}, in_choice={in_choice})"
                ));
            }
            ObjectKind::VariableAssignment {
                identifier,
                is_global_declaration,
                is_new_temporary_declaration,
            } => {
                lines.push(format!(
                    "{padding}VariableAssignment(name={:?}, global={is_global_declaration}, temp={is_new_temporary_declaration})",
                    identifier.name
                ));
                for child in borrowed.content() {
                    render_object(child, indent + 1, lines);
                }
            }
            ObjectKind::ConstantDeclaration { identifier } => {
                lines.push(format!(
                    "{padding}ConstantDeclaration(name={:?})",
                    identifier.name
                ));
                for child in borrowed.content() {
                    render_object(child, indent + 1, lines);
                }
            }
            ObjectKind::ExternalDeclaration {
                identifier,
                argument_names,
            } => {
                lines.push(format!(
                    "{padding}ExternalDeclaration(name={:?}, args={argument_names:?})",
                    identifier.name
                ));
            }
            ObjectKind::Return => {
                lines.push(format!("{padding}Return"));
                for child in borrowed.content() {
                    render_object(child, indent + 1, lines);
                }
            }
            ObjectKind::ListDefinition { identifier } => {
                lines.push(format!(
                    "{padding}ListDefinition(name={:?})",
                    identifier.name
                ));
                for child in borrowed.content() {
                    render_object(child, indent + 1, lines);
                }
            }
            ObjectKind::ListElementDefinition {
                identifier,
                explicit_value,
                series_value,
                in_initial_list,
            } => {
                lines.push(format!(
                    "{padding}ListElementDefinition(name={:?}, explicit={explicit_value:?}, series={series_value}, initial={in_initial_list})",
                    identifier.name
                ));
            }
            ObjectKind::Sequence { sequence_type } => {
                lines.push(format!("{padding}Sequence(type={sequence_type})"));
                for child in borrowed.content() {
                    render_object(child, indent + 1, lines);
                }
            }
            ObjectKind::Conditional => {
                lines.push(format!("{padding}Conditional"));
                for child in borrowed.content() {
                    render_object(child, indent + 1, lines);
                }
            }
            ObjectKind::ConditionalSingleBranch {
                is_true_branch,
                is_else,
                is_inline,
            } => {
                lines.push(format!(
                    "{padding}ConditionalBranch(true={is_true_branch}, else={is_else}, inline={is_inline})"
                ));
                for child in borrowed.content() {
                    render_object(child, indent + 1, lines);
                }
            }
            ObjectKind::Expression { .. } => {
                lines.push(format!("{padding}{}", render_expression(object)));
            }
            other => lines.push(format!("{padding}{other:?}")),
        }
    }

    fn render_expression(object: &ObjectRef) -> String {
        let borrowed = object.borrow();
        match borrowed.kind() {
            ObjectKind::Expression {
                kind: ExpressionKind::Number(value),
            } => format!("Number({value})"),
            ObjectKind::Expression {
                kind: ExpressionKind::StringExpression,
            } => format!("String({:?})", borrowed.content()),
            ObjectKind::Expression {
                kind: ExpressionKind::VariableReference { path, .. },
            } => format!(
                "VariableReference({})",
                path.iter()
                    .map(|identifier| identifier.name.clone())
                    .collect::<Vec<_>>()
                    .join(".")
            ),
            ObjectKind::Expression {
                kind: ExpressionKind::FunctionCall { function_name, .. },
            } => format!(
                "FunctionCall({}, args={})",
                function_name.name,
                borrowed.content().len()
            ),
            ObjectKind::Expression {
                kind: ExpressionKind::DivertTarget,
            } => {
                let target = borrowed
                    .content()
                    .first()
                    .and_then(|child| match child.borrow().kind() {
                        crate::parsed::ObjectKind::Divert { target, .. } => Some(
                            target
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or_else(|| "->".to_string()),
                        ),
                        _ => None,
                    })
                    .unwrap_or_else(|| "<missing>".to_string());
                format!("DivertTarget({target})")
            }
            ObjectKind::Expression {
                kind: ExpressionKind::List { item_identifiers },
            } => format!(
                "List({})",
                item_identifiers
                    .iter()
                    .map(|identifier| identifier.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ObjectKind::Expression {
                kind: ExpressionKind::Binary { op_name },
            } => format!(
                "Binary({op_name}, {}, {})",
                borrowed
                    .content()
                    .first()
                    .map(render_expression)
                    .unwrap_or_else(|| "<missing>".to_string()),
                borrowed
                    .content()
                    .get(1)
                    .map(render_expression)
                    .unwrap_or_else(|| "<missing>".to_string())
            ),
            ObjectKind::Expression {
                kind: ExpressionKind::Unary { op },
            } => format!(
                "Unary({op}, {})",
                borrowed
                    .content()
                    .first()
                    .map(render_expression)
                    .unwrap_or_else(|| "<missing>".to_string())
            ),
            ObjectKind::Expression {
                kind: ExpressionKind::IncDec { identifier, is_inc },
            } => format!("IncDec({}, inc={is_inc})", identifier.name),
            ObjectKind::Expression {
                kind: ExpressionKind::MultipleCondition,
            } => format!(
                "MultipleCondition({})",
                borrowed
                    .content()
                    .iter()
                    .map(render_expression)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            _ => "<Expression>".to_string(),
        }
    }

    #[test]
    fn ink_parser_preprocesses_comments_and_newlines() {
        let parser = InkParser::new("line1 // comment\r\nline2/*x\n y*/line3", None, None);

        assert_eq!(parser.input_string(), "line1 \nline2\nline3");
        assert_eq!(
            CommentEliminator::process("line1 // comment\r\nline2/*x\n y*/line3"),
            Some("line1 \nline2\nline3".to_string())
        );
    }

    #[test]
    fn ink_parser_parses_plain_text_into_content_nodes() {
        let mut parser = InkParser::new("Hello world\nSecond line", Some("story.ink"), None);
        let result = parser.parse();

        assert!(result.diagnostics.is_empty());
        let story = result.parsed_story.expect("expected parsed story");
        let content = story.content();

        assert_eq!(content.len(), 2);

        let first_line = content[0].borrow();
        assert!(matches!(first_line.kind(), ObjectKind::ContentList { .. }));
        assert_eq!(first_line.content().len(), 2);
        assert!(matches!(
            first_line.content()[0].borrow().kind(),
            ObjectKind::Text { text } if text == "Hello world"
        ));
        assert!(matches!(
            first_line.content()[1].borrow().kind(),
            ObjectKind::Text { text } if text == "\n"
        ));

        let second_line = content[1].borrow();
        assert!(matches!(second_line.kind(), ObjectKind::ContentList { .. }));
        assert_eq!(second_line.content().len(), 1);
        assert!(matches!(
            second_line.content()[0].borrow().kind(),
            ObjectKind::Text { text } if text == "Second line"
        ));
    }

    #[test]
    fn ink_parser_reports_unsupported_structural_syntax() {
        let mut parser = InkParser::new("* choice", Some("story.ink"), None);
        let result = parser.parse();

        assert!(result.parsed_story.is_none());
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].severity,
            crate::error::DiagnosticSeverity::Error
        );
        assert_eq!(
            result.diagnostics[0].source_filename.as_deref(),
            Some("story.ink")
        );
        assert_eq!(result.diagnostics[0].line, 1);
        assert_eq!(result.diagnostics[0].column, 1);
    }

    #[test]
    fn ink_parser_parses_variables_and_external_statements() {
        let mut parser = InkParser::new(
            "VAR score = 5\nCONST pi = 3.14\nEXTERNAL print(message)\n~ temp tmp = score\n~ score += 1\n~ score",
            Some("story.ink"),
            None,
        );
        let result = parser.parse();

        assert!(result.diagnostics.is_empty());
        let story = result.parsed_story.expect("expected parsed story");
        let content = story.content();

        assert_eq!(content.len(), 6);
        assert!(matches!(
            content[0].borrow().kind(),
            ObjectKind::VariableAssignment {
                is_global_declaration: true,
                is_new_temporary_declaration: false,
                ..
            }
        ));
        assert!(matches!(
            content[1].borrow().kind(),
            ObjectKind::ConstantDeclaration { .. }
        ));
        assert!(matches!(
            content[2].borrow().kind(),
            ObjectKind::ExternalDeclaration { .. }
        ));
        assert!(matches!(
            content[3].borrow().kind(),
            ObjectKind::VariableAssignment {
                is_global_declaration: false,
                is_new_temporary_declaration: true,
                ..
            }
        ));
        assert!(matches!(
            content[4].borrow().kind(),
            ObjectKind::Expression {
                kind: crate::parsed::ExpressionKind::IncDec { is_inc: true, .. }
            }
        ));
        assert!(matches!(
            content[5].borrow().kind(),
            ObjectKind::Expression {
                kind: crate::parsed::ExpressionKind::VariableReference { .. }
            }
        ));
    }

    #[test]
    fn ink_parser_parses_return_statements() {
        let mut parser =
            InkParser::new("== start ==\n~ return\n~ return 5", Some("story.ink"), None);
        let result = parser.parse();

        assert!(result.diagnostics.is_empty());
        let story = result.parsed_story.expect("expected parsed story");
        let story_content = story.content();
        let flow = story_content[0].borrow();
        assert!(matches!(
            flow.kind(),
            ObjectKind::Flow {
                flow_level: crate::parsed::FlowLevel::Knot,
                name,
                is_function: false,
            } if name.as_deref() == Some("start")
        ));

        let flow_content = flow.content().to_vec();
        assert_eq!(flow_content.len(), 2);
        assert!(matches!(
            flow_content[0].borrow().kind(),
            ObjectKind::Return
        ));
        assert!(matches!(
            flow_content[1].borrow().kind(),
            ObjectKind::Return
        ));
        assert!(matches!(
            flow_content[1].borrow().content()[0].borrow().kind(),
            ObjectKind::Expression { .. }
        ));
    }

    #[test]
    fn ink_parser_parses_lists_definitions_and_list_values() {
        let mut parser = InkParser::new(
            "LIST terrain = (forest), hill = 4, (beach)\nVAR chosen = (forest, terrain.hill)",
            Some("story.ink"),
            None,
        );
        let result = parser.parse();

        assert!(result.diagnostics.is_empty());
        let story = result.parsed_story.expect("expected parsed story");
        let content = story.content();

        assert_eq!(content.len(), 2);

        let list_assignment = content[0].borrow();
        assert!(matches!(
            list_assignment.kind(),
            ObjectKind::VariableAssignment {
                is_global_declaration: true,
                is_new_temporary_declaration: false,
                ..
            }
        ));
        let list_definition = list_assignment
            .content()
            .first()
            .expect("expected list definition child")
            .borrow();
        assert!(matches!(
            list_definition.kind(),
            ObjectKind::ListDefinition { identifier }
                if identifier.name == "terrain"
        ));
        assert_eq!(list_definition.content().len(), 3);
        assert!(matches!(
            list_definition.content()[0].borrow().kind(),
            ObjectKind::ListElementDefinition {
                identifier,
                explicit_value,
                series_value,
                in_initial_list,
            } if identifier.name == "forest"
                && explicit_value.is_none()
                && *series_value == 1
                && *in_initial_list
        ));
        assert!(matches!(
            list_definition.content()[1].borrow().kind(),
            ObjectKind::ListElementDefinition {
                identifier,
                explicit_value,
                series_value,
                in_initial_list,
            } if identifier.name == "hill"
                && *explicit_value == Some(4)
                && *series_value == 4
                && !*in_initial_list
        ));
        assert!(matches!(
            list_definition.content()[2].borrow().kind(),
            ObjectKind::ListElementDefinition {
                identifier,
                explicit_value,
                series_value,
                in_initial_list,
            } if identifier.name == "beach"
                && explicit_value.is_none()
                && *series_value == 5
                && *in_initial_list
        ));

        let chosen_assignment = content[1].borrow();
        assert!(matches!(
            chosen_assignment.kind(),
            ObjectKind::VariableAssignment {
                is_global_declaration: true,
                is_new_temporary_declaration: false,
                ..
            }
        ));
        assert!(matches!(
            chosen_assignment.content()[0].borrow().kind(),
            ObjectKind::Expression {
                kind: crate::parsed::ExpressionKind::List { item_identifiers }
            } if item_identifiers.len() == 2
        ));
    }

    #[test]
    fn ink_parser_parses_basic_knots_and_stitches() {
        let mut parser = InkParser::new(
            "Intro line\n== start ==\nKnot body\n= stitch\nStitch body",
            Some("story.ink"),
            None,
        );
        let result = parser.parse();

        assert!(result.diagnostics.is_empty());
        let story = result.parsed_story.expect("expected parsed story");
        let content = story.content();

        assert_eq!(content.len(), 3);
        assert!(matches!(
            content[0].borrow().kind(),
            ObjectKind::ContentList { .. }
        ));
        assert!(matches!(
            content[1].borrow().kind(),
            ObjectKind::Flow {
                flow_level: crate::parsed::FlowLevel::Knot,
                name,
                is_function: false,
            } if name.as_deref() == Some("start")
        ));
        assert!(matches!(
            content[2].borrow().kind(),
            ObjectKind::Flow {
                flow_level: crate::parsed::FlowLevel::Stitch,
                name,
                is_function: false,
            } if name.as_deref() == Some("stitch")
        ));

        let knot_content = content[1].borrow().content().to_vec();
        assert_eq!(knot_content.len(), 1);
        assert!(matches!(
            knot_content[0].borrow().kind(),
            ObjectKind::ContentList { .. }
        ));
        assert!(matches!(
            knot_content[0].borrow().content()[0].borrow().kind(),
            ObjectKind::Text { text } if text == "Knot body"
        ));
    }

    #[test]
    fn ink_parser_parses_simple_diverts() {
        let mut parser =
            InkParser::new("== start ==\n-> ending\nFlow body", Some("story.ink"), None);
        let result = parser.parse();

        assert!(result.diagnostics.is_empty());
        let story = result.parsed_story.expect("expected parsed story");
        let content = story.content();

        assert_eq!(content.len(), 1);

        let flow = content[0].borrow();
        assert!(matches!(
            flow.kind(),
            ObjectKind::Flow {
                flow_level: crate::parsed::FlowLevel::Knot,
                name,
                is_function: false,
            } if name.as_deref() == Some("start")
        ));

        let flow_content = flow.content().to_vec();
        assert_eq!(flow_content.len(), 2);
        assert!(matches!(
            flow_content[0].borrow().kind(),
            ObjectKind::Divert {
                target,
                is_empty: false,
                is_tunnel: false,
                is_thread: false,
            } if target.as_ref().and_then(|path| path.first_component()) == Some("ending")
        ));
        assert!(matches!(
            flow_content[1].borrow().kind(),
            ObjectKind::ContentList { .. }
        ));
        assert!(matches!(
            flow_content[1].borrow().content()[0].borrow().kind(),
            ObjectKind::Text { text } if text == "Flow body"
        ));
    }

    #[test]
    fn ink_parser_parses_empty_diverts() {
        let mut parser = InkParser::new("->", Some("story.ink"), None);
        let result = parser.parse();

        assert!(result.diagnostics.is_empty());
        let story = result.parsed_story.expect("expected parsed story");
        let content = story.content();

        assert_eq!(content.len(), 1);
        assert!(matches!(
            content[0].borrow().kind(),
            ObjectKind::Divert { is_empty: true, .. }
        ));
    }

    #[test]
    fn ink_parser_parses_tunnel_diverts() {
        let mut parser = InkParser::new("->-> target", Some("story.ink"), None);
        let result = parser.parse();

        assert!(result.diagnostics.is_empty());
        let story = result.parsed_story.expect("expected parsed story");
        let content = story.content();

        assert_eq!(content.len(), 1);
        assert!(matches!(
            content[0].borrow().kind(),
            ObjectKind::Divert {
                target,
                is_empty: false,
                is_tunnel: true,
                is_thread: false,
            } if target.as_ref().and_then(|path| path.first_component()) == Some("target")
        ));
    }

    #[test]
    fn ink_parser_parses_sequences() {
        let mut parser = InkParser::new("once: first | second", Some("story.ink"), None);
        let result = parser.parse();

        assert!(result.diagnostics.is_empty());
        let story = result.parsed_story.expect("expected parsed story");
        let content = story.content();

        assert_eq!(content.len(), 1);
        assert!(matches!(
            content[0].borrow().kind(),
            ObjectKind::Sequence {
                sequence_type: SequenceType::Once,
            }
        ));

        let branches = content[0].borrow().content().to_vec();
        assert_eq!(branches.len(), 2);
        assert!(matches!(
            branches[0].borrow().kind(),
            ObjectKind::ContentList { .. }
        ));
        assert!(matches!(
            branches[0].borrow().content()[0].borrow().kind(),
            ObjectKind::Text { text } if text == "first"
        ));
        assert!(matches!(
            branches[1].borrow().kind(),
            ObjectKind::ContentList { .. }
        ));
        assert!(matches!(
            branches[1].borrow().content()[0].borrow().kind(),
            ObjectKind::Text { text } if text == "second"
        ));
    }

    #[test]
    fn ink_parser_parses_inline_conditionals() {
        let mut parser = InkParser::new("{ x > 3: yes | no }", Some("story.ink"), None);
        let result = parser.parse();

        assert!(result.diagnostics.is_empty());
        let story = result.parsed_story.expect("expected parsed story");
        let content = story.content();

        assert_eq!(content.len(), 1);
        assert!(matches!(
            content[0].borrow().kind(),
            ObjectKind::Conditional
        ));

        let conditional_content = content[0].borrow().content().to_vec();
        assert_eq!(conditional_content.len(), 3);
        assert!(matches!(
            conditional_content[0].borrow().kind(),
            ObjectKind::Expression { .. }
        ));
        assert!(matches!(
            conditional_content[1].borrow().kind(),
            ObjectKind::ConditionalSingleBranch {
                is_true_branch: true,
                is_else: false,
                is_inline: true,
            }
        ));
        assert!(matches!(
            conditional_content[2].borrow().kind(),
            ObjectKind::ConditionalSingleBranch {
                is_true_branch: false,
                is_else: true,
                is_inline: true,
            }
        ));
    }

    #[test]
    fn ink_parser_parses_multiline_conditionals() {
        let mut parser = InkParser::new(
            "{ x == 4:\n  The main clause\n- else:\n  other\n}",
            Some("story.ink"),
            None,
        );
        let result = parser.parse();

        assert!(result.diagnostics.is_empty());
        let story = result.parsed_story.expect("expected parsed story");
        let content = story.content();

        assert_eq!(content.len(), 1);
        assert!(matches!(
            content[0].borrow().kind(),
            ObjectKind::Conditional
        ));

        let conditional_content = content[0].borrow().content().to_vec();
        assert_eq!(conditional_content.len(), 3);
        assert!(matches!(
            conditional_content[0].borrow().kind(),
            ObjectKind::Expression {
                kind: ExpressionKind::Binary { .. }
            }
        ));
        assert!(matches!(
            conditional_content[1].borrow().kind(),
            ObjectKind::ConditionalSingleBranch {
                is_true_branch: true,
                is_else: false,
                is_inline: false,
            }
        ));
        assert!(matches!(
            conditional_content[2].borrow().kind(),
            ObjectKind::ConditionalSingleBranch {
                is_true_branch: false,
                is_else: true,
                is_inline: false,
            }
        ));
    }

    #[test]
    fn ink_parser_parses_multiline_sequences() {
        let mut parser = InkParser::new(
            "{once:\n  - first\n  -\n  - second\n}",
            Some("story.ink"),
            None,
        );
        let result = parser.parse();

        assert!(result.diagnostics.is_empty());
        let story = result.parsed_story.expect("expected parsed story");
        let content = story.content();

        assert_eq!(content.len(), 1);
        assert!(matches!(
            content[0].borrow().kind(),
            ObjectKind::Sequence {
                sequence_type: SequenceType::Once,
            }
        ));

        let branches = content[0].borrow().content().to_vec();
        assert_eq!(branches.len(), 3);
        assert!(matches!(
            branches[0].borrow().kind(),
            ObjectKind::ContentList { .. }
        ));
        assert!(matches!(
            branches[0].borrow().content()[0].borrow().kind(),
            ObjectKind::Text { text } if text == "first"
        ));
        assert!(matches!(
            branches[1].borrow().kind(),
            ObjectKind::ContentList { .. }
        ));
        assert!(branches[1].borrow().content().is_empty());
        assert!(matches!(
            branches[2].borrow().kind(),
            ObjectKind::ContentList { .. }
        ));
        assert!(matches!(
            branches[2].borrow().content()[0].borrow().kind(),
            ObjectKind::Text { text } if text == "second"
        ));
    }

    #[test]
    fn ink_parser_parses_multiline_switch_conditionals() {
        let mut parser = InkParser::new(
            "{ 3:\n    - 3:\n    - 4:\n        txt\n}",
            Some("story.ink"),
            None,
        );
        let result = parser.parse();

        assert!(result.diagnostics.is_empty());
        let story = result.parsed_story.expect("expected parsed story");
        let content = story.content();

        assert_eq!(content.len(), 1);
        assert!(matches!(
            content[0].borrow().kind(),
            ObjectKind::Conditional
        ));

        let conditional_content = content[0].borrow().content().to_vec();
        assert_eq!(conditional_content.len(), 3);
        assert!(matches!(
            conditional_content[0].borrow().kind(),
            ObjectKind::Expression {
                kind: ExpressionKind::Number(NumberValue::Int(3))
            }
        ));
        assert!(matches!(
            conditional_content[1].borrow().kind(),
            ObjectKind::ConditionalSingleBranch {
                is_true_branch: false,
                is_else: false,
                is_inline: false,
            }
        ));
        assert!(matches!(
            conditional_content[1].borrow().content()[0].borrow().kind(),
            ObjectKind::Expression {
                kind: ExpressionKind::Number(NumberValue::Int(3))
            }
        ));
        assert!(matches!(
            conditional_content[2].borrow().kind(),
            ObjectKind::ConditionalSingleBranch {
                is_true_branch: false,
                is_else: false,
                is_inline: false,
            }
        ));
        assert!(matches!(
            conditional_content[2].borrow().content()[0].borrow().kind(),
            ObjectKind::Expression {
                kind: ExpressionKind::Number(NumberValue::Int(4))
            }
        ));
    }

    #[test]
    fn ink_parser_golden_cases_for_minimal_snippets() {
        let cases = [
            (
                "plain_text",
                "Hello world",
                "Story\n  ContentList\n    Text(\"Hello world\")",
            ),
            (
                "knot_with_body",
                "== start ==\nHello",
                "Story\n  Flow(level=Knot, name=\"start\", function=false)\n    ContentList\n      Text(\"Hello\")",
            ),
            (
                "simple_divert",
                "-> ending",
                "Story\n  Divert(target=\"-> ending\", empty=false, tunnel=false, thread=false)",
            ),
            (
                "tunnel_divert",
                "->-> ending",
                "Story\n  Divert(target=\"-> ending\", empty=false, tunnel=true, thread=false)",
            ),
            (
                "simple_glue",
                "Some <>\ncontent <>\nwith glue.",
                "Story\n  ContentList\n    Text(\"Some <>\")\n    Text(\"\\n\")\n  ContentList\n    Text(\"content <>\")\n    Text(\"\\n\")\n  ContentList\n    Text(\"with glue.\")",
            ),
            (
                "simple_divert_fixture",
                "\
We arrived into London at 9.45pm exactly.\n\
-> hurry_home\n\
\n\
=== hurry_home ===\n\
We hurried home to Savile Row as fast as we could. -> END",
                "Story\n  ContentList\n    Text(\"We arrived into London at 9.45pm exactly.\")\n    Text(\"\\n\")\n  Divert(target=\"-> hurry_home\", empty=false, tunnel=false, thread=false)\n  ContentList\n    Text(\"\\n\")\n  Flow(level=Knot, name=\"hurry_home\", function=false)\n    ContentList\n      Text(\"We hurried home to Savile Row as fast as we could. -> END\")",
            ),
            (
                "glue_with_divert",
                "\
We hurried home <>\n\
-> to_savile_row\n\
\n\
=== to_savile_row ===\n\
to Savile Row\n\
-> as_fast_as_we_could\n\
\n\
=== as_fast_as_we_could ===\n\
<> as fast as we could.\n\
-> END",
                "Story\n  ContentList\n    Text(\"We hurried home <>\")\n    Text(\"\\n\")\n  Divert(target=\"-> to_savile_row\", empty=false, tunnel=false, thread=false)\n  ContentList\n    Text(\"\\n\")\n  Flow(level=Knot, name=\"to_savile_row\", function=false)\n    ContentList\n      Text(\"to Savile Row\")\n      Text(\"\\n\")\n    Divert(target=\"-> as_fast_as_we_could\", empty=false, tunnel=false, thread=false)\n    ContentList\n      Text(\"\\n\")\n  Flow(level=Knot, name=\"as_fast_as_we_could\", function=false)\n    ContentList\n      Text(\"<> as fast as we could.\")\n      Text(\"\\n\")\n    Divert(target=\"-> END\", empty=false, tunnel=false, thread=false)",
            ),
            (
                "sequence",
                "once: first | second",
                "Story\n  Sequence(type=Once)\n    ContentList\n      Text(\"first\")\n    ContentList\n      Text(\"second\")",
            ),
            (
                "inline_conditional",
                "{ x > 3: yes | no }",
                "Story\n  Conditional\n    Binary(>, VariableReference(x), Number(3))\n    ConditionalBranch(true=true, else=false, inline=true)\n      ContentList\n        Text(\"yes\")\n    ConditionalBranch(true=false, else=true, inline=true)\n      ContentList\n        Text(\"no\")",
            ),
            (
                "brace_sequence",
                "{once:\n  - first\n  -\n  - second\n}",
                "Story\n  Sequence(type=Once)\n    ContentList\n      Text(\"first\")\n    ContentList\n    ContentList\n      Text(\"second\")",
            ),
            (
                "brace_conditional",
                "{ x == 4:\n  The main clause\n- else:\n  other\n}",
                "Story\n  Conditional\n    Binary(==, VariableReference(x), Number(4))\n    ConditionalBranch(true=true, else=false, inline=false)\n      Text(\"The main clause\")\n      Text(\"\\n\")\n    ConditionalBranch(true=false, else=true, inline=false)\n      Text(\"other\")\n      Text(\"\\n\")",
            ),
        ];

        for (name, source, expected) in cases {
            let mut parser = InkParser::new(source, Some("story.ink"), None);
            let result = parser.parse();

            assert!(
                result.diagnostics.is_empty(),
                "case {name} produced diagnostics: {:?}",
                result.diagnostics
            );

            let story = result.parsed_story.expect("expected parsed story");
            assert_eq!(render_story(&story), expected, "case {name}");
        }
    }

    #[test]
    fn ink_parser_parses_trusted_basictext_twolines_fixture() {
        let source =
            load_workspace_text("blade-ink-rs/conformance-tests/inkfiles/basictext/twolines.ink");
        let mut parser = InkParser::new(&source, Some("twolines.ink"), None);
        let result = parser.parse();

        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:#?}",
            result.diagnostics
        );

        let story = result.parsed_story.expect("expected parsed story");
        assert_eq!(
            render_story(&story),
            "Story\n  ContentList\n    Text(\"Line.\")\n    Text(\"\\n\")\n  ContentList\n    Text(\"Other line.\")\n    Text(\"\\n\")"
        );
    }

    #[test]
    fn ink_parser_parses_trusted_basictext_oneline_fixture() {
        let source =
            load_workspace_text("blade-ink-rs/conformance-tests/inkfiles/basictext/oneline.ink");
        let mut parser = InkParser::new(&source, Some("oneline.ink"), None);
        let result = parser.parse();

        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:#?}",
            result.diagnostics
        );

        let story = result.parsed_story.expect("expected parsed story");
        assert_eq!(
            render_story(&story),
            "Story\n  ContentList\n    Text(\"Line.\")\n    Text(\"\\n\")"
        );
    }

    #[test]
    fn ink_parser_parses_trusted_conditional_iftrue_fixture() {
        let source =
            load_workspace_text("blade-ink-rs/conformance-tests/inkfiles/conditional/iftrue.ink");
        let mut parser = InkParser::new(&source, Some("iftrue.ink"), None);
        let result = parser.parse();

        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:#?}",
            result.diagnostics
        );

        let story = result.parsed_story.expect("expected parsed story");
        assert_eq!(
            render_story(&story),
            "Story\n  ContentList\n    Text(\"\\n\")\n  VariableAssignment(name=\"x\", global=true, temp=false)\n    Number(2)\n  VariableAssignment(name=\"y\", global=true, temp=false)\n    Number(0)\n  Conditional\n    Binary(>, VariableReference(x), Number(0))\n    ConditionalBranch(true=true, else=false, inline=false)\n      VariableAssignment(name=\"y\", global=false, temp=false)\n        Binary(-, VariableReference(x), Number(1))\n  ContentList\n    Text(\"        The value is {y}. -> END\")\n    Text(\"\\n\")"
        );
    }

    #[test]
    fn ink_parser_parses_trusted_conditional_ifelse_fixture() {
        let source =
            load_workspace_text("blade-ink-rs/conformance-tests/inkfiles/conditional/ifelse.ink");
        let mut parser = InkParser::new(&source, Some("ifelse.ink"), None);
        let result = parser.parse();

        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:#?}",
            result.diagnostics
        );

        let story = result.parsed_story.expect("expected parsed story");
        assert_eq!(
            render_story(&story),
            "Story\n  ContentList\n    Text(\"\\n\")\n  VariableAssignment(name=\"x\", global=true, temp=false)\n    Number(0)\n  VariableAssignment(name=\"y\", global=true, temp=false)\n    Number(3)\n  Conditional\n    Binary(>, VariableReference(x), Number(0))\n    ConditionalBranch(true=true, else=false, inline=false)\n      VariableAssignment(name=\"y\", global=false, temp=false)\n        Binary(-, VariableReference(x), Number(1))\n    ConditionalBranch(true=false, else=true, inline=false)\n      VariableAssignment(name=\"y\", global=false, temp=false)\n        Binary(+, VariableReference(x), Number(1))\n  ContentList\n    Text(\"        The value is {y}. -> END\")"
        );
    }

    #[test]
    fn ink_parser_parses_trusted_function_none_fixture() {
        let source =
            load_workspace_text("blade-ink-rs/conformance-tests/inkfiles/function/func-none.ink");
        let mut parser = InkParser::new(&source, Some("func-none.ink"), None);
        let result = parser.parse();

        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:#?}",
            result.diagnostics
        );

        let story = result.parsed_story.expect("expected parsed story");
        assert_eq!(
            render_story(&story),
            "Story\n  VariableAssignment(name=\"x\", global=true, temp=false)\n    Number(0)\n  VariableAssignment(name=\"x\", global=false, temp=false)\n    FunctionCall(f, args=0)\n  ContentList\n    Text(\"  The value of x is {x}.\")\n    Text(\"\\n\")\n  Divert(target=\"-> END\", empty=false, tunnel=false, thread=false)\n  ContentList\n    Text(\"\\n\")\n  Flow(level=Knot, name=\"f()\", function=true)\n    Return\n      Number(3.8)"
        );
    }

    #[test]
    fn ink_parser_parses_trusted_function_basic_fixture() {
        let source =
            load_workspace_text("blade-ink-rs/conformance-tests/inkfiles/function/func-basic.ink");
        let mut parser = InkParser::new(&source, Some("func-basic.ink"), None);
        let result = parser.parse();

        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:#?}",
            result.diagnostics
        );

        let story = result.parsed_story.expect("expected parsed story");
        assert_eq!(
            render_story(&story),
            "Story\n  VariableAssignment(name=\"x\", global=true, temp=false)\n    Number(0)\n  VariableAssignment(name=\"x\", global=false, temp=false)\n    FunctionCall(lerp, args=3)\n  ContentList\n    Text(\"  The value of x is {x}.\")\n    Text(\"\\n\")\n  Divert(target=\"-> END\", empty=false, tunnel=false, thread=false)\n  ContentList\n    Text(\"\\n\")\n  Flow(level=Knot, name=\"lerp(a,\", function=true)\n    Return\n      Binary(+, Binary(*, Binary(-, VariableReference(b), VariableReference(a)), VariableReference(k)), VariableReference(a))"
        );
    }

    #[test]
    fn ink_parser_parses_trusted_function_inline_fixture() {
        let source =
            load_workspace_text("blade-ink-rs/conformance-tests/inkfiles/function/func-inline.ink");
        let mut parser = InkParser::new(&source, Some("func-inline.ink"), None);
        let result = parser.parse();

        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:#?}",
            result.diagnostics
        );

        let story = result.parsed_story.expect("expected parsed story");
        assert_eq!(
            render_story(&story),
            "Story\n  ContentList\n    Text(\"The value of x is {lerp(2, 8, 0.4)}.\")\n    Text(\"\\n\")\n  Divert(target=\"-> END\", empty=false, tunnel=false, thread=false)\n  ContentList\n    Text(\"\\n\")\n  Flow(level=Knot, name=\"lerp(a,\", function=true)\n    Return\n      Binary(+, Binary(*, Binary(-, VariableReference(b), VariableReference(a)), VariableReference(k)), VariableReference(a))"
        );
    }

    #[test]
    fn ink_parser_feature_cases_cover_arithmetic_variables_lists_conditions_functions_and_sequences(
    ) {
        let cases = [
            (
                "arithmetic_and_variables",
                "VAR result = 1 + 2 * 3\n~ result = result + 4",
                "Story\n  VariableAssignment(name=\"result\", global=true, temp=false)\n    Binary(+, Number(1), Binary(*, Number(2), Number(3)))\n  VariableAssignment(name=\"result\", global=false, temp=false)\n    Binary(+, VariableReference(result), Number(4))",
            ),
            (
                "lists",
                "LIST terrain = (forest), hill = 4, (beach)\nVAR chosen = (forest, terrain.hill)",
                "Story\n  VariableAssignment(name=\"terrain\", global=true, temp=false)\n    ListDefinition(name=\"terrain\")\n      ListElementDefinition(name=\"forest\", explicit=None, series=1, initial=true)\n      ListElementDefinition(name=\"hill\", explicit=Some(4), series=4, initial=false)\n      ListElementDefinition(name=\"beach\", explicit=None, series=5, initial=true)\n  VariableAssignment(name=\"chosen\", global=true, temp=false)\n    List(forest, terrain.hill)",
            ),
            (
                "conditions",
                "VAR x = 4\n{ x == 4:\n  yes\n- else:\n  no\n}",
                "Story\n  VariableAssignment(name=\"x\", global=true, temp=false)\n    Number(4)\n  Conditional\n    Binary(==, VariableReference(x), Number(4))\n    ConditionalBranch(true=true, else=false, inline=false)\n      Text(\"yes\")\n      Text(\"\\n\")\n    ConditionalBranch(true=false, else=true, inline=false)\n      Text(\"no\")\n      Text(\"\\n\")",
            ),
            (
                "functions",
                "=== function greet ===\n~ return 7\n== start ==\n~ greet()",
                "Story\n  Flow(level=Knot, name=\"greet\", function=true)\n    Return\n      Number(7)\n  Flow(level=Knot, name=\"start\", function=false)\n    FunctionCall(greet, args=0)",
            ),
            (
                "sequences",
                "{once:\n  - first\n  -\n  - second\n}",
                "Story\n  Sequence(type=Once)\n    ContentList\n      Text(\"first\")\n    ContentList\n    ContentList\n      Text(\"second\")",
            ),
        ];

        for (name, source, expected) in cases {
            let mut parser = InkParser::new(source, Some("feature.ink"), None);
            let result = parser.parse();

            assert!(
                result.diagnostics.is_empty(),
                "case {name} produced diagnostics: {:?}",
                result.diagnostics
            );

            let story = result.parsed_story.expect("expected parsed story");
            assert_eq!(
                render_story(&story),
                expected,
                "case {name} produced an unexpected feature snapshot"
            );
        }
    }

    #[test]
    fn ink_parser_parses_official_recursive_include_chain() {
        let source = load_workspace_text("ink-csharp/tests/test_included_file3.ink");
        let source_filename = workspace_root()
            .join("ink-csharp/tests/test_included_file3.ink")
            .to_string_lossy()
            .into_owned();
        let mut files = HashMap::new();
        files.insert(
            workspace_root().join("ink-csharp/tests/test_included_file4.ink"),
            load_workspace_text("ink-csharp/tests/test_included_file4.ink"),
        );
        let file_handler: Arc<dyn FileHandler> = Arc::new(IncludeFileHandler::new(files));

        let mut parser =
            InkParser::new(&source, Some(source_filename.as_str()), Some(file_handler));
        let result = parser.parse();

        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:#?}",
            result.diagnostics
        );

        let story = result.parsed_story.expect("expected parsed story");
        assert_eq!(
            render_story(&story),
            "Story\n  VariableAssignment(name=\"t2\", global=true, temp=false)\n    Number(5)\n  ContentList\n    Text(\"\\n\")\n  ContentList\n    Text(\"The value of a variable in test file 2 is { t2 }.\")\n    Text(\"\\n\")\n  ContentList\n    Text(\"\\n\")\n  Text(\"\\n\")\n  Flow(level=Knot, name=\"knot_in_2\", function=false)\n    ContentList\n      Text(\" The value when accessed from knot_in_2 is { t2 }.\")\n      Text(\"\\n\")\n    Divert(target=\"-> END\", empty=false, tunnel=false, thread=false)"
        );
    }

    #[test]
    fn ink_parser_parses_official_include_text_chain() {
        let source = "\
INCLUDE test_included_file.ink\n\
  INCLUDE test_included_file2.ink\n\
\n\
This is the main file.\n";
        let mut files = HashMap::new();
        files.insert(
            workspace_root().join("ink-csharp/tests/test_included_file.ink"),
            load_workspace_text("ink-csharp/tests/test_included_file.ink"),
        );
        files.insert(
            workspace_root().join("ink-csharp/tests/test_included_file2.ink"),
            load_workspace_text("ink-csharp/tests/test_included_file2.ink"),
        );
        let file_handler: Arc<dyn FileHandler> = Arc::new(IncludeFileHandler::new(files));

        let mut parser = InkParser::new(
            source,
            Some("ink-csharp/tests/include-chain.ink"),
            Some(file_handler),
        );
        let result = parser.parse();

        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:#?}",
            result.diagnostics
        );

        let story = result.parsed_story.expect("expected parsed story");
        assert_eq!(
            render_story(&story),
            "Story\n  ContentList\n    Text(\"This is include 1.\")\n  Text(\"\\n\")\n  ContentList\n    Text(\"This is include 2.\")\n  Text(\"\\n\")\n  ContentList\n    Text(\"\\n\")\n  ContentList\n    Text(\"This is the main file.\")\n    Text(\"\\n\")"
        );
    }
}
