#[cfg(test)]
use crate::parsed::BinaryOperator;
use crate::{
    diagnostic::Diagnostic,
    parsed::{ContentList, Expression, Object},
    source::SourceSpan,
};

use super::{is_identifier, scan, text};

mod error;
mod literals;
mod parser;
mod token;
mod tokenize;

use parser::{parse_token_expression, parse_token_expression_at};
#[cfg(test)]
use token::{binary_operator_rule, ExpressionToken, ExpressionTokenKind};
#[cfg(test)]
use tokenize::{match_operator, tokenize_expression};

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
            (
                "8 * 4 / 2",
                "Binary(/, Binary(*, Number(8), Number(4)), Number(2))",
            ),
            (
                "8 / 4 * 2",
                "Binary(*, Binary(/, Number(8), Number(4)), Number(2))",
            ),
            (
                "14 mod 5 % 3",
                "Binary(%, Binary(%, Number(14), Number(5)), Number(3))",
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
            ("%{}", "DictLiteral()"),
            (
                "%Player{ hp: 10, name: \"Ada\" }",
                r#"StructLiteral(Player, hp=Number(10), name=String("Ada"))"#,
            ),
            ("%Player{}", "StructLiteral(Player)"),
            ("%types::Player{}", "StructLiteral(types::Player)"),
            (
                "%Player{ stats: %Stats{ hp: 10 }, inventory: [] }",
                "StructLiteral(Player, stats=StructLiteral(Stats, hp=Number(10)), inventory=ArrayLiteral())",
            ),
            (r#"%{"a": 1}"#, r#"DictLiteral("a"=Number(1))"#),
            (r#"%{1: "a"}"#, r#"DictLiteral(1=String("a"))"#),
            (
                r#"%{"stats": %{1: "a"}}"#,
                r#"DictLiteral("stats"=DictLiteral(1=String("a")))"#,
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
        let tokens = tokenize_expression("%Player{hp: 1}");

        assert_eq!(
            tokens,
            vec![
                token(ExpressionTokenKind::Operator("%".to_string()), 0, 1),
                token(ExpressionTokenKind::Identifier("Player".to_string()), 1, 2),
                token(ExpressionTokenKind::OpenBrace, 7, 8),
                token(ExpressionTokenKind::Identifier("hp".to_string()), 8, 9),
                token(ExpressionTokenKind::Colon, 10, 11),
                token(ExpressionTokenKind::IntLiteral("1".to_string()), 12, 13),
                token(ExpressionTokenKind::CloseBrace, 13, 14),
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
            ("%{}", "DictLiteral()"),
            (
                "%Player{ hp: 10, name: \"Ada\" }",
                r#"StructLiteral(Player, hp=Number(10), name=String("Ada"))"#,
            ),
            ("%Player{}", "StructLiteral(Player)"),
            ("%types::Player{}", "StructLiteral(types::Player)"),
            (
                "%Player{ stats: %Stats{ hp: 10 }, inventory: [] }",
                "StructLiteral(Player, stats=StructLiteral(Stats, hp=Number(10)), inventory=ArrayLiteral())",
            ),
            (r#"%{"a": 1}"#, r#"DictLiteral("a"=Number(1))"#),
            (r#"%{1: "a"}"#, r#"DictLiteral(1=String("a"))"#),
            (
                r#"%{"stats": %{1: "a"}}"#,
                r#"DictLiteral("stats"=DictLiteral(1=String("a")))"#,
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
                "%Player{hp 1}",
                "expected `:` after struct literal field `hp`, found `1`",
                12,
            ),
            (
                "%Player{hp: 1 mp: 2}",
                "expected `,` or `}` in struct literal, found `mp`",
                15,
            ),
            (
                r#"%{"hp" 1}"#,
                "expected `:` after Dict literal key, found `1`",
                8,
            ),
            (
                r#"%{"hp": 1 mp: 2}"#,
                "expected `,` or `}` in Dict literal, found `mp`",
                11,
            ),
            (
                r#"%{"hp": 1, mp: 2}"#,
                "expected string or int key in Dict literal, found `mp`",
                12,
            ),
            (
                r#"%Player{hp: 1, "mp": 2}"#,
                "expected field name in struct literal, found `string literal`",
                16,
            ),
            ("{}", "Use `%Type{}` for structs or `%{}` for Dicts", 2),
            ("{ hp: 1 }", "Struct literals now use `%Type{...}`", 3),
            (r#"{"hp": 1}"#, "Dict literals now use `%{...}`", 2),
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
