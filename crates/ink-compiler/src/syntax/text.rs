use crate::parsed::Text;

use super::rule::RuleParser;

pub(super) fn parse_text_line(parser: &mut RuleParser<'_>) -> Option<Vec<Text>> {
    let span = parser.current_span();
    let text = parser.take_to_end_trimmed()?;

    if has_unsupported_text_syntax(&text) {
        return None;
    }

    Some(vec![Text::new(text, span.clone()), Text::new("\n", span)])
}

fn has_unsupported_text_syntax(text: &str) -> bool {
    text.starts_with("INCLUDE ")
        || text.starts_with("VAR ")
        || text.starts_with("LIST ")
        || text.starts_with("CONST ")
        || text.starts_with("EXTERNAL ")
        || text.starts_with("===")
        || text.starts_with('=')
        || text.starts_with('*')
        || text.starts_with('+')
        || (text.starts_with('-') && !text.starts_with("->"))
        || text.starts_with('~')
        || text.contains('{')
        || text.contains('}')
        || text.contains("<>")
        || text.contains('#')
}
