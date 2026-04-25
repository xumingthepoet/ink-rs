use crate::parsed::{Object, Weave};

pub(super) fn group_weave_content(objects: Vec<Object>) -> Vec<Object> {
    let base_depth = determine_base_depth(&objects);
    group_nested_weaves(objects, base_depth)
}

pub(super) fn weave_from_objects(objects: Vec<Object>) -> Weave {
    let base_depth = determine_base_depth(&objects);
    Weave::new(
        group_nested_weaves(objects, base_depth),
        base_depth.saturating_sub(1),
    )
}

fn group_nested_weaves(objects: Vec<Object>, base_depth: usize) -> Vec<Object> {
    let mut grouped = Vec::new();
    let mut index = 0;

    while index < objects.len() {
        if object_depth(&objects[index]).is_some_and(|depth| depth > base_depth) {
            let mut nested = Vec::new();
            while index < objects.len() {
                if object_depth(&objects[index]).is_some_and(|depth| depth <= base_depth) {
                    break;
                }
                nested.push(objects[index].clone());
                index += 1;
            }
            let nested_base_depth = determine_base_depth(&nested);
            grouped.push(Object::Weave(Weave::new(
                group_nested_weaves(nested, nested_base_depth),
                nested_base_depth.saturating_sub(1),
            )));
        } else {
            grouped.push(objects[index].clone());
            index += 1;
        }
    }

    grouped
}

fn determine_base_depth(objects: &[Object]) -> usize {
    objects.iter().find_map(object_depth).unwrap_or(1)
}

fn object_depth(object: &Object) -> Option<usize> {
    match object {
        Object::Choice(choice) => Some(choice.indentation_depth()),
        Object::Gather(gather) => Some(gather.indentation_depth()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parsed::{Gather, Text},
        source::SourceSpan,
    };

    use super::*;

    fn span() -> SourceSpan {
        SourceSpan::new(None, 1, 1)
    }

    fn gather(depth: usize) -> Object {
        Object::Gather(Gather::new(span(), depth))
    }

    fn text(value: &str) -> Object {
        Object::Text(Text::new(value, span()))
    }

    #[test]
    fn groups_nested_weave_points_under_their_base_depth() {
        let grouped = group_weave_content(vec![
            gather(1),
            text("root"),
            gather(2),
            text("nested"),
            gather(1),
        ]);

        assert_eq!(grouped.len(), 4);
        assert!(matches!(grouped[0], Object::Gather(_)));
        assert!(matches!(grouped[1], Object::Text(_)));
        assert!(matches!(grouped[3], Object::Gather(_)));

        let Object::Weave(nested) = &grouped[2] else {
            panic!("expected nested weave");
        };
        assert_eq!(nested.base_indent(), 1);
        assert_eq!(nested.content().len(), 2);
        assert!(matches!(nested.content()[0], Object::Gather(_)));
        assert!(matches!(nested.content()[1], Object::Text(_)));
    }
}
