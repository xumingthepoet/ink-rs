use crate::{
    diagnostic::{Diagnostic, DiagnosticCode},
    source::SourceSpan,
};

use super::token::ExpressionTokenKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExpressionParseError {
    kind: ExpressionParseErrorKind,
    pub(super) span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ExpressionParseErrorKind {
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
    LegacyEmptyCompositeLiteral,
    LegacyStructLiteral,
    LegacyDictLiteral,
    ExpectedPercentLiteralTarget {
        found: Option<String>,
    },
    ExpectedPercentLiteralOpenBrace {
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
    ExpectedCompositeLiteralKey {
        found: Option<String>,
    },
    ExpectedDictLiteralKey {
        found: Option<String>,
    },
    ExpectedDictEntryColon {
        found: Option<String>,
    },
    ExpectedCommaOrDictCloseBrace {
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
    pub(super) fn new(kind: ExpressionParseErrorKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    pub(super) fn message(&self) -> String {
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
            ExpressionParseErrorKind::LegacyEmptyCompositeLiteral => {
                "Use `%Type{}` for structs or `%{}` for Dicts".to_string()
            }
            ExpressionParseErrorKind::LegacyStructLiteral => {
                "Struct literals now use `%Type{...}`".to_string()
            }
            ExpressionParseErrorKind::LegacyDictLiteral => {
                "Dict literals now use `%{...}`".to_string()
            }
            ExpressionParseErrorKind::ExpectedPercentLiteralTarget { found } => {
                format!(
                    "expected `{{` for a Dict literal or struct type name after `%`{}",
                    found_clause(found)
                )
            }
            ExpressionParseErrorKind::ExpectedPercentLiteralOpenBrace { found } => {
                format!(
                    "expected `{{` after struct literal type{}",
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
            ExpressionParseErrorKind::ExpectedCompositeLiteralKey { found } => {
                format!(
                    "expected struct field name or Dict literal key{}",
                    found_clause(found)
                )
            }
            ExpressionParseErrorKind::ExpectedDictLiteralKey { found } => {
                format!(
                    "expected string or int key in Dict literal{}",
                    found_clause(found)
                )
            }
            ExpressionParseErrorKind::ExpectedDictEntryColon { found } => {
                format!("expected `:` after Dict literal key{}", found_clause(found))
            }
            ExpressionParseErrorKind::ExpectedCommaOrDictCloseBrace { found } => {
                format!(
                    "expected `,` or `}}` in Dict literal{}",
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

    pub(super) fn into_diagnostic(self) -> Diagnostic {
        let message = self.message();
        Diagnostic::error(self.span, message).with_code(DiagnosticCode::InvalidExpression)
    }

    pub(super) fn for_right_operand(self, operator: &str) -> Self {
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

fn found_clause(found: &Option<String>) -> String {
    match found {
        Some(found) => format!(", found `{found}`"),
        None => " before end of input".to_string(),
    }
}

pub(super) fn describe_token_kind(kind: &ExpressionTokenKind) -> String {
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
