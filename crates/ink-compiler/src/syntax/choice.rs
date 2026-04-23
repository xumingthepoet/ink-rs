use crate::parsed::{Choice, ContentList, Object, Text};

use super::rule::RuleParser;

pub(super) fn parse_choice(parser: &mut RuleParser<'_>) -> Option<Choice> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    parser.match_string("*")?;
    parser.skip_horizontal_whitespace();

    let choice_text = parser.expect(
        "choice text",
        |parser| parser.take_to_end_trimmed(),
        |parser| parser.skip_to_end(),
    )?;

    if has_unsupported_choice_syntax(&choice_text) {
        parser.error("unsupported syntax: choice");
        parser.skip_to_end();
        return None;
    }

    let start_content = ContentList::new(vec![Object::Text(Text::new(choice_text, span.clone()))]);
    let inner_content = ContentList::new(vec![Object::Text(Text::new("\n", span.clone()))]);

    Some(Choice::new(Some(start_content), None, inner_content, span))
}

fn has_unsupported_choice_syntax(choice_text: &str) -> bool {
    choice_text.contains('{')
        || choice_text.contains('}')
        || choice_text.contains('[')
        || choice_text.contains(']')
}
