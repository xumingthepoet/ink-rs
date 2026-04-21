use crate::{
    error::Diagnostic,
    parsed::{
        BinaryExpression, Divert, DivertTarget, FunctionCall, Identifier, List, Number, ObjectKind,
        ObjectRef, Path, StringExpression, UnaryExpression, VariableReference,
    },
};

use super::{character_set::CharacterSet, string_parser::StringParser};

#[derive(Debug)]
pub struct ExpressionParser {
    parser: StringParser,
}

#[derive(Debug, Clone, Copy)]
struct OperatorSpec {
    text: &'static str,
    precedence: u8,
    require_whitespace: bool,
}

const INFIX_OPERATORS: &[OperatorSpec] = &[
    OperatorSpec {
        text: "&&",
        precedence: 1,
        require_whitespace: false,
    },
    OperatorSpec {
        text: "||",
        precedence: 1,
        require_whitespace: false,
    },
    OperatorSpec {
        text: "and",
        precedence: 1,
        require_whitespace: true,
    },
    OperatorSpec {
        text: "or",
        precedence: 1,
        require_whitespace: true,
    },
    OperatorSpec {
        text: "==",
        precedence: 2,
        require_whitespace: false,
    },
    OperatorSpec {
        text: ">=",
        precedence: 2,
        require_whitespace: false,
    },
    OperatorSpec {
        text: "<=",
        precedence: 2,
        require_whitespace: false,
    },
    OperatorSpec {
        text: "!=",
        precedence: 2,
        require_whitespace: false,
    },
    OperatorSpec {
        text: "<",
        precedence: 2,
        require_whitespace: false,
    },
    OperatorSpec {
        text: ">",
        precedence: 2,
        require_whitespace: false,
    },
    OperatorSpec {
        text: "!?",
        precedence: 3,
        require_whitespace: false,
    },
    OperatorSpec {
        text: "hasnt",
        precedence: 3,
        require_whitespace: true,
    },
    OperatorSpec {
        text: "has",
        precedence: 3,
        require_whitespace: true,
    },
    OperatorSpec {
        text: "?",
        precedence: 3,
        require_whitespace: false,
    },
    OperatorSpec {
        text: "^",
        precedence: 3,
        require_whitespace: false,
    },
    OperatorSpec {
        text: "+",
        precedence: 4,
        require_whitespace: false,
    },
    OperatorSpec {
        text: "-",
        precedence: 5,
        require_whitespace: false,
    },
    OperatorSpec {
        text: "*",
        precedence: 6,
        require_whitespace: false,
    },
    OperatorSpec {
        text: "/",
        precedence: 7,
        require_whitespace: false,
    },
    OperatorSpec {
        text: "mod",
        precedence: 8,
        require_whitespace: true,
    },
    OperatorSpec {
        text: "%",
        precedence: 8,
        require_whitespace: false,
    },
];

impl ExpressionParser {
    pub fn new(input: impl Into<String>, source_filename: Option<impl Into<String>>) -> Self {
        let mut parser = StringParser::new(input);
        if let Some(source_filename) = source_filename {
            parser = parser.with_source_filename(source_filename);
        }

        Self { parser }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        self.parser.diagnostics()
    }

    pub fn had_error(&self) -> bool {
        self.parser.had_error()
    }

    pub fn parse_expression(&mut self) -> Option<ObjectRef> {
        let expression = Self::parse_binary_expression(&mut self.parser, 0)?;
        Self::skip_whitespace(&mut self.parser);

        if !self.parser.end_of_input() {
            self.parser
                .error("Unexpected trailing input after expression", false);
            return None;
        }

        Some(expression)
    }

    fn parse_binary_expression(
        parser: &mut StringParser,
        minimum_precedence: u8,
    ) -> Option<ObjectRef> {
        parser.parse_object(|parser| {
            let mut expr = Self::parse_unary_expression(parser)?;

            loop {
                Self::skip_whitespace(parser);

                let Some(op) = Self::peek_infix_operator(parser) else {
                    break;
                };

                if op.precedence <= minimum_precedence {
                    break;
                }

                parser.parse_string(op.text)?;

                if op.require_whitespace && !Self::skip_whitespace(parser) {
                    parser.error(format!("Expected whitespace after '{0}'", op.text), false);
                    return None;
                }

                let right = Self::parse_binary_expression(parser, op.precedence)?;
                expr = BinaryExpression::new(expr, right, op.text).object();
            }

            Some(expr)
        })
    }

    fn parse_unary_expression(parser: &mut StringParser) -> Option<ObjectRef> {
        parser.parse_object(|parser| {
            Self::skip_whitespace(parser);

            if parser.remaining_string().starts_with("->") {
                return Self::parse_divert_target(parser);
            }

            if let Some(op) = Self::try_parse_prefix_operator(parser) {
                let inner = Self::parse_unary_expression(parser)?;
                return Some(UnaryExpression::new(inner, op).object());
            }

            let mut expr = Self::parse_primary(parser)?;
            Self::skip_whitespace(parser);

            if let Some(op) = Self::try_parse_postfix_operator(parser) {
                if let Some(identifier) = Self::single_identifier_for_variable_reference(&expr) {
                    expr =
                        crate::parsed::IncDecExpression::new(identifier, op == "++", None).object();
                } else {
                    parser.error(
                        "can only increment and decrement variables, but saw expression",
                        false,
                    );
                }
            }

            Some(expr)
        })
    }

    fn parse_primary(parser: &mut StringParser) -> Option<ObjectRef> {
        parser.parse_object(|parser| {
            Self::skip_whitespace(parser);

            if parser.end_of_input() {
                return None;
            }

            if parser.remaining_string().starts_with("\"") {
                return Self::parse_string_expression(parser);
            }

            if parser.remaining_string().starts_with("(") {
                if let Some(list_expression) = Self::parse_list_expression(parser) {
                    return Some(list_expression);
                }

                return Self::parse_paren_expression(parser);
            }

            if parser.remaining_string().starts_with("->") {
                return Self::parse_divert_target(parser);
            }

            let current_character = parser.current_character();
            if current_character.is_ascii_digit() {
                if let Some(number) = Self::parse_number(parser) {
                    return Some(number);
                }
            }

            if current_character.is_alphabetic() || current_character == '_' {
                if let Some(expression) = Self::parse_identifier_expression(parser) {
                    return Some(expression);
                }
            }

            None
        })
    }

    fn parse_identifier_expression(parser: &mut StringParser) -> Option<ObjectRef> {
        let path = Self::parse_identifier_path(parser)?;
        let is_bool_literal = path.len() == 1
            && matches!(
                path.first().map(|identifier| identifier.name.as_str()),
                Some("true" | "false")
            );

        Self::skip_whitespace(parser);

        if path.len() == 1 && parser.remaining_string().starts_with("(") {
            return Self::parse_function_call(parser, path.into_iter().next().unwrap());
        }

        if is_bool_literal {
            let value = path
                .first()
                .map(|identifier| identifier.name.as_str() == "true")
                .unwrap_or(false);
            return Some(Number::bool(value).object());
        }

        Some(VariableReference::new(path).object())
    }

    fn parse_function_call(
        parser: &mut StringParser,
        function_name: Identifier,
    ) -> Option<ObjectRef> {
        parser.parse_object(|parser| {
            parser.parse_string("(")?;
            Self::skip_whitespace(parser);

            let mut arguments = Vec::new();
            if parser.parse_string(")").is_some() {
                return Some(FunctionCall::new(function_name.clone(), arguments).object());
            }

            loop {
                let argument = Self::parse_binary_expression(parser, 0)?;
                arguments.push(argument);
                Self::skip_whitespace(parser);

                if parser.parse_string(",").is_some() {
                    Self::skip_whitespace(parser);
                    continue;
                }

                break;
            }

            Self::skip_whitespace(parser);
            parser.parse_string(")")?;

            Some(FunctionCall::new(function_name.clone(), arguments).object())
        })
    }

    fn parse_list_expression(parser: &mut StringParser) -> Option<ObjectRef> {
        parser.parse_object(|parser| {
            parser.parse_string("(")?;
            Self::skip_whitespace(parser);

            if parser.parse_string(")").is_some() {
                return Some(List::new(Vec::new()).object());
            }

            let mut items = Vec::new();
            loop {
                let item = Self::parse_identifier_member(parser)?;
                items.push(item);
                Self::skip_whitespace(parser);

                if parser.parse_string(",").is_some() {
                    Self::skip_whitespace(parser);
                    continue;
                }

                break;
            }

            if items.len() < 2 {
                return None;
            }

            Self::skip_whitespace(parser);
            parser.parse_string(")")?;

            Some(List::new(items).object())
        })
    }

    fn parse_paren_expression(parser: &mut StringParser) -> Option<ObjectRef> {
        parser.parse_object(|parser| {
            parser.parse_string("(")?;
            let expression = Self::parse_binary_expression(parser, 0)?;
            Self::skip_whitespace(parser);
            parser.parse_string(")")?;
            Some(expression)
        })
    }

    fn parse_divert_target(parser: &mut StringParser) -> Option<ObjectRef> {
        parser.parse_object(|parser| {
            parser.parse_string("->")?;
            Self::skip_whitespace(parser);

            let path = Self::parse_identifier_path(parser)?;
            Some(DivertTarget::new(Divert::new(Some(Path::new(path)))).object())
        })
    }

    fn parse_string_expression(parser: &mut StringParser) -> Option<ObjectRef> {
        parser.parse_object(|parser| {
            parser.parse_string("\"")?;
            let mut text = String::new();

            while !parser.end_of_input() {
                if parser.remaining_string().starts_with("\"") {
                    parser.parse_string("\"")?;
                    let content = if text.is_empty() {
                        Vec::new()
                    } else {
                        vec![crate::parsed::Text::new(text).object()]
                    };
                    return Some(StringExpression::new(content).object());
                }

                let current = parser.parse_single_character()?;
                if current == '\\' && !parser.end_of_input() {
                    let escaped = parser.parse_single_character()?;
                    match escaped {
                        '"' | '\\' => text.push(escaped),
                        other => {
                            text.push('\\');
                            text.push(other);
                        }
                    }
                } else {
                    text.push(current);
                }
            }

            parser.error("Unterminated string expression", false);
            None
        })
    }

    fn parse_number(parser: &mut StringParser) -> Option<ObjectRef> {
        parser.parse_object(|parser| {
            let mut token = String::new();
            let mut has_decimal_point = false;

            while !parser.end_of_input() {
                let current = parser.current_character();
                if current.is_ascii_digit() {
                    token.push(parser.parse_single_character()?);
                    continue;
                }

                if current == '.' && !has_decimal_point {
                    has_decimal_point = true;
                    token.push(parser.parse_single_character()?);
                    continue;
                }

                break;
            }

            if token.is_empty() {
                return None;
            }

            if token == "." {
                return None;
            }

            if has_decimal_point {
                let value = token.parse::<f64>().ok()?;
                Some(Number::float(value).object())
            } else {
                let value = token.parse::<i64>().ok()?;
                Some(Number::int(value).object())
            }
        })
    }

    fn parse_identifier_path(parser: &mut StringParser) -> Option<Vec<Identifier>> {
        parser.parse_object(|parser| {
            let mut path = Vec::new();

            let first_identifier = Self::parse_identifier(parser)?;
            path.push(first_identifier);

            loop {
                Self::skip_whitespace(parser);
                if parser.parse_string(".").is_none() {
                    break;
                }

                Self::skip_whitespace(parser);
                let next_identifier = Self::parse_identifier(parser)?;
                path.push(next_identifier);
            }

            Some(path)
        })
    }

    fn parse_identifier_member(parser: &mut StringParser) -> Option<Identifier> {
        Self::parse_identifier_path(parser)
            .map(|path| Identifier::new(Self::join_identifier_path(&path)))
    }

    fn parse_identifier(parser: &mut StringParser) -> Option<Identifier> {
        parser.parse_object(|parser| {
            let current = parser.current_character();
            if !(current.is_alphabetic() || current == '_') {
                return None;
            }

            let mut name = String::new();
            name.push(parser.parse_single_character()?);

            while !parser.end_of_input() {
                let current = parser.current_character();
                if current.is_alphanumeric() || current == '_' {
                    name.push(parser.parse_single_character()?);
                } else {
                    break;
                }
            }

            Some(Identifier::new(name))
        })
    }

    fn try_parse_prefix_operator(parser: &mut StringParser) -> Option<&'static str> {
        for op in ["not", "-", "!"] {
            if Self::starts_with_operator(parser, op) {
                parser.parse_string(op)?;
                return Some(op);
            }
        }

        None
    }

    fn try_parse_postfix_operator(parser: &mut StringParser) -> Option<&'static str> {
        for op in ["++", "--"] {
            if Self::starts_with_operator(parser, op) {
                parser.parse_string(op)?;
                return Some(op);
            }
        }

        None
    }

    fn peek_infix_operator(parser: &StringParser) -> Option<OperatorSpec> {
        for op in INFIX_OPERATORS {
            if Self::starts_with_operator(parser, op.text) {
                return Some(*op);
            }
        }

        None
    }

    fn starts_with_operator(parser: &StringParser, op: &str) -> bool {
        let remaining = parser.remaining_string();
        if !remaining.starts_with(op) {
            return false;
        }

        let next_character = remaining.chars().nth(op.chars().count());
        if matches!(op, "and" | "or" | "has" | "hasnt" | "mod" | "not") {
            if next_character
                .map(|character| character.is_alphanumeric() || character == '_')
                .unwrap_or(false)
            {
                return false;
            }
        }

        true
    }

    fn skip_whitespace(parser: &mut StringParser) -> bool {
        parser
            .parse_characters_from_char_set(&Self::whitespace_characters(), true, None)
            .is_some()
    }

    fn whitespace_characters() -> CharacterSet {
        CharacterSet::from(" \t\r\n")
    }

    fn single_identifier_for_variable_reference(object: &ObjectRef) -> Option<Identifier> {
        match object.borrow().kind() {
            ObjectKind::Expression {
                kind: crate::parsed::ExpressionKind::VariableReference { path, .. },
            } if path.len() == 1 => path.first().cloned(),
            _ => None,
        }
    }

    fn join_identifier_path(path: &[Identifier]) -> String {
        path.iter()
            .map(|identifier| identifier.name.clone())
            .collect::<Vec<_>>()
            .join(".")
    }
}

impl ExpressionParser {
    pub fn new_with_source_filename(
        input: impl Into<String>,
        source_filename: Option<impl Into<String>>,
    ) -> Self {
        Self::new(input, source_filename)
    }
}

#[cfg(test)]
mod tests {
    use crate::parsed::{ExpressionKind, ObjectKind};

    use super::ExpressionParser;

    fn render_expression(object: &crate::parsed::ObjectRef) -> String {
        let borrowed = object.borrow();

        match borrowed.kind() {
            ObjectKind::Expression {
                kind: ExpressionKind::Number(value),
            } => format!("Number({value:?})"),
            ObjectKind::Expression {
                kind: ExpressionKind::StringExpression,
            } => {
                let parts = borrowed
                    .content()
                    .iter()
                    .map(render_expression)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("StringExpression([{parts}])")
            }
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
            ObjectKind::Text { text } => format!("Text({text:?})"),
            ObjectKind::Divert {
                target,
                is_empty,
                is_tunnel,
                is_thread,
            } => {
                let target = target
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "->".to_string());
                format!(
                    "Divert(target={target}, empty={is_empty}, tunnel={is_tunnel}, thread={is_thread})"
                )
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
            } => format!(
                "IncDec({}, {identifier})",
                if *is_inc { "++" } else { "--" }
            ),
            other => format!("{other:?}"),
        }
    }

    #[test]
    fn expression_parser_parses_basic_expression_forms() {
        let cases = [
            (
                "1 + 2 * 3",
                "Binary(+, Number(Int(1)), Binary(*, Number(Int(2)), Number(Int(3))))",
            ),
            ("not foo", "Unary(not, VariableReference(foo))"),
            ("call(1, x)", "FunctionCall(call, args=2)"),
            ("(a, b)", "List(a, b)"),
            ("-> knot.stitch", "DivertTarget(-> knot.stitch)"),
            ("\"hello\"", "StringExpression([Text(\"hello\")])"),
        ];

        for (source, expected) in cases {
            let mut parser = ExpressionParser::new(source, Some("story.ink"));
            let expression = parser
                .parse_expression()
                .expect("expected expression to parse");

            assert!(
                parser.diagnostics().is_empty(),
                "{source:?} produced diagnostics"
            );
            assert_eq!(render_expression(&expression), expected, "case {source}");
        }
    }
}
