use crate::{
    parsed::{ContentList, Expression, Object, Text},
    source::SourceSpan,
};

use super::super::scan;

pub(super) fn parse_string_expression(value: &str) -> Expression {
    let span = SourceSpan::new(None, 1, 1);
    let Some(objects) = parse_string_content(value, &span) else {
        return Expression::String(value.to_string());
    };

    Expression::StringContent(ContentList::new(objects))
}

fn parse_string_content(value: &str, span: &SourceSpan) -> Option<Vec<Object>> {
    let mut remaining = value;
    let mut objects = Vec::new();

    while let Some(open_index) =
        scan::find_top_level_char_with_options(remaining, '{', scan::ScanOptions::inline_text())
    {
        if open_index > 0 {
            objects.push(Object::Text(Text::new(
                &remaining[..open_index],
                span.clone(),
            )));
        }

        let rest = &remaining[open_index + '{'.len_utf8()..];
        let close_index = scan::find_matching_delimiter(rest, '{', '}')?;
        let inner = &rest[..close_index];
        let expression = super::parse_initial_expression(inner.trim())?;
        objects.push(Object::Expression(expression));
        remaining = &rest[close_index + '}'.len_utf8()..];
    }

    if !remaining.is_empty() && !objects.is_empty() {
        objects.push(Object::Text(Text::new(remaining, span.clone())));
    }

    if objects.is_empty() {
        None
    } else {
        Some(objects)
    }
}

#[cfg(test)]
mod tests {
    use crate::parsed::{Object, Text};

    use super::*;

    fn text_object(value: &str) -> Object {
        Object::Text(Text::new(value, SourceSpan::new(None, 1, 1)))
    }

    #[test]
    fn plain_string_tokens_stay_literal() {
        assert_eq!(
            parse_string_expression("# <> -> <-"),
            Expression::String("# <> -> <-".to_string())
        );
    }

    #[test]
    fn interpolation_keeps_surrounding_inline_tokens_literal() {
        let expression = parse_string_expression("value {name} # <> -> <-");
        let Expression::StringContent(content) = expression else {
            panic!("expected interpolated string content");
        };

        assert_eq!(content.objects().len(), 3);
        assert_eq!(content.objects()[0], text_object("value "));
        assert_eq!(content.objects()[2], text_object(" # <> -> <-"));
        let Object::Expression(Expression::VariableReference(name)) = &content.objects()[1] else {
            panic!("expected interpolated variable reference");
        };
        assert_eq!(name, "name");
        assert!(!content.objects().iter().any(|object| {
            matches!(
                object,
                Object::Tag(_) | Object::Glue(_) | Object::Divert(_) | Object::TunnelOnwards(_)
            )
        }));
    }

    #[test]
    fn invalid_interpolation_keeps_whole_string_literal() {
        assert_eq!(
            parse_string_expression("{ready: yes} #"),
            Expression::String("{ready: yes} #".to_string())
        );
    }
}
