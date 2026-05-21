use crate::parsed::{Object, Weave};

use super::for_loop::{dynamic_choice_aliases, rewrite_choice_own_dynamic_aliases, rewrite_object};

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

    rewrite_dynamic_choice_scopes(grouped)
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

fn rewrite_dynamic_choice_scopes(objects: Vec<Object>) -> Vec<Object> {
    let mut objects = objects
        .into_iter()
        .map(rewrite_nested_dynamic_choice_scopes)
        .collect::<Vec<_>>();

    let mut index = 0;
    while index < objects.len() {
        let Some(aliases) = dynamic_choice_scope_aliases(&objects[index]) else {
            index += 1;
            continue;
        };

        let Object::Choice(choice) = objects[index].clone() else {
            unreachable!();
        };
        objects[index] = Object::Choice(rewrite_choice_own_dynamic_aliases(choice));

        let mut body_index = index + 1;
        while body_index < objects.len()
            && !matches!(objects[body_index], Object::Choice(_) | Object::Gather(_))
        {
            objects[body_index] = rewrite_nested_dynamic_choice_scopes(rewrite_object(
                objects[body_index].clone(),
                &aliases,
            ));
            body_index += 1;
        }

        index += 1;
    }

    objects
}

fn rewrite_nested_dynamic_choice_scopes(object: Object) -> Object {
    match object {
        Object::Weave(weave) => Object::Weave(Weave::new(
            rewrite_dynamic_choice_scopes(weave.content().to_vec()),
            weave.base_indent(),
        )),
        other => other,
    }
}

fn dynamic_choice_scope_aliases(object: &Object) -> Option<Vec<super::for_loop::LoopAlias>> {
    let Object::Choice(choice) = object else {
        return None;
    };
    choice.dynamic_binding().map(dynamic_choice_aliases)
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
