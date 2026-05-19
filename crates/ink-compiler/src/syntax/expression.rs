use crate::{
    diagnostic::{Diagnostic, DiagnosticCode},
    parsed::{
        BinaryOperator, ContentList, Expression, FloatLiteral, Object, QualifiedName,
        StructLiteralField, UnaryOperator,
    },
    source::SourceSpan,
};

use super::{is_identifier, is_identifier_continue, scan, text};

pub(super) fn parse_initial_expression(source: &str) -> Option<Expression> {
    match parse_token_expression(source.trim()) {
        Ok(expression) => Some(expression),
        Err(error) => {
            let _ = error.message();
            None
        }
    }
}

pub(super) fn parse_initial_expression_or_error(
    source: &str,
    base_span: SourceSpan,
) -> Result<Expression, Diagnostic> {
    parse_token_expression_at(source, base_span).map_err(|error| error.into_diagnostic())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExpressionToken {
    kind: ExpressionTokenKind,
    byte_index: usize,
    span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExpressionTokenKind {
    Identifier(String),
    IntLiteral(String),
    FloatLiteral(String),
    StringLiteral(String),
    Operator(String),
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
    Colon,
    DoubleColon,
    Dot,
    Comma,
    Arrow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BinaryOperatorRule {
    text: &'static str,
    operator: BinaryOperator,
    precedence: u8,
    token_kind: OperatorTokenKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperatorTokenKind {
    Symbol,
    Word,
}

const BINARY_OPERATOR_RULES: &[BinaryOperatorRule] = &[
    BinaryOperatorRule {
        text: "&&",
        operator: BinaryOperator::AndSymbol,
        precedence: 1,
        token_kind: OperatorTokenKind::Symbol,
    },
    BinaryOperatorRule {
        text: "||",
        operator: BinaryOperator::OrSymbol,
        precedence: 1,
        token_kind: OperatorTokenKind::Symbol,
    },
    BinaryOperatorRule {
        text: "and",
        operator: BinaryOperator::And,
        precedence: 1,
        token_kind: OperatorTokenKind::Word,
    },
    BinaryOperatorRule {
        text: "or",
        operator: BinaryOperator::Or,
        precedence: 1,
        token_kind: OperatorTokenKind::Word,
    },
    BinaryOperatorRule {
        text: "==",
        operator: BinaryOperator::Equals,
        precedence: 2,
        token_kind: OperatorTokenKind::Symbol,
    },
    BinaryOperatorRule {
        text: "!=",
        operator: BinaryOperator::NotEquals,
        precedence: 2,
        token_kind: OperatorTokenKind::Symbol,
    },
    BinaryOperatorRule {
        text: ">=",
        operator: BinaryOperator::GreaterThanOrEquals,
        precedence: 2,
        token_kind: OperatorTokenKind::Symbol,
    },
    BinaryOperatorRule {
        text: "<=",
        operator: BinaryOperator::LessThanOrEquals,
        precedence: 2,
        token_kind: OperatorTokenKind::Symbol,
    },
    BinaryOperatorRule {
        text: ">",
        operator: BinaryOperator::GreaterThan,
        precedence: 2,
        token_kind: OperatorTokenKind::Symbol,
    },
    BinaryOperatorRule {
        text: "<",
        operator: BinaryOperator::LessThan,
        precedence: 2,
        token_kind: OperatorTokenKind::Symbol,
    },
    BinaryOperatorRule {
        text: "!?",
        operator: BinaryOperator::Hasnt,
        precedence: 3,
        token_kind: OperatorTokenKind::Symbol,
    },
    BinaryOperatorRule {
        text: "hasnt",
        operator: BinaryOperator::Hasnt,
        precedence: 4,
        token_kind: OperatorTokenKind::Word,
    },
    BinaryOperatorRule {
        text: "has",
        operator: BinaryOperator::Has,
        precedence: 4,
        token_kind: OperatorTokenKind::Word,
    },
    BinaryOperatorRule {
        text: "?",
        operator: BinaryOperator::Has,
        precedence: 4,
        token_kind: OperatorTokenKind::Symbol,
    },
    BinaryOperatorRule {
        text: "+",
        operator: BinaryOperator::Add,
        precedence: 5,
        token_kind: OperatorTokenKind::Symbol,
    },
    BinaryOperatorRule {
        text: "-",
        operator: BinaryOperator::Subtract,
        precedence: 5,
        token_kind: OperatorTokenKind::Symbol,
    },
    BinaryOperatorRule {
        text: "*",
        operator: BinaryOperator::Multiply,
        precedence: 6,
        token_kind: OperatorTokenKind::Symbol,
    },
    BinaryOperatorRule {
        text: "/",
        operator: BinaryOperator::Divide,
        precedence: 7,
        token_kind: OperatorTokenKind::Symbol,
    },
    BinaryOperatorRule {
        text: "mod",
        operator: BinaryOperator::Modulo,
        precedence: 8,
        token_kind: OperatorTokenKind::Word,
    },
    BinaryOperatorRule {
        text: "%",
        operator: BinaryOperator::Modulo,
        precedence: 9,
        token_kind: OperatorTokenKind::Symbol,
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExpressionParseError {
    kind: ExpressionParseErrorKind,
    span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExpressionParseErrorKind {
    ExpectedExpression {
        found: Option<String>,
    },
    ExpectedRightOperand {
        operator: String,
        found: Option<String>,
    },
    ExpectedCloseParen {
        found: Option<String>,
    },
    ExpectedCommaOrCallCloseParen {
        name: String,
        found: Option<String>,
    },
    ExpectedCommaOrArrayCloseBracket {
        found: Option<String>,
    },
    ExpectedStructLiteralField {
        found: Option<String>,
    },
    ExpectedStructFieldColon {
        name: String,
        found: Option<String>,
    },
    ExpectedCommaOrStructCloseBrace {
        found: Option<String>,
    },
    ExpectedFieldName {
        found: Option<String>,
    },
    ExpectedQualifiedSymbol {
        module: String,
        found: Option<String>,
    },
    ExpectedDynamicInterfaceMember {
        found: Option<String>,
    },
    ExpectedDynamicInterfaceTargetClose {
        found: Option<String>,
    },
    ExpectedIndexCloseBracket {
        found: Option<String>,
    },
    ExpectedDivertTarget {
        found: Option<String>,
    },
    UnexpectedTrailingToken {
        found: String,
    },
    InvalidIntegerLiteral {
        value: String,
    },
    InvalidFloatLiteral {
        value: String,
    },
}

impl ExpressionParseError {
    fn new(kind: ExpressionParseErrorKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    fn message(&self) -> String {
        match &self.kind {
            ExpressionParseErrorKind::ExpectedExpression { found } => {
                format!("expected expression{}", found_clause(found))
            }
            ExpressionParseErrorKind::ExpectedRightOperand { operator, found } => {
                format!(
                    "expected expression after operator `{operator}`{}",
                    found_clause(found)
                )
            }
            ExpressionParseErrorKind::ExpectedCloseParen { found } => {
                format!(
                    "expected `)` to close parenthesized expression{}",
                    found_clause(found)
                )
            }
            ExpressionParseErrorKind::ExpectedCommaOrCallCloseParen { name, found } => {
                format!(
                    "expected `,` or `)` in call to `{name}`{}",
                    found_clause(found)
                )
            }
            ExpressionParseErrorKind::ExpectedCommaOrArrayCloseBracket { found } => {
                format!(
                    "expected `,` or `]` in array literal{}",
                    found_clause(found)
                )
            }
            ExpressionParseErrorKind::ExpectedStructLiteralField { found } => {
                format!(
                    "expected field name in struct literal{}",
                    found_clause(found)
                )
            }
            ExpressionParseErrorKind::ExpectedStructFieldColon { name, found } => {
                format!(
                    "expected `:` after struct literal field `{name}`{}",
                    found_clause(found)
                )
            }
            ExpressionParseErrorKind::ExpectedCommaOrStructCloseBrace { found } => {
                format!(
                    "expected `,` or `}}` in struct literal{}",
                    found_clause(found)
                )
            }
            ExpressionParseErrorKind::ExpectedFieldName { found } => {
                format!("expected field name after `.`{}", found_clause(found))
            }
            ExpressionParseErrorKind::ExpectedQualifiedSymbol { module, found } => {
                format!(
                    "expected symbol name after `{module}::`{}",
                    found_clause(found)
                )
            }
            ExpressionParseErrorKind::ExpectedDynamicInterfaceMember { found } => {
                format!(
                    "expected dynamic interface member name after `::`{}",
                    found_clause(found)
                )
            }
            ExpressionParseErrorKind::ExpectedDynamicInterfaceTargetClose { found } => {
                format!(
                    "expected `}}` to close dynamic interface target{}",
                    found_clause(found)
                )
            }
            ExpressionParseErrorKind::ExpectedIndexCloseBracket { found } => {
                format!("expected `]` to close index access{}", found_clause(found))
            }
            ExpressionParseErrorKind::ExpectedDivertTarget { found } => {
                format!("expected divert target after `->`{}", found_clause(found))
            }
            ExpressionParseErrorKind::UnexpectedTrailingToken { found } => {
                format!("unexpected token `{found}` after expression")
            }
            ExpressionParseErrorKind::InvalidIntegerLiteral { value } => {
                format!("invalid integer literal `{value}`")
            }
            ExpressionParseErrorKind::InvalidFloatLiteral { value } => {
                format!("invalid float literal `{value}`")
            }
        }
    }

    fn into_diagnostic(self) -> Diagnostic {
        let message = self.message();
        Diagnostic::error(self.span, message).with_code(DiagnosticCode::InvalidExpression)
    }
}

fn found_clause(found: &Option<String>) -> String {
    match found {
        Some(found) => format!(", found `{found}`"),
        None => " before end of input".to_string(),
    }
}

#[cfg(test)]
fn tokenize_expression(source: &str) -> Vec<ExpressionToken> {
    tokenize_expression_at(source, &SourceSpan::new(None, 1, 1))
}

fn tokenize_expression_at(source: &str, base_span: &SourceSpan) -> Vec<ExpressionToken> {
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < source.len() {
        let rest = &source[index..];
        let ch = rest.chars().next().expect("index is inside source");

        if ch.is_whitespace() {
            index += ch.len_utf8();
            continue;
        }

        if rest.starts_with("->") {
            tokens.push(token_at(
                source,
                index,
                ExpressionTokenKind::Arrow,
                base_span,
            ));
            index += "->".len();
            continue;
        }

        if rest.starts_with("::") {
            tokens.push(token_at(
                source,
                index,
                ExpressionTokenKind::DoubleColon,
                base_span,
            ));
            index += "::".len();
            continue;
        }

        if let Some(operator) = match_operator(rest) {
            tokens.push(token_at(
                source,
                index,
                ExpressionTokenKind::Operator(operator.to_string()),
                base_span,
            ));
            index += operator.len();
            continue;
        }

        match ch {
            '(' => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::OpenParen,
                    base_span,
                ));
                index += ch.len_utf8();
            }
            ')' => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::CloseParen,
                    base_span,
                ));
                index += ch.len_utf8();
            }
            '[' => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::OpenBracket,
                    base_span,
                ));
                index += ch.len_utf8();
            }
            ']' => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::CloseBracket,
                    base_span,
                ));
                index += ch.len_utf8();
            }
            '{' => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::OpenBrace,
                    base_span,
                ));
                index += ch.len_utf8();
            }
            '}' => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::CloseBrace,
                    base_span,
                ));
                index += ch.len_utf8();
            }
            ':' => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::Colon,
                    base_span,
                ));
                index += ch.len_utf8();
            }
            '.' => {
                tokens.push(token_at(source, index, ExpressionTokenKind::Dot, base_span));
                index += ch.len_utf8();
            }
            ',' => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::Comma,
                    base_span,
                ));
                index += ch.len_utf8();
            }
            '"' => {
                let (literal, next_index) = read_string_literal(source, index);
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::StringLiteral(literal),
                    base_span,
                ));
                index = next_index;
            }
            _ if is_token_word_start(ch) => {
                let (word, next_index) = read_token_word(source, index);
                let (kind, next_index) =
                    classify_word_token_with_decimal_suffix(source, index, word, next_index);
                tokens.push(token_at(source, index, kind, base_span));
                index = next_index;
            }
            _ => {
                tokens.push(token_at(
                    source,
                    index,
                    ExpressionTokenKind::Operator(ch.to_string()),
                    base_span,
                ));
                index += ch.len_utf8();
            }
        }
    }

    tokens
}

fn token_at(
    source: &str,
    byte_index: usize,
    kind: ExpressionTokenKind,
    base_span: &SourceSpan,
) -> ExpressionToken {
    ExpressionToken {
        kind,
        byte_index,
        span: SourceSpan::new(
            base_span.source_name.clone(),
            base_span.line,
            base_span.column + source[..byte_index].chars().count(),
        ),
    }
}

fn match_operator(source: &str) -> Option<&'static str> {
    BINARY_OPERATOR_RULES
        .iter()
        .filter(|rule| rule.token_kind == OperatorTokenKind::Symbol)
        .map(|rule| rule.text)
        .find(|operator| source.starts_with(operator))
}

fn read_string_literal(source: &str, start: usize) -> (String, usize) {
    let mut literal = String::new();
    let mut index = start + '"'.len_utf8();
    let mut escaped = false;
    let mut brace_depth = 0;
    let mut in_nested_string = false;

    while index < source.len() {
        let ch = source[index..]
            .chars()
            .next()
            .expect("index is inside source");
        index += ch.len_utf8();

        if escaped {
            if brace_depth == 0 {
                match ch {
                    'n' => literal.push('\n'),
                    'r' => literal.push('\r'),
                    't' => literal.push('\t'),
                    '"' => literal.push('"'),
                    '\\' => literal.push('\\'),
                    other => literal.push(other),
                }
            } else {
                literal.push('\\');
                literal.push(ch);
            }
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' if brace_depth == 0 => return (literal, index),
            '"' => {
                in_nested_string = !in_nested_string;
                literal.push(ch);
            }
            '{' if !in_nested_string => {
                brace_depth += 1;
                literal.push(ch);
            }
            '}' if !in_nested_string && brace_depth > 0 => {
                brace_depth -= 1;
                literal.push(ch);
            }
            other => literal.push(other),
        }
    }

    if escaped {
        literal.push('\\');
    }

    (literal, index)
}

fn is_token_word_start(ch: char) -> bool {
    is_identifier_continue(ch)
}

fn read_token_word(source: &str, start: usize) -> (&str, usize) {
    let mut end = start;

    for (relative_index, ch) in source[start..].char_indices() {
        if is_identifier_continue(ch) {
            end = start + relative_index + ch.len_utf8();
        } else {
            break;
        }
    }

    (&source[start..end], end)
}

fn classify_word_token_with_decimal_suffix(
    source: &str,
    start: usize,
    word: &str,
    word_end: usize,
) -> (ExpressionTokenKind, usize) {
    let kind = classify_word_token(word);
    if !matches!(kind, ExpressionTokenKind::IntLiteral(_)) {
        return (kind, word_end);
    }

    let Some(fraction_start) = source[word_end..]
        .strip_prefix('.')
        .map(|_| word_end + '.'.len_utf8())
    else {
        return (kind, word_end);
    };

    let mut fraction_end = fraction_start;
    for (relative_index, ch) in source[fraction_start..].char_indices() {
        if ch.is_ascii_digit() {
            fraction_end = fraction_start + relative_index + ch.len_utf8();
        } else {
            break;
        }
    }

    if fraction_end == fraction_start {
        return (kind, word_end);
    }

    (
        ExpressionTokenKind::FloatLiteral(source[start..fraction_end].to_string()),
        fraction_end,
    )
}

fn classify_word_token(word: &str) -> ExpressionTokenKind {
    if binary_operator_rule(word).is_some() || word == "not" {
        return ExpressionTokenKind::Operator(word.to_string());
    }

    if word.chars().all(|ch| ch.is_ascii_digit()) {
        return ExpressionTokenKind::IntLiteral(word.to_string());
    }

    if word.contains('.')
        && word.chars().all(|ch| ch.is_ascii_digit() || ch == '.')
        && word.parse::<f64>().is_ok()
    {
        return ExpressionTokenKind::FloatLiteral(word.to_string());
    }

    ExpressionTokenKind::Identifier(word.to_string())
}

fn binary_operator_rule(text: &str) -> Option<BinaryOperatorRule> {
    BINARY_OPERATOR_RULES
        .iter()
        .find(|rule| rule.text == text)
        .copied()
}

struct TokenExpressionParser<'a> {
    tokens: &'a [ExpressionToken],
    index: usize,
    eof_span: SourceSpan,
}

impl<'a> TokenExpressionParser<'a> {
    fn new(tokens: &'a [ExpressionToken], eof_span: SourceSpan) -> Self {
        Self {
            tokens,
            index: 0,
            eof_span,
        }
    }

    fn parse(mut self) -> Result<Expression, ExpressionParseError> {
        let expression = self.parse_expression(0)?;
        if let Some(token) = self.peek() {
            return Err(ExpressionParseError::new(
                ExpressionParseErrorKind::UnexpectedTrailingToken {
                    found: describe_token_kind(&token.kind),
                },
                token.span.clone(),
            ));
        }
        Ok(expression)
    }

    fn parse_expression(
        &mut self,
        minimum_precedence: u8,
    ) -> Result<Expression, ExpressionParseError> {
        let mut left = self.parse_prefix()?;
        left = self.parse_postfix(left)?;

        while let Some((operator, precedence, operator_text)) = self.current_binary_operator() {
            if precedence < minimum_precedence {
                break;
            }
            self.index += 1;
            let right = self
                .parse_expression(precedence + 1)
                .map_err(|error| error.for_right_operand(operator_text))?;
            left = Expression::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
            left = self.parse_postfix(left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expression, ExpressionParseError> {
        let Some(token) = self.advance() else {
            return Err(
                self.error_at_eof(ExpressionParseErrorKind::ExpectedExpression { found: None })
            );
        };
        let kind = token.kind.clone();
        match kind {
            ExpressionTokenKind::IntLiteral(value) => {
                let parsed = value.parse::<i32>().map_err(|_| {
                    ExpressionParseError::new(
                        ExpressionParseErrorKind::InvalidIntegerLiteral {
                            value: value.clone(),
                        },
                        token.span.clone(),
                    )
                })?;
                Ok(Expression::NumberInt(parsed))
            }
            ExpressionTokenKind::FloatLiteral(value) => {
                let parsed = value.parse().map_err(|_| {
                    ExpressionParseError::new(
                        ExpressionParseErrorKind::InvalidFloatLiteral {
                            value: value.clone(),
                        },
                        token.span.clone(),
                    )
                })?;
                Ok(Expression::NumberFloat(FloatLiteral::new(parsed)))
            }
            ExpressionTokenKind::StringLiteral(value) => Ok(parse_string_expression(&value)),
            ExpressionTokenKind::Identifier(name) => {
                self.parse_identifier_or_call(&name, token.span.clone())
            }
            ExpressionTokenKind::Arrow => self.parse_divert_target(),
            ExpressionTokenKind::Operator(operator)
                if matches!(operator.as_str(), "-" | "!" | "not") =>
            {
                let unary_operator = match operator.as_str() {
                    "-" => UnaryOperator::Negate,
                    "!" | "not" => UnaryOperator::Not,
                    _ => {
                        return Err(ExpressionParseError::new(
                            ExpressionParseErrorKind::ExpectedExpression {
                                found: Some(operator),
                            },
                            token.span.clone(),
                        ))
                    }
                };
                let expression = self.parse_expression(10)?;
                Ok(unary_expression(unary_operator, expression))
            }
            ExpressionTokenKind::OpenParen => {
                let expression = self.parse_expression(0)?;
                self.expect_kind(
                    |kind| matches!(kind, ExpressionTokenKind::CloseParen),
                    |found| ExpressionParseErrorKind::ExpectedCloseParen { found },
                )?;
                Ok(expression)
            }
            ExpressionTokenKind::OpenBracket => self.parse_array_literal(),
            ExpressionTokenKind::OpenBrace => {
                if self.current_brace_pair_is_dynamic_interface_target() {
                    self.parse_dynamic_interface_target()
                } else {
                    self.parse_struct_literal()
                }
            }
            other => Err(ExpressionParseError::new(
                ExpressionParseErrorKind::ExpectedExpression {
                    found: Some(describe_token_kind(&other)),
                },
                token.span.clone(),
            )),
        }
    }

    fn parse_postfix(
        &mut self,
        mut expression: Expression,
    ) -> Result<Expression, ExpressionParseError> {
        loop {
            if self.match_kind(|kind| matches!(kind, ExpressionTokenKind::Dot)) {
                let Some(token) = self.advance() else {
                    return Err(
                        self.error_at_eof(ExpressionParseErrorKind::ExpectedFieldName {
                            found: None,
                        }),
                    );
                };
                let kind = token.kind.clone();
                let ExpressionTokenKind::Identifier(field) = kind else {
                    return Err(ExpressionParseError::new(
                        ExpressionParseErrorKind::ExpectedFieldName {
                            found: Some(describe_token_kind(&kind)),
                        },
                        token.span.clone(),
                    ));
                };
                if !is_identifier(&field) {
                    return Err(ExpressionParseError::new(
                        ExpressionParseErrorKind::ExpectedFieldName { found: Some(field) },
                        token.span.clone(),
                    ));
                }
                expression = Expression::FieldAccess {
                    base: Box::new(expression),
                    field,
                };
                continue;
            }

            if self.match_kind(|kind| matches!(kind, ExpressionTokenKind::OpenBracket)) {
                let index = self.parse_expression(0)?;
                self.expect_kind(
                    |kind| matches!(kind, ExpressionTokenKind::CloseBracket),
                    |found| ExpressionParseErrorKind::ExpectedIndexCloseBracket { found },
                )?;
                expression = Expression::IndexAccess {
                    base: Box::new(expression),
                    index: Box::new(index),
                };
                continue;
            }

            break;
        }

        Ok(expression)
    }

    fn parse_identifier_or_call(
        &mut self,
        name: &str,
        span: SourceSpan,
    ) -> Result<Expression, ExpressionParseError> {
        if name == "true" {
            return Ok(Expression::NumberBool(true));
        }
        if name == "false" {
            return Ok(Expression::NumberBool(false));
        }

        let qualified_name =
            if self.match_kind(|kind| matches!(kind, ExpressionTokenKind::DoubleColon)) {
                Some(self.parse_qualified_name_after_module(name, span.clone())?)
            } else {
                None
            };

        if !self.match_kind(|kind| matches!(kind, ExpressionTokenKind::OpenParen)) {
            if let Some(qualified_name) = qualified_name {
                return Ok(Expression::QualifiedReference(qualified_name));
            }
            if !is_path_identifier(name) {
                return Err(ExpressionParseError::new(
                    ExpressionParseErrorKind::ExpectedExpression {
                        found: Some(name.to_string()),
                    },
                    span,
                ));
            }
            return Ok(Expression::VariableReference(name.to_string()));
        }

        if qualified_name.is_none() && !is_identifier(name) {
            return Err(ExpressionParseError::new(
                ExpressionParseErrorKind::ExpectedExpression {
                    found: Some(name.to_string()),
                },
                span,
            ));
        }

        let args = self.parse_argument_list(name)?;

        if let Some(qualified_name) = qualified_name {
            return Ok(Expression::QualifiedFunctionCall {
                name: qualified_name,
                args,
            });
        }
        Ok(Expression::FunctionCall {
            name: name.to_string(),
            args,
        })
    }

    fn parse_argument_list(&mut self, name: &str) -> Result<Vec<Expression>, ExpressionParseError> {
        let mut args = Vec::new();
        if self.match_kind(|kind| matches!(kind, ExpressionTokenKind::CloseParen)) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expression(0)?);
            if self.match_kind(|kind| matches!(kind, ExpressionTokenKind::Comma)) {
                continue;
            }
            self.expect_kind(
                |kind| matches!(kind, ExpressionTokenKind::CloseParen),
                |found| ExpressionParseErrorKind::ExpectedCommaOrCallCloseParen {
                    name: name.to_string(),
                    found,
                },
            )?;
            break;
        }

        Ok(args)
    }

    fn current_brace_pair_is_dynamic_interface_target(&self) -> bool {
        let mut depth = 1;
        for (index, token) in self.tokens.iter().enumerate().skip(self.index) {
            match &token.kind {
                ExpressionTokenKind::OpenBrace => depth += 1,
                ExpressionTokenKind::CloseBrace => {
                    depth -= 1;
                    if depth == 0 {
                        return self.tokens.get(index + 1).is_some_and(|next| {
                            matches!(&next.kind, ExpressionTokenKind::DoubleColon)
                        });
                    }
                }
                _ => {}
            }
        }

        false
    }

    fn parse_dynamic_interface_target(&mut self) -> Result<Expression, ExpressionParseError> {
        let target = self.parse_expression(0)?;
        self.expect_kind(
            |kind| matches!(kind, ExpressionTokenKind::CloseBrace),
            |found| ExpressionParseErrorKind::ExpectedDynamicInterfaceTargetClose { found },
        )?;
        self.expect_kind(
            |kind| matches!(kind, ExpressionTokenKind::DoubleColon),
            |found| ExpressionParseErrorKind::ExpectedDynamicInterfaceMember { found },
        )?;
        self.parse_dynamic_interface_member_after_target(target)
    }

    fn parse_dynamic_interface_member_after_target(
        &mut self,
        target: Expression,
    ) -> Result<Expression, ExpressionParseError> {
        let Some(token) = self.advance() else {
            return Err(self.error_at_eof(
                ExpressionParseErrorKind::ExpectedDynamicInterfaceMember { found: None },
            ));
        };
        let kind = token.kind.clone();
        let ExpressionTokenKind::Identifier(member) = kind else {
            return Err(ExpressionParseError::new(
                ExpressionParseErrorKind::ExpectedDynamicInterfaceMember {
                    found: Some(describe_token_kind(&kind)),
                },
                token.span.clone(),
            ));
        };
        if !is_identifier(&member) {
            return Err(ExpressionParseError::new(
                ExpressionParseErrorKind::ExpectedDynamicInterfaceMember {
                    found: Some(member),
                },
                token.span.clone(),
            ));
        }

        if !self.match_kind(|kind| matches!(kind, ExpressionTokenKind::OpenParen)) {
            return Ok(Expression::DynamicInterfaceAccess {
                target: Box::new(target),
                member,
            });
        }

        let args = self.parse_argument_list(&member)?;
        Ok(Expression::DynamicInterfaceFunctionCall {
            target: Box::new(target),
            member,
            args,
        })
    }

    fn parse_qualified_name_after_module(
        &mut self,
        module: &str,
        module_span: SourceSpan,
    ) -> Result<QualifiedName, ExpressionParseError> {
        let Some(token) = self.advance() else {
            return Err(
                self.error_at_eof(ExpressionParseErrorKind::ExpectedQualifiedSymbol {
                    module: module.to_string(),
                    found: None,
                }),
            );
        };
        let kind = token.kind.clone();
        let ExpressionTokenKind::Identifier(symbol) = kind else {
            return Err(ExpressionParseError::new(
                ExpressionParseErrorKind::ExpectedQualifiedSymbol {
                    module: module.to_string(),
                    found: Some(describe_token_kind(&kind)),
                },
                token.span.clone(),
            ));
        };
        if !is_identifier(module) || !is_identifier(&symbol) {
            return Err(ExpressionParseError::new(
                ExpressionParseErrorKind::ExpectedQualifiedSymbol {
                    module: module.to_string(),
                    found: Some(symbol),
                },
                token.span.clone(),
            ));
        }

        Ok(QualifiedName::new(
            module.to_string(),
            module_span,
            symbol,
            token.span.clone(),
        ))
    }

    fn parse_array_literal(&mut self) -> Result<Expression, ExpressionParseError> {
        let mut elements = Vec::new();
        if self.match_kind(|kind| matches!(kind, ExpressionTokenKind::CloseBracket)) {
            return Ok(Expression::ArrayLiteral(elements));
        }

        loop {
            elements.push(self.parse_expression(0)?);
            if self.match_kind(|kind| matches!(kind, ExpressionTokenKind::Comma)) {
                continue;
            }
            self.expect_kind(
                |kind| matches!(kind, ExpressionTokenKind::CloseBracket),
                |found| ExpressionParseErrorKind::ExpectedCommaOrArrayCloseBracket { found },
            )?;
            break;
        }

        Ok(Expression::ArrayLiteral(elements))
    }

    fn parse_struct_literal(&mut self) -> Result<Expression, ExpressionParseError> {
        let mut fields = Vec::new();
        if self.match_kind(|kind| matches!(kind, ExpressionTokenKind::CloseBrace)) {
            return Ok(Expression::StructLiteral(fields));
        }

        loop {
            let Some(token) = self.advance() else {
                return Err(self.error_at_eof(
                    ExpressionParseErrorKind::ExpectedStructLiteralField { found: None },
                ));
            };
            let kind = token.kind.clone();
            let ExpressionTokenKind::Identifier(name) = kind else {
                return Err(ExpressionParseError::new(
                    ExpressionParseErrorKind::ExpectedStructLiteralField {
                        found: Some(describe_token_kind(&kind)),
                    },
                    token.span.clone(),
                ));
            };
            if !is_identifier(&name) {
                return Err(ExpressionParseError::new(
                    ExpressionParseErrorKind::ExpectedStructLiteralField { found: Some(name) },
                    token.span.clone(),
                ));
            }

            self.expect_kind(
                |kind| matches!(kind, ExpressionTokenKind::Colon),
                |found| ExpressionParseErrorKind::ExpectedStructFieldColon {
                    name: name.clone(),
                    found,
                },
            )?;
            let expression = self.parse_expression(0)?;
            fields.push(StructLiteralField::new(name, expression));

            if self.match_kind(|kind| matches!(kind, ExpressionTokenKind::Comma)) {
                continue;
            }
            self.expect_kind(
                |kind| matches!(kind, ExpressionTokenKind::CloseBrace),
                |found| ExpressionParseErrorKind::ExpectedCommaOrStructCloseBrace { found },
            )?;
            break;
        }

        Ok(Expression::StructLiteral(fields))
    }

    fn parse_divert_target(&mut self) -> Result<Expression, ExpressionParseError> {
        let Some(token) = self.advance() else {
            return Err(
                self.error_at_eof(ExpressionParseErrorKind::ExpectedDivertTarget { found: None })
            );
        };
        let kind = token.kind.clone();
        let ExpressionTokenKind::Identifier(target) = kind else {
            return Err(ExpressionParseError::new(
                ExpressionParseErrorKind::ExpectedDivertTarget {
                    found: Some(describe_token_kind(&kind)),
                },
                token.span.clone(),
            ));
        };
        if self.match_kind(|kind| matches!(kind, ExpressionTokenKind::DoubleColon)) {
            let target = self.parse_qualified_name_after_module(&target, token.span.clone())?;
            return Ok(Expression::DivertTarget(target.as_str().to_string()));
        }
        let mut target = target;
        while self.match_kind(|kind| matches!(kind, ExpressionTokenKind::Dot)) {
            let Some(token) = self.advance() else {
                return Err(self
                    .error_at_eof(ExpressionParseErrorKind::ExpectedDivertTarget { found: None }));
            };
            let kind = token.kind.clone();
            let ExpressionTokenKind::Identifier(part) = kind else {
                return Err(ExpressionParseError::new(
                    ExpressionParseErrorKind::ExpectedDivertTarget {
                        found: Some(describe_token_kind(&kind)),
                    },
                    token.span.clone(),
                ));
            };
            if !is_identifier(&part) {
                return Err(ExpressionParseError::new(
                    ExpressionParseErrorKind::ExpectedDivertTarget { found: Some(part) },
                    token.span.clone(),
                ));
            }
            target.push('.');
            target.push_str(&part);
        }
        Ok(Expression::DivertTarget(
            crate::parsed::DivertTarget::from_source(&target).to_snapshot_string(),
        ))
    }

    fn current_binary_operator(&self) -> Option<(BinaryOperator, u8, &'static str)> {
        let token = self.peek()?;
        let ExpressionTokenKind::Operator(operator) = &token.kind else {
            return None;
        };
        binary_operator_rule(operator).map(|rule| (rule.operator, rule.precedence, rule.text))
    }

    fn peek(&self) -> Option<&'a ExpressionToken> {
        self.tokens.get(self.index)
    }

    fn advance(&mut self) -> Option<&'a ExpressionToken> {
        let token = self.peek()?;
        self.index += 1;
        Some(token)
    }

    fn match_kind(&mut self, matches: impl FnOnce(&ExpressionTokenKind) -> bool) -> bool {
        if self.peek().is_some_and(|token| matches(&token.kind)) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn expect_kind(
        &mut self,
        matches: impl FnOnce(&ExpressionTokenKind) -> bool,
        error_kind: impl FnOnce(Option<String>) -> ExpressionParseErrorKind,
    ) -> Result<&'a ExpressionToken, ExpressionParseError> {
        if self.peek().is_some_and(|token| matches(&token.kind)) {
            Ok(self.advance().expect("peek already checked token"))
        } else {
            let (found, span) = self.found_token_or_eof();
            Err(ExpressionParseError::new(error_kind(found), span))
        }
    }

    fn found_token_or_eof(&self) -> (Option<String>, SourceSpan) {
        self.peek()
            .map(|token| (Some(describe_token_kind(&token.kind)), token.span.clone()))
            .unwrap_or_else(|| (None, self.eof_span.clone()))
    }

    fn error_at_eof(&self, kind: ExpressionParseErrorKind) -> ExpressionParseError {
        ExpressionParseError::new(kind, self.eof_span.clone())
    }
}

impl ExpressionParseError {
    fn for_right_operand(self, operator: &str) -> Self {
        match self.kind {
            ExpressionParseErrorKind::ExpectedExpression { found } => ExpressionParseError::new(
                ExpressionParseErrorKind::ExpectedRightOperand {
                    operator: operator.to_string(),
                    found,
                },
                self.span,
            ),
            _ => self,
        }
    }
}

fn parse_token_expression(source: &str) -> Result<Expression, ExpressionParseError> {
    parse_token_expression_at(source, SourceSpan::new(None, 1, 1))
}

fn parse_token_expression_at(
    source: &str,
    base_span: SourceSpan,
) -> Result<Expression, ExpressionParseError> {
    let leading_whitespace = source.chars().take_while(|ch| ch.is_whitespace()).count();
    let source = source.trim();
    let base_span = SourceSpan::new(
        base_span.source_name,
        base_span.line,
        base_span.column + leading_whitespace,
    );
    let tokens = tokenize_expression_at(source, &base_span);
    TokenExpressionParser::new(&tokens, end_span(source, &base_span)).parse()
}

fn end_span(source: &str, base_span: &SourceSpan) -> SourceSpan {
    SourceSpan::new(
        base_span.source_name.clone(),
        base_span.line,
        base_span.column + source.chars().count(),
    )
}

fn describe_token_kind(kind: &ExpressionTokenKind) -> String {
    match kind {
        ExpressionTokenKind::Identifier(value)
        | ExpressionTokenKind::IntLiteral(value)
        | ExpressionTokenKind::FloatLiteral(value)
        | ExpressionTokenKind::Operator(value) => value.clone(),
        ExpressionTokenKind::StringLiteral(_) => "string literal".to_string(),
        ExpressionTokenKind::OpenParen => "(".to_string(),
        ExpressionTokenKind::CloseParen => ")".to_string(),
        ExpressionTokenKind::OpenBracket => "[".to_string(),
        ExpressionTokenKind::CloseBracket => "]".to_string(),
        ExpressionTokenKind::OpenBrace => "{".to_string(),
        ExpressionTokenKind::CloseBrace => "}".to_string(),
        ExpressionTokenKind::Colon => ":".to_string(),
        ExpressionTokenKind::DoubleColon => "::".to_string(),
        ExpressionTokenKind::Dot => ".".to_string(),
        ExpressionTokenKind::Comma => ",".to_string(),
        ExpressionTokenKind::Arrow => "->".to_string(),
    }
}

fn unary_expression(operator: UnaryOperator, expression: Expression) -> Expression {
    match (operator, expression) {
        (UnaryOperator::Negate, Expression::NumberInt(value)) => Expression::NumberInt(-value),
        (UnaryOperator::Negate, Expression::NumberFloat(value)) => {
            Expression::NumberFloat(FloatLiteral::new(-value.value()))
        }
        (operator, expression) => Expression::Unary {
            operator,
            expression: Box::new(expression),
        },
    }
}

fn parse_string_expression(value: &str) -> Expression {
    let span = crate::source::SourceSpan::new(None, 1, 1);
    let Some(objects) = text::parse_inline_content(value, &span) else {
        return Expression::String(value.to_string());
    };
    let objects = flatten_string_expression_content(objects);

    if objects.len() == 1 {
        if let Object::Text(text) = &objects[0] {
            if text.text() == value {
                return Expression::String(value.to_string());
            }
        }
    }

    Expression::StringContent(ContentList::new(objects))
}

fn flatten_string_expression_content(objects: Vec<Object>) -> Vec<Object> {
    let mut flattened = Vec::new();
    for object in objects {
        match object {
            Object::ContentList(content) => flattened.extend(content.objects().iter().cloned()),
            other => flattened.push(other),
        }
    }
    flattened
}

pub(super) fn split_top_level_args(source: &str) -> Vec<&str> {
    scan::split_top_level_with_options(source, ',', scan::ScanOptions::expression())
}

fn is_path_identifier(source: &str) -> bool {
    source.split('.').all(is_identifier)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(source: &str) -> String {
        let expression = parse_initial_expression(source)
            .unwrap_or_else(|| panic!("expected expression for {source:?}"));
        let mut output = String::new();
        expression.write_parse_snapshot(&mut output, 0);
        output
    }

    fn token_parser_snapshot(source: &str) -> String {
        let expression = parse_token_expression(source)
            .unwrap_or_else(|_| panic!("expected token parser expression for {source:?}"));
        let mut output = String::new();
        expression.write_parse_snapshot(&mut output, 0);
        output
    }

    fn expression(source: &str) -> Expression {
        parse_initial_expression(source)
            .unwrap_or_else(|| panic!("expected expression for {source:?}"))
    }

    fn token_parser_error(source: &str) -> (String, usize) {
        let error = parse_token_expression(source)
            .expect_err("expected token parser to report a structured expression error");
        (error.message(), error.span.column)
    }

    fn token(kind: ExpressionTokenKind, byte_index: usize, column: usize) -> ExpressionToken {
        ExpressionToken {
            kind,
            byte_index,
            span: SourceSpan::new(None, 1, column),
        }
    }

    #[test]
    fn parses_current_expression_behavior_baseline() {
        let cases = [
            (
                "1 + 2 * 3",
                "Binary(+, Number(1), Binary(*, Number(2), Number(3)))",
            ),
            (
                "(1 + 2) * 3",
                "Binary(*, Binary(+, Number(1), Number(2)), Number(3))",
            ),
            (
                "1 - 2 - 3",
                "Binary(-, Binary(-, Number(1), Number(2)), Number(3))",
            ),
            (
                "8 / 4 / 2",
                "Binary(/, Binary(/, Number(8), Number(4)), Number(2))",
            ),
            ("8 mod 3", "Binary(%, Number(8), Number(3))"),
            ("8 % 3", "Binary(%, Number(8), Number(3))"),
            ("-5", "Number(-5)"),
            ("-x", "Unary(-, VariableReference(x))"),
            ("not ready", "Unary(not, VariableReference(ready))"),
            ("!ready", "Unary(not, VariableReference(ready))"),
            ("notebook", "VariableReference(notebook)"),
            ("true", "Number(true)"),
            ("false", "Number(false)"),
            ("3.5", "Number(3.5)"),
            (r#""hello""#, r#"String("hello")"#),
            ("foo(1, bar(2, 3), \"x,y\")", "FunctionCall(foo, args=3)"),
            ("[]", "ArrayLiteral()"),
            (
                "[1, true, \"x\"]",
                r#"ArrayLiteral(Number(1), Number(true), String("x"))"#,
            ),
            (
                "[[], [1, 2], [foo(3 + 4)]]",
                "ArrayLiteral(ArrayLiteral(), ArrayLiteral(Number(1), Number(2)), ArrayLiteral(FunctionCall(foo, args=1)))",
            ),
            (
                "[player, companion]",
                "ArrayLiteral(VariableReference(player), VariableReference(companion))",
            ),
            ("{}", "StructLiteral()"),
            (
                "{ hp: 10, name: \"Ada\" }",
                r#"StructLiteral(hp=Number(10), name=String("Ada"))"#,
            ),
            (
                "{ stats: { hp: 10 }, inventory: [] }",
                "StructLiteral(stats=StructLiteral(hp=Number(10)), inventory=ArrayLiteral())",
            ),
            ("-> knot.stitch", "DivertTarget(-> knot.stitch)"),
            ("items::sword", "QualifiedReference(items::sword)"),
            ("audio::play(\"hit\")", "QualifiedFunctionCall(audio::play, args=1)"),
            (
                "{route}::target",
                "DynamicInterfaceAccess(VariableReference(route), target)",
            ),
            (
                "{route}::score(3)",
                "DynamicInterfaceFunctionCall(VariableReference(route), score, args=1)",
            ),
            (
                "{route.next}::score(amount + 1)",
                "DynamicInterfaceFunctionCall(VariableReference(route.next), score, args=1)",
            ),
            ("-> items::open", "DivertTarget(-> items::open)"),
            ("state.hp", "VariableReference(state.hp)"),
            ("state.stats.hp", "VariableReference(state.stats.hp)"),
            ("knot.stitch.label", "VariableReference(knot.stitch.label)"),
            (
                "items[0]",
                "IndexAccess(VariableReference(items), Number(0))",
            ),
            (
                "items[i]",
                "IndexAccess(VariableReference(items), VariableReference(i))",
            ),
            (
                "party[0].hp",
                "FieldAccess(IndexAccess(VariableReference(party), Number(0)), hp)",
            ),
            (
                "matrix[0][i]",
                "IndexAccess(IndexAccess(VariableReference(matrix), Number(0)), VariableReference(i))",
            ),
            (
                "a and b or c",
                "Binary(or, Binary(and, VariableReference(a), VariableReference(b)), VariableReference(c))",
            ),
            (
                "a && b || c",
                "Binary(||, Binary(&&, VariableReference(a), VariableReference(b)), VariableReference(c))",
            ),
            (
                "list ? item",
                "Binary(?, VariableReference(list), VariableReference(item))",
            ),
            (
                "list hasnt item",
                "Binary(!?, VariableReference(list), VariableReference(item))",
            ),
            (
                "a >= b",
                "Binary(>=, VariableReference(a), VariableReference(b))",
            ),
            (
                "1 < 2 == true",
                "Binary(==, Binary(<, Number(1), Number(2)), Number(true))",
            ),
        ];

        for (source, expected) in cases {
            assert_eq!(snapshot(source), expected, "source: {source}");
        }
    }

    #[test]
    fn tokenizer_covers_expression_token_categories() {
        let tokens = tokenize_expression(r#"foo([1, 2.5], "a,b", -> knot, list ? item)"#);

        assert_eq!(
            tokens,
            vec![
                token(ExpressionTokenKind::Identifier("foo".to_string()), 0, 1),
                token(ExpressionTokenKind::OpenParen, 3, 4),
                token(ExpressionTokenKind::OpenBracket, 4, 5),
                token(ExpressionTokenKind::IntLiteral("1".to_string()), 5, 6),
                token(ExpressionTokenKind::Comma, 6, 7),
                token(ExpressionTokenKind::FloatLiteral("2.5".to_string()), 8, 9),
                token(ExpressionTokenKind::CloseBracket, 11, 12),
                token(ExpressionTokenKind::Comma, 12, 13),
                token(
                    ExpressionTokenKind::StringLiteral("a,b".to_string()),
                    14,
                    15
                ),
                token(ExpressionTokenKind::Comma, 19, 20),
                token(ExpressionTokenKind::Arrow, 21, 22),
                token(ExpressionTokenKind::Identifier("knot".to_string()), 24, 25),
                token(ExpressionTokenKind::Comma, 28, 29),
                token(ExpressionTokenKind::Identifier("list".to_string()), 30, 31),
                token(ExpressionTokenKind::Operator("?".to_string()), 35, 36),
                token(ExpressionTokenKind::Identifier("item".to_string()), 37, 38),
                token(ExpressionTokenKind::CloseParen, 41, 42),
            ]
        );
    }

    #[test]
    fn tokenizer_covers_dynamic_interface_separators() {
        let tokens = tokenize_expression("{route}::target");

        assert_eq!(
            tokens,
            vec![
                token(ExpressionTokenKind::OpenBrace, 0, 1),
                token(ExpressionTokenKind::Identifier("route".to_string()), 1, 2),
                token(ExpressionTokenKind::CloseBrace, 6, 7),
                token(ExpressionTokenKind::DoubleColon, 7, 8),
                token(ExpressionTokenKind::Identifier("target".to_string()), 9, 10),
            ]
        );
    }

    #[test]
    fn tokenizer_covers_module_qualified_separator() {
        let tokens = tokenize_expression("items::sword");

        assert_eq!(
            tokens,
            vec![
                token(ExpressionTokenKind::Identifier("items".to_string()), 0, 1),
                token(ExpressionTokenKind::DoubleColon, 5, 6),
                token(ExpressionTokenKind::Identifier("sword".to_string()), 7, 8),
            ]
        );
    }

    #[test]
    fn tokenizer_covers_struct_literal_tokens() {
        let tokens = tokenize_expression("{hp: 1}");

        assert_eq!(
            tokens,
            vec![
                token(ExpressionTokenKind::OpenBrace, 0, 1),
                token(ExpressionTokenKind::Identifier("hp".to_string()), 1, 2),
                token(ExpressionTokenKind::Colon, 3, 4),
                token(ExpressionTokenKind::IntLiteral("1".to_string()), 5, 6),
                token(ExpressionTokenKind::CloseBrace, 6, 7),
            ]
        );
    }

    #[test]
    fn parses_field_access_nodes() {
        let Expression::FieldAccess { base, field } = expression("state.hp") else {
            panic!("expected field access");
        };
        assert_eq!(field, "hp");
        assert!(matches!(*base, Expression::VariableReference(ref name) if name == "state"));

        let nested = expression("state.stats.hp");
        assert_eq!(nested.dotted_path().as_deref(), Some("state.stats.hp"));
        assert!(matches!(nested, Expression::FieldAccess { .. }));
    }

    #[test]
    fn tokenizer_keeps_word_operators_distinct_from_identifiers() {
        let tokens = tokenize_expression("not ready and notebook or list hasnt item mod 2");

        assert_eq!(
            tokens,
            vec![
                token(ExpressionTokenKind::Operator("not".to_string()), 0, 1),
                token(ExpressionTokenKind::Identifier("ready".to_string()), 4, 5),
                token(ExpressionTokenKind::Operator("and".to_string()), 10, 11),
                token(
                    ExpressionTokenKind::Identifier("notebook".to_string()),
                    14,
                    15
                ),
                token(ExpressionTokenKind::Operator("or".to_string()), 23, 24),
                token(ExpressionTokenKind::Identifier("list".to_string()), 26, 27),
                token(ExpressionTokenKind::Operator("hasnt".to_string()), 31, 32),
                token(ExpressionTokenKind::Identifier("item".to_string()), 37, 38),
                token(ExpressionTokenKind::Operator("mod".to_string()), 42, 43),
                token(ExpressionTokenKind::IntLiteral("2".to_string()), 46, 47),
            ]
        );
    }

    #[test]
    fn tokenizer_covers_symbol_operators_and_field_access_separators() {
        let tokens = tokenize_expression("a.b >= c && x != y || z <= 3");

        assert_eq!(
            tokens,
            vec![
                token(ExpressionTokenKind::Identifier("a".to_string()), 0, 1),
                token(ExpressionTokenKind::Dot, 1, 2),
                token(ExpressionTokenKind::Identifier("b".to_string()), 2, 3),
                token(ExpressionTokenKind::Operator(">=".to_string()), 4, 5),
                token(ExpressionTokenKind::Identifier("c".to_string()), 7, 8),
                token(ExpressionTokenKind::Operator("&&".to_string()), 9, 10),
                token(ExpressionTokenKind::Identifier("x".to_string()), 12, 13),
                token(ExpressionTokenKind::Operator("!=".to_string()), 14, 15),
                token(ExpressionTokenKind::Identifier("y".to_string()), 17, 18),
                token(ExpressionTokenKind::Operator("||".to_string()), 19, 20),
                token(ExpressionTokenKind::Identifier("z".to_string()), 22, 23),
                token(ExpressionTokenKind::Operator("<=".to_string()), 24, 25),
                token(ExpressionTokenKind::IntLiteral("3".to_string()), 27, 28),
            ]
        );
    }

    #[test]
    fn operator_rule_table_drill_covers_tokenizer_and_parser_lookup() {
        assert_eq!(
            match_operator("!? item"),
            Some("!?"),
            "symbol aliases should be recognized from the operator table"
        );
        assert_eq!(
            binary_operator_rule("hasnt").map(|rule| (rule.operator, rule.precedence)),
            Some((BinaryOperator::Hasnt, 4)),
            "word aliases should be parsed from the same operator table"
        );
        assert_eq!(
            token_parser_snapshot("list !? item"),
            "Binary(!?, VariableReference(list), VariableReference(item))"
        );
    }

    #[test]
    fn tokenizer_tracks_character_columns_for_unicode_prefixes() {
        let tokens = tokenize_expression("变量 + \"ok\"");

        assert_eq!(
            tokens,
            vec![
                token(ExpressionTokenKind::Identifier("变量".to_string()), 0, 1),
                token(ExpressionTokenKind::Operator("+".to_string()), 7, 4),
                token(ExpressionTokenKind::StringLiteral("ok".to_string()), 9, 6),
            ]
        );
    }

    #[test]
    fn token_parser_reproduces_current_expression_baseline() {
        let cases = [
            (
                "1 + 2 * 3",
                "Binary(+, Number(1), Binary(*, Number(2), Number(3)))",
            ),
            (
                "(1 + 2) * 3",
                "Binary(*, Binary(+, Number(1), Number(2)), Number(3))",
            ),
            (
                "1 - 2 - 3",
                "Binary(-, Binary(-, Number(1), Number(2)), Number(3))",
            ),
            (
                "8 / 4 / 2",
                "Binary(/, Binary(/, Number(8), Number(4)), Number(2))",
            ),
            ("8 mod 3", "Binary(%, Number(8), Number(3))"),
            ("8 % 3", "Binary(%, Number(8), Number(3))"),
            ("-5", "Number(-5)"),
            ("-x", "Unary(-, VariableReference(x))"),
            ("not ready", "Unary(not, VariableReference(ready))"),
            ("!ready", "Unary(not, VariableReference(ready))"),
            ("notebook", "VariableReference(notebook)"),
            ("true", "Number(true)"),
            ("false", "Number(false)"),
            ("3.5", "Number(3.5)"),
            (r#""hello""#, r#"String("hello")"#),
            ("foo(1, bar(2, 3), \"x,y\")", "FunctionCall(foo, args=3)"),
            ("[]", "ArrayLiteral()"),
            (
                "[1, true, \"x\"]",
                r#"ArrayLiteral(Number(1), Number(true), String("x"))"#,
            ),
            (
                "[[], [1, 2], [foo(3 + 4)]]",
                "ArrayLiteral(ArrayLiteral(), ArrayLiteral(Number(1), Number(2)), ArrayLiteral(FunctionCall(foo, args=1)))",
            ),
            (
                "[player, companion]",
                "ArrayLiteral(VariableReference(player), VariableReference(companion))",
            ),
            ("{}", "StructLiteral()"),
            (
                "{ hp: 10, name: \"Ada\" }",
                r#"StructLiteral(hp=Number(10), name=String("Ada"))"#,
            ),
            (
                "{ stats: { hp: 10 }, inventory: [] }",
                "StructLiteral(stats=StructLiteral(hp=Number(10)), inventory=ArrayLiteral())",
            ),
            ("-> knot.stitch", "DivertTarget(-> knot.stitch)"),
            ("items::sword", "QualifiedReference(items::sword)"),
            ("audio::play(\"hit\")", "QualifiedFunctionCall(audio::play, args=1)"),
            (
                "{route}::target",
                "DynamicInterfaceAccess(VariableReference(route), target)",
            ),
            (
                "{route}::score(3)",
                "DynamicInterfaceFunctionCall(VariableReference(route), score, args=1)",
            ),
            (
                "{route.next}::score(amount + 1)",
                "DynamicInterfaceFunctionCall(VariableReference(route.next), score, args=1)",
            ),
            ("-> items::open", "DivertTarget(-> items::open)"),
            ("state.hp", "VariableReference(state.hp)"),
            ("state.stats.hp", "VariableReference(state.stats.hp)"),
            ("knot.stitch.label", "VariableReference(knot.stitch.label)"),
            (
                "items[0]",
                "IndexAccess(VariableReference(items), Number(0))",
            ),
            (
                "items[i]",
                "IndexAccess(VariableReference(items), VariableReference(i))",
            ),
            (
                "party[0].hp",
                "FieldAccess(IndexAccess(VariableReference(party), Number(0)), hp)",
            ),
            (
                "matrix[0][i]",
                "IndexAccess(IndexAccess(VariableReference(matrix), Number(0)), VariableReference(i))",
            ),
            (
                "a and b or c",
                "Binary(or, Binary(and, VariableReference(a), VariableReference(b)), VariableReference(c))",
            ),
            (
                "a && b || c",
                "Binary(||, Binary(&&, VariableReference(a), VariableReference(b)), VariableReference(c))",
            ),
            (
                "list ? item",
                "Binary(?, VariableReference(list), VariableReference(item))",
            ),
            (
                "list hasnt item",
                "Binary(!?, VariableReference(list), VariableReference(item))",
            ),
            (
                "a >= b",
                "Binary(>=, VariableReference(a), VariableReference(b))",
            ),
            (
                "1 < 2 == true",
                "Binary(==, Binary(<, Number(1), Number(2)), Number(true))",
            ),
        ];

        for (source, expected) in cases {
            assert_eq!(token_parser_snapshot(source), expected, "source: {source}");
        }
    }

    #[test]
    fn token_parser_reports_structured_errors_with_spans() {
        let cases = [
            (
                "1 +",
                "expected expression after operator `+` before end of input",
                4,
            ),
            (
                "(1 + 2",
                "expected `)` to close parenthesized expression before end of input",
                7,
            ),
            (
                "->",
                "expected divert target after `->` before end of input",
                3,
            ),
            (
                "foo(1 2)",
                "expected `,` or `)` in call to `foo`, found `2`",
                7,
            ),
            (
                "[1 2]",
                "expected `,` or `]` in array literal, found `2`",
                4,
            ),
            (
                "{hp 1}",
                "expected `:` after struct literal field `hp`, found `1`",
                5,
            ),
            (
                "{hp: 1 mp: 2}",
                "expected `,` or `}` in struct literal, found `mp`",
                8,
            ),
            (
                "state.",
                "expected field name after `.` before end of input",
                7,
            ),
            (
                "items::",
                "expected symbol name after `items::` before end of input",
                8,
            ),
            (
                "items::123",
                "expected symbol name after `items::`, found `123`",
                8,
            ),
            (
                "{route}::",
                "expected dynamic interface member name after `::` before end of input",
                10,
            ),
            (
                "{route}::123",
                "expected dynamic interface member name after `::`, found `123`",
                10,
            ),
            (
                "items[0",
                "expected `]` to close index access before end of input",
                8,
            ),
            ("1 $ 2", "unexpected token `$` after expression", 3),
        ];

        for (source, message, column) in cases {
            assert_eq!(
                token_parser_error(source),
                (message.to_string(), column),
                "source: {source}"
            );
        }
    }

    #[test]
    fn parsed_qualified_names_preserve_spans() {
        let parsed = expression("items::sword");
        let Expression::QualifiedReference(name) = parsed else {
            panic!("expected qualified reference");
        };

        assert_eq!(name.module(), "items");
        assert_eq!(name.symbol(), "sword");
        assert_eq!(name.module_span().column, 1);
        assert_eq!(name.symbol_span().column, 8);
    }
}
