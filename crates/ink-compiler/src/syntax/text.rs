use crate::parsed::{Divert, DivertTarget, Object, Text};

use super::rule::RuleParser;

pub(super) fn parse_text_line(parser: &mut RuleParser<'_>) -> Option<Vec<Object>> {
    let span = parser.current_span();
    let text = parser.line_remainder().trim_start().to_string();
    parser.skip_to_end();

    if text.is_empty() {
        return None;
    }

    if has_unsupported_text_syntax(&text) {
        return None;
    }

    let mut objects = Vec::new();

    if let Some((text_content, divert_target)) = split_inline_divert(&text) {
        objects.push(Object::Text(Text::new(text_content, span.clone())));
        objects.push(Object::Divert(Divert::new(
            DivertTarget::from_source(divert_target),
            span.clone(),
        )));
    } else {
        objects.push(Object::Text(Text::new(text, span.clone())));
    }

    objects.push(Object::Text(Text::new("\n", span)));
    Some(objects)
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

fn split_inline_divert(text: &str) -> Option<(&str, &str)> {
    let (prefix, target) = text.rsplit_once("->")?;
    if prefix.trim().is_empty() || target.trim().is_empty() {
        return None;
    }

    Some((prefix, target))
}
