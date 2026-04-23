use crate::parsed::{Choice, ContentList};

use super::{rule::RuleParser, text};

pub(super) fn parse_choice(parser: &mut RuleParser<'_>) -> Option<Choice> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    parser.match_string("*")?;
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

    let segments = parse_choice_segments(&choice_body)
        .map_err(|message| {
            parser.error(message);
        })
        .ok()?;

    Some(Choice::new(
        content_list_from_segment(segments.start, span.clone(), false),
        segments.choice_only.map(|segment| {
            content_list_from_segment(segment, span.clone(), true).unwrap_or_default()
        }),
        append_newline(
            content_list_from_segment(segments.inner, span.clone(), true).unwrap_or_default(),
            span.clone(),
        ),
        span,
    ))
}

struct ChoiceSegments {
    start: String,
    choice_only: Option<String>,
    inner: String,
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
        choice_only: Some(choice_body[open_index + 1..close_index].to_string()),
        inner: choice_body[close_index + 1..].to_string(),
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
