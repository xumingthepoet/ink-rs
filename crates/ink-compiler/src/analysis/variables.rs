use std::collections::{HashMap, HashSet};

use crate::parsed::{
    visit::{walk_story, ParsedVisitor, VisitContext},
    Flow, Object, Story,
};

#[derive(Debug, Default)]
pub(super) struct VariableScopeIndex {
    globals: HashSet<String>,
    locals_by_flow_path: HashMap<String, HashSet<String>>,
}

impl VariableScopeIndex {
    pub(super) fn contains_visible_variable(
        &self,
        name: &str,
        current_flow_path: Option<&str>,
    ) -> bool {
        if let Some(flow_path) = current_flow_path {
            if self
                .locals_by_flow_path
                .get(flow_path)
                .is_some_and(|locals| locals.contains(name))
            {
                return true;
            }
        }

        self.globals.contains(name)
    }
}

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
