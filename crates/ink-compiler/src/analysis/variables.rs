use crate::parsed::{
    visit::{walk_story, ParsedVisitor, VisitContext},
    Flow, Object, Story,
};

use super::context::VariableScopeIndex;

pub(super) fn build_variable_scope_index(story: &Story) -> VariableScopeIndex {
    #[derive(Default)]
    struct VariableScopeVisitor {
        index: VariableScopeIndex,
    }

    impl ParsedVisitor for VariableScopeVisitor {
        fn visit_flow(&mut self, flow: &Flow, context: &VisitContext) {
            let Some(flow_path) = &context.current_flow_path else {
                return;
            };
            for argument in flow.arguments() {
                self.index.insert_local(
                    flow_path.clone(),
                    argument.name().to_string(),
                    argument.declared_type().cloned(),
                );
            }
        }

        fn visit_object(&mut self, object: &Object, context: &VisitContext) {
            match object {
                Object::ConstantDeclaration(declaration) => {
                    self.index.insert_global(
                        declaration.name().to_string(),
                        Some(declaration.declared_type().clone()),
                    );
                }
                Object::VariableAssignment(assignment) if assignment.is_global() => {
                    self.index.insert_global(
                        assignment.name().to_string(),
                        assignment.declared_type().cloned(),
                    );
                }
                Object::VariableAssignment(assignment)
                    if assignment.is_temporary() && context.current_flow_path.is_none() =>
                {
                    self.index.insert_global(
                        assignment.name().to_string(),
                        assignment.declared_type().cloned(),
                    );
                }
                Object::VariableAssignment(assignment) if assignment.is_temporary() => {
                    if let Some(flow_path) = &context.current_flow_path {
                        self.index.insert_local(
                            flow_path.clone(),
                            assignment.name().to_string(),
                            assignment.declared_type().cloned(),
                        );
                    }
                }
                _ => {}
            }
        }
    }

    let mut visitor = VariableScopeVisitor::default();
    walk_story(story, &mut visitor);
    visitor.index
}

#[cfg(test)]
mod tests {
    use crate::parsed::TypeName;

    use super::{super::test_support::parse_story, *};

    #[test]
    fn indexes_globals_and_flow_local_variables() {
        let story = parse_story(
            "VAR score: int = 0\n\
             == knot(arg) ==\n\
             ~ temp local: int = 0\n\
             -> DONE\n\
             == other ==\n\
             -> DONE",
        );

        let index = build_variable_scope_index(&story);

        assert!(index.contains_visible_variable("score", None));
        assert!(index.contains_visible_variable("score", Some("knot")));
        assert!(index.contains_visible_variable("arg", Some("knot")));
        assert!(index.contains_visible_variable("local", Some("knot")));
        assert!(!index.contains_visible_variable("arg", None));
        assert!(!index.contains_visible_variable("local", Some("other")));
    }

    #[test]
    fn indexes_declared_types_for_globals_temps_and_arguments() {
        let story = parse_story(
            "VAR score: int = 0\n\
             == knot(arg: string) ==\n\
             ~ temp local: bool = true\n\
             -> DONE",
        );

        let index = build_variable_scope_index(&story);

        assert_eq!(
            index.visible_variable_declared_type("score", None),
            Some(Some(&TypeName::int()))
        );
        assert_eq!(
            index.visible_variable_declared_type("score", Some("knot")),
            Some(Some(&TypeName::int()))
        );
        assert_eq!(
            index.visible_variable_declared_type("arg", Some("knot")),
            Some(Some(&TypeName::string()))
        );
        assert_eq!(
            index.visible_variable_declared_type("local", Some("knot")),
            Some(Some(&TypeName::bool()))
        );
    }

    #[test]
    fn indexes_constant_declared_types() {
        let story = parse_story(
            "CONST derived: int = other\n\
             == knot(arg) ==\n\
             -> DONE",
        );

        let index = build_variable_scope_index(&story);

        assert_eq!(
            index.visible_variable_declared_type("derived", None),
            Some(Some(&TypeName::int()))
        );
        assert_eq!(
            index.visible_variable_declared_type("arg", Some("knot")),
            Some(None)
        );
    }

    #[test]
    fn local_declared_types_shadow_global_declared_types() {
        let story = parse_story(
            "VAR value: int = 0\n\
             == knot(value: string) ==\n\
             -> DONE",
        );

        let index = build_variable_scope_index(&story);

        assert_eq!(
            index.visible_variable_declared_type("value", None),
            Some(Some(&TypeName::int()))
        );
        assert_eq!(
            index.visible_variable_declared_type("value", Some("knot")),
            Some(Some(&TypeName::string()))
        );
    }

    #[test]
    fn typed_locals_stay_within_flow_boundaries() {
        let story = parse_story(
            "== one(arg: int) ==\n\
             ~ temp local: bool = true\n\
             -> DONE\n\
             == two(arg: string) ==\n\
             -> DONE",
        );

        let index = build_variable_scope_index(&story);

        assert_eq!(
            index.visible_variable_declared_type("arg", Some("one")),
            Some(Some(&TypeName::int()))
        );
        assert_eq!(
            index.visible_variable_declared_type("local", Some("one")),
            Some(Some(&TypeName::bool()))
        );
        assert_eq!(
            index.visible_variable_declared_type("arg", Some("two")),
            Some(Some(&TypeName::string()))
        );
        assert_eq!(
            index.visible_variable_declared_type("local", Some("two")),
            None
        );
    }
}
