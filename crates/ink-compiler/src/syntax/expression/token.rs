use crate::{parsed::BinaryOperator, source::SourceSpan};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExpressionToken {
    pub(super) kind: ExpressionTokenKind,
    pub(super) byte_index: usize,
    pub(super) span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ExpressionTokenKind {
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
pub(super) struct BinaryOperatorRule {
    pub(super) text: &'static str,
    pub(super) operator: BinaryOperator,
    pub(super) precedence: u8,
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
        precedence: 6,
        token_kind: OperatorTokenKind::Symbol,
    },
    BinaryOperatorRule {
        text: "mod",
        operator: BinaryOperator::Modulo,
        precedence: 6,
        token_kind: OperatorTokenKind::Word,
    },
    BinaryOperatorRule {
        text: "%",
        operator: BinaryOperator::Modulo,
        precedence: 6,
        token_kind: OperatorTokenKind::Symbol,
    },
];

pub(super) fn symbol_operator_text(source: &str) -> Option<&'static str> {
    BINARY_OPERATOR_RULES
        .iter()
        .filter(|rule| rule.token_kind == OperatorTokenKind::Symbol)
        .map(|rule| rule.text)
        .find(|operator| source.starts_with(operator))
}

pub(super) fn binary_operator_rule(text: &str) -> Option<BinaryOperatorRule> {
    BINARY_OPERATOR_RULES
        .iter()
        .find(|rule| rule.text == text)
        .copied()
}
