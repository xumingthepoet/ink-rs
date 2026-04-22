use std::{fmt, rc::Rc};

use super::{find_all, FlowLevel, Identifier, Object, ObjectKind, ObjectRef};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    base_target_level: Option<FlowLevel>,
    pub components: Vec<Identifier>,
}

impl Path {
    pub fn new(components: Vec<Identifier>) -> Self {
        Self {
            base_target_level: None,
            components,
        }
    }

    pub fn with_base_target_level(
        base_target_level: Option<FlowLevel>,
        components: Vec<Identifier>,
    ) -> Self {
        Self {
            base_target_level,
            components,
        }
    }

    pub fn from_identifier(identifier: Identifier) -> Self {
        Self::new(vec![identifier])
    }

    pub fn base_target_level(&self) -> FlowLevel {
        self.base_target_level.unwrap_or(FlowLevel::Story)
    }

    pub fn base_level_is_ambiguous(&self) -> bool {
        self.base_target_level.is_none()
    }

    pub fn components(&self) -> &[Identifier] {
        &self.components
    }

    pub fn from_components(
        base_target_level: Option<FlowLevel>,
        components: Vec<Identifier>,
    ) -> Self {
        Self {
            base_target_level,
            components,
        }
    }

    pub fn first_component(&self) -> Option<&str> {
        self.components
            .first()
            .map(|component| component.name.as_str())
    }

    pub fn number_of_components(&self) -> usize {
        self.components.len()
    }

    pub fn dot_separated_components(&self) -> Option<String> {
        if self.components.is_empty() {
            None
        } else {
            Some(
                self.components
                    .iter()
                    .map(|component| component.name.as_str())
                    .collect::<Vec<_>>()
                    .join("."),
            )
        }
    }

    pub fn resolve_from_context(&self, context: &ObjectRef) -> Option<ObjectRef> {
        if self.components.is_empty() {
            return None;
        }

        let base_target = self.resolve_base_target(context)?;
        if self.components.len() == 1 {
            return Some(base_target);
        }

        self.resolve_tail_components(&base_target)
    }

    fn resolve_base_target(&self, original_context: &ObjectRef) -> Option<ObjectRef> {
        let first_component = self.first_component()?;
        let mut ancestor = Some(original_context.clone());

        while let Some(context) = ancestor {
            let deep_search = Rc::ptr_eq(&context, original_context);
            if let Some(found) =
                try_get_child_from_context(&context, first_component, None, deep_search)
            {
                return Some(found);
            }

            ancestor = context.borrow().parent();
        }

        None
    }

    fn resolve_tail_components(&self, root_target: &ObjectRef) -> Option<ObjectRef> {
        let mut found_component = root_target.clone();

        for component in self.components.iter().skip(1) {
            let minimum_expected_level = match found_component.borrow().kind() {
                ObjectKind::Flow { flow_level, .. } => flow_level.next(),
                _ => Some(FlowLevel::WeavePoint),
            };

            found_component = try_get_child_from_context(
                &found_component,
                &component.name,
                minimum_expected_level,
                false,
            )?;
        }

        Some(found_component)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.dot_separated_components() {
            Some(components) => write!(f, "-> {components}"),
            None if self.base_target_level() == FlowLevel::WeavePoint => {
                f.write_str("-> <next gather point>")
            }
            None => f.write_str("<invalid Path>"),
        }
    }
}

fn try_get_child_from_context(
    context: &ObjectRef,
    child_name: &str,
    minimum_level: Option<FlowLevel>,
    force_deep_search: bool,
) -> Option<ObjectRef> {
    if let Some(found) = current_object_match(context, child_name, minimum_level) {
        return Some(found);
    }

    if let Some(found) = direct_named_child(context, child_name, minimum_level) {
        return Some(found);
    }

    if force_deep_search {
        for child in context.borrow().content().iter().cloned() {
            if let Some(found) = try_get_child_from_context(&child, child_name, minimum_level, true)
            {
                return Some(found);
            }
        }
    }

    None
}

fn current_object_match(
    context: &ObjectRef,
    child_name: &str,
    minimum_level: Option<FlowLevel>,
) -> Option<ObjectRef> {
    let borrowed = context.borrow();

    match borrowed.kind() {
        ObjectKind::Flow {
            flow_level, name, ..
        } => {
            if level_allows(*flow_level, minimum_level) && name.as_deref() == Some(child_name) {
                return Some(context.clone());
            }
        }
        ObjectKind::Choice { identifier, .. } | ObjectKind::Gather { identifier, .. } => {
            if level_allows(FlowLevel::WeavePoint, minimum_level)
                && identifier
                    .as_ref()
                    .map(|identifier| identifier.name.as_str())
                    == Some(child_name)
            {
                return Some(context.clone());
            }
        }
        _ => {}
    }

    None
}

fn direct_named_child(
    context: &ObjectRef,
    child_name: &str,
    minimum_level: Option<FlowLevel>,
) -> Option<ObjectRef> {
    let children = context.borrow().content().to_vec();

    for child in children {
        if let Some(found) = current_object_match(&child, child_name, minimum_level) {
            return Some(found);
        }

        let borrowed = child.borrow();
        match borrowed.kind() {
            ObjectKind::Weave { .. } => {
                if minimum_level.is_none() || minimum_level == Some(FlowLevel::WeavePoint) {
                    if let Some(found) = weave_point_named_in_subtree(&child, child_name) {
                        return Some(found);
                    }
                }
            }
            ObjectKind::Flow { flow_level, .. } => {
                if level_allows(*flow_level, minimum_level) {
                    if let Some(name) = borrowed_flow_name(&borrowed) {
                        if name == child_name {
                            return Some(child.clone());
                        }
                    }
                }
            }
            ObjectKind::Choice { identifier, .. } | ObjectKind::Gather { identifier, .. } => {
                if level_allows(FlowLevel::WeavePoint, minimum_level)
                    && identifier
                        .as_ref()
                        .map(|identifier| identifier.name.as_str())
                        == Some(child_name)
                {
                    return Some(child.clone());
                }
            }
            _ => {}
        }
    }

    None
}

fn borrowed_flow_name(object: &Object) -> Option<&str> {
    match object.kind() {
        ObjectKind::Flow { name, .. } => name.as_deref(),
        _ => None,
    }
}

fn weave_point_named_in_subtree(context: &ObjectRef, child_name: &str) -> Option<ObjectRef> {
    find_all(context, |object| match object.kind() {
        ObjectKind::Choice { identifier, .. } | ObjectKind::Gather { identifier, .. } => {
            identifier
                .as_ref()
                .map(|identifier| identifier.name.as_str())
                == Some(child_name)
        }
        _ => false,
    })
    .into_iter()
    .next()
}

fn level_allows(candidate: FlowLevel, minimum_level: Option<FlowLevel>) -> bool {
    match minimum_level {
        None => true,
        Some(minimum_level) => candidate == minimum_level,
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::{FlowLevel, Identifier, Path};
    use crate::parsed::{Choice, Gather, Knot, Object, Stitch, Story, Text, Weave};

    #[test]
    fn parsed_path_joins_components() {
        let path = Path::new(vec![Identifier::new("root"), Identifier::new("child")]);

        assert_eq!(path.first_component(), Some("root"));
        assert_eq!(path.number_of_components(), 2);
        assert_eq!(
            path.dot_separated_components().as_deref(),
            Some("root.child")
        );
        assert_eq!(path.to_string(), "-> root.child");
    }

    #[test]
    fn parsed_path_handles_empty_components() {
        let path = Path::new(Vec::new());

        assert_eq!(path.first_component(), None);
        assert_eq!(path.dot_separated_components(), None);
        assert_eq!(path.to_string(), "<invalid Path>");
    }

    #[test]
    fn parsed_path_formats_next_gather_point() {
        let path = Path::with_base_target_level(Some(FlowLevel::WeavePoint), Vec::new());

        assert_eq!(path.base_target_level(), FlowLevel::WeavePoint);
        assert!(!path.base_level_is_ambiguous());
        assert_eq!(path.to_string(), "-> <next gather point>");
    }

    #[test]
    fn parsed_path_resolves_named_flow_and_nested_stitch() {
        let stitch = Stitch::new(Identifier::new("inner"), vec![], Vec::new(), false).object();
        let knot = Knot::new(
            Identifier::new("start"),
            vec![stitch.clone()],
            Vec::new(),
            false,
        )
        .object();
        let story = Story::new(vec![knot.clone()], false);

        let knot_path = Path::from_identifier(Identifier::new("start"));
        let resolved_knot = knot_path.resolve_from_context(&story.root()).expect("knot");
        assert!(Rc::ptr_eq(&resolved_knot, &knot));

        let stitch_path = Path::new(vec![Identifier::new("start"), Identifier::new("inner")]);
        let resolved_stitch = stitch_path
            .resolve_from_context(&story.root())
            .expect("stitch");
        assert!(Rc::ptr_eq(&resolved_stitch, &stitch));
    }

    #[test]
    fn parsed_path_resolves_weave_points_within_flow() {
        let choice = Choice::new(Some(Identifier::new("branch")), 1);
        let weave = Weave::new(vec![choice.object()], 0).object();
        let knot = Knot::new(
            Identifier::new("start"),
            vec![weave.clone()],
            Vec::new(),
            false,
        )
        .object();
        let story = Story::new(vec![knot.clone()], false);

        let path = Path::from_identifier(Identifier::new("branch"));
        let resolved = path.resolve_from_context(&story.root()).expect("choice");

        assert!(Rc::ptr_eq(&resolved, &choice.object()));
    }

    #[test]
    fn parsed_path_resolves_nested_stitch_from_gather_context() {
        let gather = Gather::new(Some(Identifier::new("gatherpoint")), 1);
        Object::add_content(&gather.object(), Text::new("Some content.").object());
        let stitch_one = Stitch::new(
            Identifier::new("stitch_one"),
            vec![gather.object()],
            Vec::new(),
            false,
        )
        .object();
        let stitch_two =
            Stitch::new(Identifier::new("stitch_two"), vec![], Vec::new(), false).object();
        let knot = Knot::new(
            Identifier::new("knot"),
            vec![stitch_one.clone(), stitch_two.clone()],
            Vec::new(),
            false,
        )
        .object();
        let story = Story::new(vec![knot.clone()], false);

        let path = Path::new(vec![Identifier::new("knot"), Identifier::new("stitch_two")]);
        let resolved = path
            .resolve_from_context(&gather.object())
            .expect("stitch_two");

        assert!(Rc::ptr_eq(&resolved, &stitch_two));
        assert!(story.content().len() >= 1);
    }
}
