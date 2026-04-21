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

use std::sync::Arc;

use crate::{
    error::{Diagnostic, DiagnosticSeverity},
    parsed::{
        ConstantDeclaration, ContentList, Divert, ExternalDeclaration, FlowLevel, Identifier, Knot,
        Object, ObjectRef, Path, Stitch, Story as ParsedStory, Text, VariableAssignment,
    },
    results::{FileHandler, ParseResult},
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
        let source_filename = self.source_filename.map(str::to_string);

        if self.input_string.is_empty() {
            return Ok(ParsedStory::new(top_level_content, false));
        }

        for (line_index, segment) in self.input_string.split_inclusive('\n').enumerate() {
            let had_newline = segment.ends_with('\n');
            let line_text = segment.strip_suffix('\n').unwrap_or(segment);

            if let Some(statement) =
                Self::parse_statement_line(line_text, line_index + 1, source_filename.clone())?
            {
                if let Some(parent) = current_flow.as_ref() {
                    Object::add_content(parent, statement);
                } else {
                    top_level_content.push(statement);
                }
                continue;
            }

            if let Some(flow) =
                Self::parse_flow_header(line_text, line_index + 1, source_filename.clone())?
            {
                top_level_content.push(flow.clone());
                current_flow = Some(flow);
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
                continue;
            }

            let line = Self::build_content_line(line_text, had_newline);
            if let Some(parent) = current_flow.as_ref() {
                Object::add_content(parent, line);
            } else {
                top_level_content.push(line);
            }
        }

        Ok(ParsedStory::new(top_level_content, false))
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

        if trimmed_start.starts_with("->->") {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                source_filename,
                line_number,
                line_text.len().saturating_sub(trimmed_start.len()) + 1,
                "Tunnel diverts are not supported yet",
            ));
        }

        let remainder = trimmed_start[2..].trim_start();
        let line_indent = line_text.len().saturating_sub(trimmed_start.len()) + 1;

        if remainder.is_empty() {
            return Ok(Some(Divert::empty().object()));
        }

        let mut target_tokens = remainder.split_whitespace();
        let target_text = target_tokens.next().unwrap_or_default();
        if target_text.is_empty() {
            return Ok(Some(Divert::empty().object()));
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

        Ok(Some(Divert::new(Some(target)).object()))
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
        for (line_index, line) in self.input_string.lines().enumerate() {
            let trimmed = line.trim_start();
            let column = line.len().saturating_sub(trimmed.len()) + 1;

            for marker in ["*", "-", "#", "{", "}"] {
                if marker == "-" && trimmed.starts_with("->") {
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
    use super::{CommentEliminator, InkParser};
    use crate::parsed::{ObjectKind, ObjectRef, Story as ParsedStory};

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
            other => lines.push(format!("{padding}{other:?}")),
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
    fn ink_parser_rejects_tunnel_diverts() {
        let mut parser = InkParser::new("->-> target", Some("story.ink"), None);
        let result = parser.parse();

        assert!(result.parsed_story.is_none());
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].message,
            "Tunnel diverts are not supported yet"
        );
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
}
