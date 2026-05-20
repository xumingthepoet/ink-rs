use crate::{
    parsed::{ContentList, Expression, Object},
    source::SourceSpan,
};

use super::super::text;

pub(super) fn parse_string_expression(value: &str) -> Expression {
    let span = SourceSpan::new(None, 1, 1);
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
