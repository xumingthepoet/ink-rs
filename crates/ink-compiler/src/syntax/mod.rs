mod choice;
mod conditional;
mod declaration;
mod divert;
mod error;
mod expression;
mod gather;
mod knot;
mod logic;
mod rule;
mod scan;
mod sequence;
mod state;
mod text;
mod variable;
mod weave;

mod parser;

#[cfg(test)]
pub(crate) use parser::parse;
pub(crate) use parser::parse_source;

use crate::{
    parsed::{AuthorWarning, Choice, Object},
    source::SourceLine,
};

use self::rule::RuleParser;

fn parse_choice_from_line(line: &SourceLine) -> Option<Choice> {
    let mut line_parser = RuleParser::new(line);
    let choice = line_parser.parse_rule(choice::parse_choice);
    if line_parser.had_error() {
        None
    } else {
        choice
    }
}

fn choice_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    choice::parse_choice(parser).map(|choice| vec![Object::Choice(choice)])
}

fn author_warning_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    let identifier = parser.take_while(is_identifier_continue)?;
    if identifier != "TODO" {
        return None;
    }
    parser.skip_horizontal_whitespace();
    let _ = parser.match_string(":");
    parser.skip_horizontal_whitespace();
    let message = parser.line_remainder().trim_end().to_string();
    parser.skip_to_end();
    Some(vec![Object::AuthorWarning(AuthorWarning::new(
        message, span,
    ))])
}

fn leading_whitespace_count(source: &str) -> usize {
    source
        .chars()
        .take_while(|ch| ch.is_whitespace() && *ch != '\n' && *ch != '\r')
        .count()
}

fn is_choice_continuation_boundary(trimmed: &str) -> bool {
    trimmed.starts_with('*')
        || trimmed.starts_with('+')
        || trimmed.starts_with('-')
        || trimmed.starts_with('=')
        || trimmed.starts_with("->")
        || knot::is_knot_declaration_line(trimmed)
        || knot::is_stitch_declaration_line(trimmed)
}

pub(super) fn parse_initial_expression(source: &str) -> Option<crate::parsed::Expression> {
    expression::parse_initial_expression(source)
}

pub(super) fn split_top_level_args(source: &str) -> Vec<&str> {
    expression::split_top_level_args(source)
}

pub(super) fn is_identifier(source: &str) -> bool {
    !source.is_empty()
        && source.chars().all(is_identifier_continue)
        && source.chars().any(|ch| !ch.is_ascii_digit())
}

pub(super) fn is_identifier_start(ch: char) -> bool {
    is_identifier_continue(ch)
}

pub(super) fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric() || is_supported_unicode_identifier(ch)
}

fn is_supported_unicode_identifier(ch: char) -> bool {
    matches!(
        ch,
        '\u{0080}'..='\u{00ff}'
            | '\u{0100}'..='\u{017f}'
            | '\u{0180}'..='\u{024f}'
            | '\u{0370}'..='\u{03ff}'
            | '\u{0400}'..='\u{04ff}'
            | '\u{0531}'..='\u{0556}'
            | '\u{0561}'..='\u{0587}'
            | '\u{0590}'..='\u{05ff}'
            | '\u{0600}'..='\u{06ff}'
            | '\u{3041}'..='\u{3096}'
            | '\u{30a0}'..='\u{30fc}'
            | '\u{4e00}'..='\u{9fff}'
            | '\u{ac00}'..='\u{d7af}'
    ) && !matches!(ch, '\u{0374}' | '\u{0375}' | '\u{0378}'..='\u{0385}' | '\u{0387}' | '\u{038b}' | '\u{038d}' | '\u{03a2}' | '\u{0482}'..='\u{0489}')
}

fn divert_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    divert::parse_divert_objects(parser)
}

fn text_statement(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    text::parse_text_line(parser)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::SourceInput;

    #[test]
    fn parses_plain_text_lines() {
        let output = parse(SourceInput::new("Line.\nOther line."));
        assert!(output.diagnostics.is_empty());
        let story = output.artifact.unwrap();
        assert_eq!(story.root_weave().content().len(), 4);
        assert!(story.flows().is_empty());
    }

    #[test]
    fn parses_choice() {
        let output = parse(SourceInput::new("* Choice"));
        assert!(output.diagnostics.is_empty());
        let story = output.artifact.unwrap();
        assert_eq!(story.root_weave().content().len(), 1);
    }

    #[test]
    fn parses_choice_after_same_line_gather() {
        let output = parse(SourceInput::new("- * Choice"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();
        assert!(matches!(story.root_weave().content()[0], Object::Gather(_)));
        assert!(matches!(story.root_weave().content()[1], Object::Choice(_)));
    }

    #[test]
    fn parses_inline_choice_segments() {
        let output = parse(SourceInput::new("* Hello [back!] right back to you!"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::Choice(choice) = &story.root_weave().content()[0] else {
            panic!("expected choice");
        };
        assert_eq!(choice.start_content().unwrap().objects().len(), 1);
        assert_eq!(choice.choice_only_content().unwrap().objects().len(), 1);
        assert_eq!(choice.inner_content().objects().len(), 2);
        assert!(choice.has_weave_style_inline_brackets());
    }

    #[test]
    fn parses_choice_only_inline_choice() {
        let output = parse(SourceInput::new("* [Hello back!]"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::Choice(choice) = &story.root_weave().content()[0] else {
            panic!("expected choice");
        };
        assert!(!choice.has_start_content());
        assert!(choice.has_choice_only_content());
        assert_eq!(choice.inner_content().objects().len(), 1);
    }

    #[test]
    fn parses_choice_with_inner_divert() {
        let output = parse(SourceInput::new("* [Open the gate] -> paragraph_2"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::Choice(choice) = &story.root_weave().content()[0] else {
            panic!("expected choice");
        };
        assert!(matches!(
            choice.inner_content().objects()[1],
            Object::Divert(_)
        ));
    }

    #[test]
    fn parses_choice_condition_with_dotted_path() {
        let output = parse(SourceInput::new("* {knot.stitch.label} Text"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::Choice(choice) = &story.root_weave().content()[0] else {
            panic!("expected choice");
        };
        let Some(crate::parsed::Expression::VariableReference(name)) = choice.condition() else {
            panic!("expected variable reference condition");
        };
        assert_eq!(name, "knot.stitch.label");
    }

    #[test]
    fn parses_empty_inline_choice_brackets_without_choice_only_content() {
        let output = parse(SourceInput::new("* Text[] inner"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::Choice(choice) = &story.root_weave().content()[0] else {
            panic!("expected choice");
        };
        assert!(choice.has_weave_style_inline_brackets());
        assert!(!choice.has_choice_only_content());
    }

    #[test]
    fn parses_inline_divert_in_text() {
        let output = parse(SourceInput::new("A line. -> END"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        assert_eq!(story.root_weave().content().len(), 3);
        assert!(matches!(story.root_weave().content()[1], Object::Divert(_)));
    }

    #[test]
    fn parses_glue_and_inline_divert_in_text() {
        let output = parse(SourceInput::new("A line <> -> knot"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        assert!(matches!(story.root_weave().content()[0], Object::Text(_)));
        assert!(matches!(story.root_weave().content()[1], Object::Glue(_)));
        assert!(matches!(story.root_weave().content()[2], Object::Text(_)));
        assert!(matches!(story.root_weave().content()[3], Object::Divert(_)));
        assert!(matches!(story.root_weave().content()[4], Object::Text(_)));
    }

    #[test]
    fn trims_extra_separator_whitespace_before_terminal_divert() {
        let output = parse(SourceInput::new("<>as fast as we could.  -> END"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::Text(text) = &story.root_weave().content()[1] else {
            panic!("expected text");
        };
        assert_eq!(text.text(), "as fast as we could. ");
    }

    #[test]
    fn parses_knot_definition() {
        let output = parse(SourceInput::new(
            "Top line.\n-> knot_name\n\n== knot_name ===\nInside knot. -> END",
        ));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        assert_eq!(story.flows().len(), 1);
        assert_eq!(story.flows()[0].name(), "knot_name");
        assert_eq!(story.flows()[0].weave().content().len(), 3);
    }

    #[test]
    fn parses_identifiers_that_start_with_numbers() {
        assert!(is_identifier("2tests"));
        assert!(is_identifier("512x2"));
        assert!(!is_identifier("512"));

        let output = parse(SourceInput::new("== 2tests ==\n-> DONE"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();
        assert_eq!(story.flows()[0].name(), "2tests");
    }

    #[test]
    fn parses_postfix_increment_logic_line() {
        let output = parse(SourceInput::new("~ x++\n~ x--"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::IncDec(increment) = &story.root_weave().content()[0] else {
            panic!("expected increment");
        };
        assert_eq!(increment.name(), "x");
        assert!(increment.is_increment());

        let Object::IncDec(decrement) = &story.root_weave().content()[1] else {
            panic!("expected decrement");
        };
        assert_eq!(decrement.name(), "x");
        assert!(!decrement.is_increment());
    }

    #[test]
    fn parses_return_without_expression() {
        let output = parse(SourceInput::new("=== function f() ===\n~ return"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::Return(ret) = &story.flows()[0].weave().content()[0] else {
            panic!("expected return");
        };
        assert!(ret.returned_expression().is_none());
    }

    #[test]
    fn return_prefix_identifier_stays_expression() {
        let output = parse(SourceInput::new("~ returnValue()"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::LogicLine(crate::parsed::Expression::FunctionCall { name, args }) =
            &story.root_weave().content()[0]
        else {
            panic!("expected function-call logic line");
        };
        assert_eq!(name, "returnValue");
        assert!(args.is_empty());
    }

    #[test]
    fn parses_sticky_choice() {
        let output = parse(SourceInput::new("+ Choice"));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.unwrap();

        let Object::Choice(choice) = &story.root_weave().content()[0] else {
            panic!("expected choice");
        };
        assert!(!choice.once_only());
    }
}
