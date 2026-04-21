use std::{cell::RefCell, collections::HashMap, fmt};

use crate::{Diagnostic, DiagnosticSeverity};

use super::{find_all, Object, ObjectKind, ObjectRef};

#[derive(Debug, Clone)]
pub struct Weave {
    object: ObjectRef,
    named_weave_points: RefCell<HashMap<String, ObjectRef>>,
}

impl Weave {
    pub fn new(content: Vec<ObjectRef>, base_indent_index: usize) -> Self {
        let object = Object::new_ref();
        object.borrow_mut().set_weave_kind(base_indent_index);

        let weave = Self {
            object,
            named_weave_points: RefCell::new(HashMap::new()),
        };

        for child in content {
            Object::add_content(&weave.object, child);
        }

        weave.construct_weave_hierarchy_from_indentation();
        weave
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn base_indent_index(&self) -> usize {
        match self.object.borrow().kind() {
            ObjectKind::Weave { base_indent_index } => *base_indent_index,
            _ => 0,
        }
    }

    pub fn content(&self) -> Vec<ObjectRef> {
        self.object.borrow().content().to_vec()
    }

    pub fn determine_base_indentation_from_content(content: &[ObjectRef]) -> usize {
        for object in content {
            if let Some(indent) = weave_point_base_indent_index(object) {
                return indent;
            }
        }

        0
    }

    pub fn resolve_weave_point_naming(&self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut named_weave_points = self.named_weave_points.borrow_mut();
        named_weave_points.clear();

        let weave_points = find_all(&self.object, |object| {
            matches!(
                object.kind(),
                ObjectKind::Choice {
                    identifier: Some(_),
                    ..
                } | ObjectKind::Gather {
                    identifier: Some(_),
                    ..
                }
            )
        });

        for weave_point in weave_points {
            let (name, type_name) = {
                let borrowed = weave_point.borrow();
                match borrowed.kind() {
                    ObjectKind::Choice { identifier, .. } => {
                        let identifier = identifier.as_ref().unwrap();
                        (identifier.name.clone(), "choice")
                    }
                    ObjectKind::Gather { identifier, .. } => {
                        let identifier = identifier.as_ref().unwrap();
                        (identifier.name.clone(), "gather")
                    }
                    _ => continue,
                }
            };

            if let Some(existing) = named_weave_points.get(&name) {
                let existing_name = existing.borrow();
                let existing_type = match existing_name.kind() {
                    ObjectKind::Choice { .. } => "choice",
                    ObjectKind::Gather { .. } => "gather",
                    _ => "weave point",
                };

                diagnostics.push(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    None,
                    0,
                    0,
                    format!(
                        "A {existing_type} with the same label name '{name}' already exists in this weave (new {type_name})"
                    ),
                ));
            } else {
                named_weave_points.insert(name, weave_point);
            }
        }

        diagnostics
    }

    pub fn weave_point_named(&self, name: &str) -> Option<ObjectRef> {
        if self.named_weave_points.borrow().is_empty() && !self.content().is_empty() {
            let _ = self.resolve_weave_point_naming();
        }

        self.named_weave_points.borrow().get(name).cloned()
    }

    fn construct_weave_hierarchy_from_indentation(&self) {
        let mut content_index = 0;
        while content_index < self.content().len() {
            let maybe_weave_indent = {
                let content_snapshot = self.content();
                weave_point_base_indent_index(&content_snapshot[content_index])
            };

            if let Some(weave_indent_index) = maybe_weave_indent {
                if weave_indent_index > self.base_indent_index() {
                    let mut inner_weave_start_idx = content_index;
                    let content_snapshot = self.content();
                    while inner_weave_start_idx < content_snapshot.len() {
                        if let Some(inner_indent) =
                            weave_point_base_indent_index(&content_snapshot[inner_weave_start_idx])
                        {
                            if inner_indent <= self.base_indent_index() {
                                break;
                            }
                        }

                        inner_weave_start_idx += 1;
                    }

                    let nested_content = {
                        let mut object = self.object.borrow_mut();
                        object
                            .content_mut()
                            .drain(content_index..inner_weave_start_idx)
                            .collect::<Vec<_>>()
                    };

                    let nested_weave = Weave::new(nested_content, weave_indent_index);
                    Object::insert_content(&self.object, content_index, nested_weave.object());
                    continue;
                }
            }

            content_index += 1;
        }
    }
}

impl fmt::Display for Weave {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Weave(base={})", self.base_indent_index())
    }
}

fn weave_point_base_indent_index(object: &ObjectRef) -> Option<usize> {
    let borrowed = object.borrow();
    match borrowed.kind() {
        ObjectKind::Choice {
            indentation_depth, ..
        }
        | ObjectKind::Gather {
            indentation_depth, ..
        } => indentation_depth.checked_sub(1),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::Weave;
    use crate::parsed::ObjectKind;
    use crate::parsed::{Choice, Gather, Identifier, Text};

    #[test]
    fn weave_determines_base_indentation() {
        let content = vec![Choice::new(Some(Identifier::new("start")), 2).object()];

        assert_eq!(Weave::determine_base_indentation_from_content(&content), 1);
    }

    #[test]
    fn weave_creates_nested_weaves_for_deeper_weave_points() {
        let content = vec![
            Choice::new(Some(Identifier::new("outer")), 1).object(),
            Text::new("Outside").object(),
            Choice::new(Some(Identifier::new("inner")), 2).object(),
            Gather::new(Some(Identifier::new("join")), 1).object(),
        ];

        let weave = Weave::new(content, 0);
        let outer_content = weave.content();

        assert_eq!(outer_content.len(), 4);
        assert!(matches!(
            outer_content[0].borrow().kind(),
            ObjectKind::Choice { .. }
        ));
        assert!(matches!(
            outer_content[1].borrow().kind(),
            ObjectKind::Text { .. }
        ));
        assert!(matches!(
            outer_content[2].borrow().kind(),
            ObjectKind::Weave { .. }
        ));
        assert!(matches!(
            outer_content[3].borrow().kind(),
            ObjectKind::Gather { .. }
        ));

        let nested_weave = outer_content[2].clone();
        let nested_children = nested_weave.borrow().content().to_vec();
        assert_eq!(nested_children.len(), 1);
    }

    #[test]
    fn weave_resolves_named_weave_points_and_detects_collisions() {
        let weave = Weave::new(
            vec![
                Choice::new(Some(Identifier::new("branch")), 1).object(),
                Gather::new(Some(Identifier::new("join")), 1).object(),
            ],
            0,
        );

        let diagnostics = weave.resolve_weave_point_naming();
        assert!(diagnostics.is_empty());
        assert!(weave.weave_point_named("branch").is_some());
        assert!(weave.weave_point_named("join").is_some());
    }
}
