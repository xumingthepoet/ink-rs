use crate::tags;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TextStyle {
    pub color: Option<String>,
}

impl TextStyle {
    fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledSegment {
    pub text: String,
    pub style: TextStyle,
}

pub fn parse_style_markup(input: &str) -> Vec<StyledSegment> {
    parse_style_markup_with_base(input, TextStyle::default())
}

pub fn parse_style_markup_with_base(input: &str, base_style: TextStyle) -> Vec<StyledSegment> {
    let mut segments = Vec::new();
    let mut style_stack = vec![base_style];
    let mut index = 0;

    while index < input.len() {
        let Some(open_offset) = input[index..].find('[') else {
            push_segment(&mut segments, &input[index..], current_style(&style_stack));
            break;
        };

        let open_index = index + open_offset;
        push_segment(
            &mut segments,
            &input[index..open_index],
            current_style(&style_stack),
        );

        let candidate = &input[open_index..];
        if candidate.starts_with("[/style]") {
            if style_stack.len() > 1 {
                style_stack.pop();
            }
            index = open_index + "[/style]".len();
            continue;
        }

        if let Some((tag_len, style)) =
            parse_opening_style_tag(candidate, current_style(&style_stack))
        {
            style_stack.push(style);
            index = open_index + tag_len;
            continue;
        }

        if is_incomplete_style_tag(candidate) {
            break;
        }

        push_segment(&mut segments, "[", current_style(&style_stack));
        index = open_index + '['.len_utf8();
    }

    segments
}

pub fn parse_style_markup_with_tags(
    input: &str,
    tags: &[String],
    enabled: bool,
) -> Vec<StyledSegment> {
    parse_style_markup_with_base(input, style_from_tags(tags, enabled))
}

pub fn strip_style_markup(input: &str) -> String {
    parse_style_markup(input)
        .into_iter()
        .map(|segment| segment.text)
        .collect()
}

pub fn tagged_style_markup(input: &str, tags: &[String], enabled: bool) -> String {
    let style = style_from_tags(tags, enabled);
    if style.is_default() {
        return input.to_string();
    }

    let Some(color) = style.color else {
        return input.to_string();
    };

    format!("[style color={color}]{input}[/style]")
}

pub fn style_from_tags(tags: &[String], enabled: bool) -> TextStyle {
    let mut style = TextStyle {
        color: tags::tag_value(tags, "color"),
    };

    if !enabled {
        style.color = Some("gray".to_string());
    }

    style
}

fn current_style(style_stack: &[TextStyle]) -> &TextStyle {
    style_stack.last().expect("style stack is never empty")
}

fn parse_opening_style_tag(
    candidate: &str,
    current_style: &TextStyle,
) -> Option<(usize, TextStyle)> {
    let rest = candidate.strip_prefix("[style")?;
    let next_char = rest.chars().next()?;
    if next_char != ']' && !next_char.is_whitespace() {
        return None;
    }

    let close_offset = candidate.find(']')?;
    let attributes = &candidate["[style".len()..close_offset];
    let mut style = current_style.clone();
    apply_style_attributes(attributes, &mut style);
    Some((close_offset + 1, style))
}

fn apply_style_attributes(attributes: &str, style: &mut TextStyle) {
    for attribute in attributes.split_whitespace() {
        let Some((key, value)) = attribute.split_once('=') else {
            continue;
        };

        if key.eq_ignore_ascii_case("color") {
            let value = normalize_attribute_value(value);
            style.color = (!value.is_empty()).then_some(value);
        }
    }
}

fn normalize_attribute_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn is_incomplete_style_tag(candidate: &str) -> bool {
    if candidate.contains(']') {
        return false;
    }

    let lower = candidate.to_ascii_lowercase();
    "[style".starts_with(lower.as_str())
        || "[/style]".starts_with(lower.as_str())
        || lower.starts_with("[style")
        || lower.starts_with("[/style")
}

fn push_segment(segments: &mut Vec<StyledSegment>, text: &str, style: &TextStyle) {
    if text.is_empty() {
        return;
    }

    if let Some(previous) = segments.last_mut() {
        if previous.style == *style {
            previous.text.push_str(text);
            return;
        }
    }

    segments.push(StyledSegment {
        text: text.to_string(),
        style: style.clone(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_color_style_segments() {
        let segments = parse_style_markup("this word uses [style color=red]red[/style] color.");

        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].text, "this word uses ");
        assert_eq!(segments[0].style.color, None);
        assert_eq!(segments[1].text, "red");
        assert_eq!(segments[1].style.color.as_deref(), Some("red"));
        assert_eq!(segments[2].text, " color.");
        assert_eq!(segments[2].style.color, None);
    }

    #[test]
    fn strip_style_markup_removes_tags() {
        assert_eq!(
            strip_style_markup("A [style color=red]red[/style] word"),
            "A red word"
        );
    }

    #[test]
    fn incomplete_style_tag_is_hidden_for_streaming_reveal() {
        assert_eq!(strip_style_markup("A [style color=red"), "A ");
        assert_eq!(strip_style_markup("A [style color=red]red[/sty"), "A red");
    }

    #[test]
    fn literal_non_style_brackets_are_preserved() {
        assert_eq!(strip_style_markup("Use [x] marker"), "Use [x] marker");
    }

    #[test]
    fn tag_style_is_default_and_inline_style_overrides_it() {
        let tags = vec!["color:blue".to_string()];
        let segments = parse_style_markup_with_tags(
            "this [style color=red]choice [/style]is blue.",
            &tags,
            true,
        );

        assert_eq!(segments[0].text, "this ");
        assert_eq!(segments[0].style.color.as_deref(), Some("blue"));
        assert_eq!(segments[1].text, "choice ");
        assert_eq!(segments[1].style.color.as_deref(), Some("red"));
        assert_eq!(segments[2].text, "is blue.");
        assert_eq!(segments[2].style.color.as_deref(), Some("blue"));
    }

    #[test]
    fn disabled_choice_defaults_to_gray() {
        let tags = vec!["color:blue".to_string(), "enabled:false".to_string()];
        let segments = parse_style_markup_with_tags("locked", &tags, false);

        assert_eq!(segments[0].style.color.as_deref(), Some("gray"));
    }
}
