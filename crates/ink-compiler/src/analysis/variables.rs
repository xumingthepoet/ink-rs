use std::collections::HashSet;

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
            let locals = flow
                .arguments()
                .iter()
                .map(|argument| argument.name().to_string())
                .collect::<HashSet<_>>();
            self.index
                .locals_by_flow_path
                .entry(flow_path.clone())
                .or_insert(locals);
        }

        fn visit_object(&mut self, object: &Object, context: &VisitContext) {
            match object {
                Object::ConstantDeclaration(declaration) => {
                    self.index.globals.insert(declaration.name().to_string());
                }
                Object::VariableAssignment(assignment) if assignment.is_global() => {
                    self.index.globals.insert(assignment.name().to_string());
                }
                Object::VariableAssignment(assignment)
                    if assignment.is_temporary() && context.current_flow_path.is_none() =>
                {
                    self.index.globals.insert(assignment.name().to_string());
                }
                Object::VariableAssignment(assignment) if assignment.is_temporary() => {
                    if let Some(flow_path) = &context.current_flow_path {
                        self.index
                            .locals_by_flow_path
                            .entry(flow_path.clone())
                            .or_default()
                            .insert(assignment.name().to_string());
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
    use super::{super::test_support::parse_story, *};

    #[test]
    fn indexes_globals_and_flow_local_variables() {
        let story = parse_story(
            "VAR score = 0\n\
             == knot(arg) ==\n\
             ~ temp local = arg\n\
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
}
