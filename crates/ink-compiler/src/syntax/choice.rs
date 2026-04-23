use crate::parsed::{Choice, ContentList};

use super::{rule::RuleParser, text};

pub(super) fn parse_choice(parser: &mut RuleParser<'_>) -> Option<Choice> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();

    // Match either '*' (once-only) or '+' (sticky)
    let once_only = if parser.match_string("*").is_some() {
        true
    } else if parser.match_string("+").is_some() {
        false
    } else {
        return None;
    };

    parser.skip_horizontal_whitespace();

    let choice_body = parser.expect(
        "choice text",
        |parser| {
            let text = parser.line_remainder().to_string();
            parser.skip_to_end();
            if text.is_empty() {
                None
            } else {
                Some(text)
            }
        },
        |parser| parser.skip_to_end(),
    )?;

    // Handle fallback choices like "* -> " which have no text content
    // and the -> is a divert to empty (fall through to gather)
    let trimmed_body = choice_body.trim();
    if trimmed_body == "->" || trimmed_body.is_empty() {
        // Invisible default choice - inner content is just a newline
        let inner = append_newline(ContentList::new(vec![]), span.clone());
        let mut choice = Choice::new_with_inline_brackets(
            None, // no start content
            None, // no choice-only content
            inner, span, false, // no inline brackets
        );
        choice.set_once_only(once_only);
        choice.set_is_invisible_default(true);
        return Some(choice);
    }

    let segments = parse_choice_segments(&choice_body)
        .map_err(|message| {
            parser.error(message);
        })
        .ok()?;

    let mut choice = Choice::new_with_inline_brackets(
        content_list_from_segment(segments.start, span.clone(), false),
        segments.choice_only.map(|segment| {
            content_list_from_segment(segment, span.clone(), true).unwrap_or_default()
        }),
        append_newline(
            content_list_from_segment(segments.inner, span.clone(), true).unwrap_or_default(),
            span.clone(),
        ),
        span,
        segments.has_inline_brackets,
    );
    choice.set_once_only(once_only);

    // Check if this is an invisible default (empty content)
    let is_invisible_default = !choice.has_start_content()
        && !choice.has_choice_only_content()
        && choice.inner_content().objects().len() <= 1; // Only newline
    choice.set_is_invisible_default(is_invisible_default);

    Some(choice)
}

struct ChoiceSegments {
    start: String,
    choice_only: Option<String>,
    inner: String,
    has_inline_brackets: bool,
}

fn parse_choice_segments(choice_body: &str) -> Result<ChoiceSegments, &'static str> {
    if choice_body.contains('{') || choice_body.contains('}') {
        return Err("unsupported syntax: choice");
    }

    let Some(open_index) = choice_body.find('[') else {
        if choice_body.contains(']') {
            return Err("Expected opening '[' for weave-style option but saw ']'");
        }

        return Ok(ChoiceSegments {
            start: choice_body.to_string(),
            choice_only: None,
            inner: String::new(),
            has_inline_brackets: false,
        });
    };

    let close_index = choice_body[open_index + 1..]
        .find(']')
        .map(|relative| open_index + 1 + relative)
        .ok_or("Expected closing ']' for weave-style option but saw end of line")?;

    if choice_body[open_index + 1..close_index].contains('[')
        || choice_body[open_index + 1..close_index].contains(']')
    {
        return Err("unsupported syntax: choice");
    }

    if choice_body[close_index + 1..].contains('[') || choice_body[close_index + 1..].contains(']')
    {
        return Err("unsupported syntax: choice");
    }

    Ok(ChoiceSegments {
        start: choice_body[..open_index].to_string(),
        choice_only: (!choice_body[open_index + 1..close_index].is_empty())
            .then(|| choice_body[open_index + 1..close_index].to_string()),
        inner: choice_body[close_index + 1..].to_string(),
        has_inline_brackets: true,
    })
}

fn content_list_from_segment(
    segment: String,
    span: crate::source::SourceSpan,
    keep_empty: bool,
) -> Option<ContentList> {
    if segment.is_empty() && !keep_empty {
        return None;
    }

    let objects = text::parse_inline_content(&segment, &span).unwrap_or_default();
    Some(ContentList::new(objects))
}

fn append_newline(mut content: ContentList, span: crate::source::SourceSpan) -> ContentList {
    content.push(crate::parsed::Object::Text(crate::parsed::Text::new(
        "\n", span,
    )));
    content
}
