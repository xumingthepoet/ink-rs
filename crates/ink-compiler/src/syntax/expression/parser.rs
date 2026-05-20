use crate::{
    parsed::{BinaryOperator, Expression, FloatLiteral, QualifiedName, UnaryOperator},
    source::SourceSpan,
};

use super::super::is_identifier;
use super::{
    error::{describe_token_kind, ExpressionParseError, ExpressionParseErrorKind},
    is_path_identifier, parse_string_expression,
    token::{binary_operator_rule, ExpressionToken, ExpressionTokenKind},
    tokenize::tokenize_expression_at,
};

pub(super) struct TokenExpressionParser<'a> {
    pub(super) tokens: &'a [ExpressionToken],
    pub(super) index: usize,
    pub(super) eof_span: SourceSpan,
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

    pub(super) fn parse_expression(
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
                if operator == "%" && self.peek_is_percent_literal_target() =>
            {
                self.parse_percent_literal(token.span.clone())
            }
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
                    self.parse_braced_literal()
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

    pub(super) fn parse_argument_list(
        &mut self,
        name: &str,
    ) -> Result<Vec<Expression>, ExpressionParseError> {
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

    pub(super) fn parse_qualified_name_after_module(
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

    pub(super) fn peek(&self) -> Option<&'a ExpressionToken> {
        self.tokens.get(self.index)
    }

    pub(super) fn next_token_is_int_literal(&self) -> bool {
        self.tokens
            .get(self.index + 1)
            .is_some_and(|token| matches!(token.kind, ExpressionTokenKind::IntLiteral(_)))
    }

    fn peek_is_percent_literal_target(&self) -> bool {
        matches!(
            self.peek().map(|token| &token.kind),
            Some(ExpressionTokenKind::OpenBrace | ExpressionTokenKind::Identifier(_))
        )
    }

    pub(super) fn previous_span(&self) -> SourceSpan {
        self.tokens
            .get(self.index.saturating_sub(1))
            .map(|token| token.span.clone())
            .unwrap_or_else(|| self.eof_span.clone())
    }

    pub(super) fn advance(&mut self) -> Option<&'a ExpressionToken> {
        let token = self.peek()?;
        self.index += 1;
        Some(token)
    }

    pub(super) fn match_kind(
        &mut self,
        matches: impl FnOnce(&ExpressionTokenKind) -> bool,
    ) -> bool {
        if self.peek().is_some_and(|token| matches(&token.kind)) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    pub(super) fn expect_kind(
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

    pub(super) fn error_at_eof(&self, kind: ExpressionParseErrorKind) -> ExpressionParseError {
        ExpressionParseError::new(kind, self.eof_span.clone())
    }
}

pub(super) fn parse_token_expression(source: &str) -> Result<Expression, ExpressionParseError> {
    parse_token_expression_at(source, SourceSpan::new(None, 1, 1))
}

pub(super) fn parse_token_expression_at(
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
