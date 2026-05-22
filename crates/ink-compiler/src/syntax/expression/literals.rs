use crate::{
    parsed::{DictLiteralEntry, DictLiteralKey, Expression, StructLiteralField, TypeName},
    source::SourceSpan,
};

use super::super::is_identifier;
use super::{
    error::{describe_token_kind, ExpressionParseError, ExpressionParseErrorKind},
    parser::TokenExpressionParser,
    token::ExpressionTokenKind,
};

impl<'a> TokenExpressionParser<'a> {
    pub(super) fn parse_array_literal(&mut self) -> Result<Expression, ExpressionParseError> {
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

    pub(super) fn parse_braced_literal(&mut self) -> Result<Expression, ExpressionParseError> {
        if self.match_kind(|kind| matches!(kind, ExpressionTokenKind::CloseBrace)) {
            return Err(ExpressionParseError::new(
                ExpressionParseErrorKind::UnsupportedEmptyCompositeLiteral,
                self.previous_span(),
            ));
        }

        match self.peek().map(|token| &token.kind) {
            Some(ExpressionTokenKind::Identifier(_)) => Err(ExpressionParseError::new(
                ExpressionParseErrorKind::UnsupportedStructLiteral,
                self.peek().expect("kind came from peek").span.clone(),
            )),
            Some(ExpressionTokenKind::StringLiteral(_))
            | Some(ExpressionTokenKind::IntLiteral(_)) => Err(ExpressionParseError::new(
                ExpressionParseErrorKind::UnsupportedDictLiteral,
                self.peek().expect("kind came from peek").span.clone(),
            )),
            Some(ExpressionTokenKind::Operator(operator))
                if operator == "-" && self.next_token_is_int_literal() =>
            {
                Err(ExpressionParseError::new(
                    ExpressionParseErrorKind::UnsupportedDictLiteral,
                    self.peek().expect("kind came from peek").span.clone(),
                ))
            }
            Some(kind) => {
                let token = self.peek().expect("kind came from peek");
                Err(ExpressionParseError::new(
                    ExpressionParseErrorKind::ExpectedCompositeLiteralKey {
                        found: Some(describe_token_kind(kind)),
                    },
                    token.span.clone(),
                ))
            }
            None => Err(
                self.error_at_eof(ExpressionParseErrorKind::ExpectedCompositeLiteralKey {
                    found: None,
                }),
            ),
        }
    }

    pub(super) fn parse_percent_literal(
        &mut self,
        percent_span: SourceSpan,
    ) -> Result<Expression, ExpressionParseError> {
        if self.match_kind(|kind| matches!(kind, ExpressionTokenKind::OpenBrace)) {
            return self.parse_dict_literal();
        }

        let type_name = match self.peek().map(|token| token.kind.clone()) {
            Some(ExpressionTokenKind::Identifier(name)) => {
                let type_span = self.advance().expect("peek checked token").span.clone();
                if self.match_kind(|kind| matches!(kind, ExpressionTokenKind::DoubleColon)) {
                    TypeName::qualified_struct_type(
                        self.parse_qualified_name_after_module(&name, type_span)?,
                    )
                } else {
                    if !is_identifier(&name) {
                        return Err(ExpressionParseError::new(
                            ExpressionParseErrorKind::ExpectedPercentLiteralTarget {
                                found: Some(name),
                            },
                            type_span,
                        ));
                    }
                    TypeName::struct_type(name)
                }
            }
            Some(kind) => {
                let span = self.peek().expect("kind came from peek").span.clone();
                return Err(ExpressionParseError::new(
                    ExpressionParseErrorKind::ExpectedPercentLiteralTarget {
                        found: Some(describe_token_kind(&kind)),
                    },
                    span,
                ));
            }
            None => {
                return Err(ExpressionParseError::new(
                    ExpressionParseErrorKind::ExpectedPercentLiteralTarget { found: None },
                    percent_span,
                ));
            }
        };

        self.expect_kind(
            |kind| matches!(kind, ExpressionTokenKind::OpenBrace),
            |found| ExpressionParseErrorKind::ExpectedPercentLiteralOpenBrace { found },
        )?;
        self.parse_struct_literal(type_name)
    }

    fn parse_struct_literal(
        &mut self,
        type_name: TypeName,
    ) -> Result<Expression, ExpressionParseError> {
        let mut fields = Vec::new();
        if self.match_kind(|kind| matches!(kind, ExpressionTokenKind::CloseBrace)) {
            return Ok(Expression::StructLiteral { type_name, fields });
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

        Ok(Expression::StructLiteral { type_name, fields })
    }

    fn parse_dict_literal(&mut self) -> Result<Expression, ExpressionParseError> {
        let mut entries = Vec::new();
        if self.match_kind(|kind| matches!(kind, ExpressionTokenKind::CloseBrace)) {
            return Ok(Expression::DictLiteral(entries));
        }

        loop {
            let key = self.parse_dict_literal_key()?;
            self.expect_kind(
                |kind| matches!(kind, ExpressionTokenKind::Colon),
                |found| ExpressionParseErrorKind::ExpectedDictEntryColon { found },
            )?;
            let value = self.parse_expression(0)?;
            entries.push(DictLiteralEntry::new(key, value));

            if self.match_kind(|kind| matches!(kind, ExpressionTokenKind::Comma)) {
                continue;
            }
            self.expect_kind(
                |kind| matches!(kind, ExpressionTokenKind::CloseBrace),
                |found| ExpressionParseErrorKind::ExpectedCommaOrDictCloseBrace { found },
            )?;
            break;
        }

        Ok(Expression::DictLiteral(entries))
    }

    fn parse_dict_literal_key(&mut self) -> Result<DictLiteralKey, ExpressionParseError> {
        let Some(token) = self.advance() else {
            return Err(
                self.error_at_eof(ExpressionParseErrorKind::ExpectedDictLiteralKey { found: None })
            );
        };
        let kind = token.kind.clone();
        match kind {
            ExpressionTokenKind::StringLiteral(value) => Ok(DictLiteralKey::String(value)),
            ExpressionTokenKind::IntLiteral(value) => {
                value.parse::<i32>().map(DictLiteralKey::Int).map_err(|_| {
                    ExpressionParseError::new(
                        ExpressionParseErrorKind::InvalidIntegerLiteral { value },
                        token.span.clone(),
                    )
                })
            }
            ExpressionTokenKind::Operator(operator) if operator == "-" => {
                let Some(value_token) = self.advance() else {
                    return Err(self.error_at_eof(
                        ExpressionParseErrorKind::ExpectedDictLiteralKey { found: None },
                    ));
                };
                let value_kind = value_token.kind.clone();
                let ExpressionTokenKind::IntLiteral(value) = value_kind else {
                    return Err(ExpressionParseError::new(
                        ExpressionParseErrorKind::ExpectedDictLiteralKey {
                            found: Some(describe_token_kind(&value_kind)),
                        },
                        value_token.span.clone(),
                    ));
                };
                let parsed = value.parse::<i32>().map_err(|_| {
                    ExpressionParseError::new(
                        ExpressionParseErrorKind::InvalidIntegerLiteral {
                            value: format!("-{value}"),
                        },
                        token.span.clone(),
                    )
                })?;
                parsed
                    .checked_neg()
                    .map(DictLiteralKey::Int)
                    .ok_or_else(|| {
                        ExpressionParseError::new(
                            ExpressionParseErrorKind::InvalidIntegerLiteral {
                                value: format!("-{value}"),
                            },
                            token.span.clone(),
                        )
                    })
            }
            other => Err(ExpressionParseError::new(
                ExpressionParseErrorKind::ExpectedDictLiteralKey {
                    found: Some(describe_token_kind(&other)),
                },
                token.span.clone(),
            )),
        }
    }
}
