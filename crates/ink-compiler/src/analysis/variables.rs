use crate::parsed::{
    visit::{walk_story, ParsedVisitor, VisitContext},
    Flow, Object, Story,
};

use super::{
    context::{
        EnumTypeIndex, StructTypeIndex, TargetSymbolIndex, VariableScopeIndex, VariableSymbolKind,
    },
    expression_types::infer_expression_type,
    for_loops::loop_variable_types,
    interfaces::InterfaceMemberIndex,
};

pub(super) fn build_variable_scope_index(story: &Story) -> VariableScopeIndex {
    let struct_types = StructTypeIndex::default();
    let enum_types = EnumTypeIndex::default();
    let target_symbols = TargetSymbolIndex::default();
    let interface_members = InterfaceMemberIndex::default();
    build_variable_scope_index_with_type_indexes(
        story,
        &struct_types,
        &enum_types,
        &target_symbols,
        &interface_members,
    )
}

pub(super) fn build_variable_scope_index_with_type_indexes(
    story: &Story,
    struct_types: &StructTypeIndex,
    enum_types: &EnumTypeIndex,
    target_symbols: &TargetSymbolIndex,
    interface_members: &InterfaceMemberIndex,
) -> VariableScopeIndex {
    struct VariableScopeVisitor<'a> {
        index: VariableScopeIndex,
        struct_types: &'a StructTypeIndex,
        enum_types: &'a EnumTypeIndex,
        target_symbols: &'a TargetSymbolIndex,
        interface_members: &'a InterfaceMemberIndex,
    }

    impl ParsedVisitor for VariableScopeVisitor<'_> {
        fn visit_flow(&mut self, flow: &Flow, context: &VisitContext) {
            let Some(flow_path) = &context.current_flow_path else {
                return;
            };
            for argument in flow.arguments() {
                self.index.insert_local(
                    context.current_module.as_deref(),
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
                        context.current_module.as_deref(),
                        declaration.name().to_string(),
                        Some(declaration.declared_type().clone()),
                        VariableSymbolKind::Constant,
                    );
                }
                Object::VariableAssignment(assignment) if assignment.is_global() => {
                    self.index.insert_global(
                        context.current_module.as_deref(),
                        assignment.name().to_string(),
                        assignment.declared_type().cloned(),
                        VariableSymbolKind::GlobalVariable,
                    );
                }
                Object::VariableAssignment(assignment)
                    if assignment.is_temporary() && context.current_flow_path.is_none() =>
                {
                    self.index.insert_global(
                        context.current_module.as_deref(),
                        assignment.name().to_string(),
                        assignment.declared_type().cloned(),
                        VariableSymbolKind::GlobalVariable,
                    );
                }
                Object::VariableAssignment(assignment) if assignment.is_temporary() => {
                    if let Some(flow_path) = &context.current_flow_path {
                        self.index.insert_local(
                            context.current_module.as_deref(),
                            flow_path.clone(),
                            assignment.name().to_string(),
                            assignment.declared_type().cloned(),
                        );
                    }
                }
                Object::ForLoop(for_loop) => {
                    self.index_loop_variables(for_loop, context);
                }
                _ => {}
            }
        }
    }

    impl VariableScopeVisitor<'_> {
        fn index_loop_variables(
            &mut self,
            for_loop: &crate::parsed::ForLoop,
            context: &VisitContext,
        ) {
            let Ok(iterable_type) = infer_expression_type(
                for_loop.iterable(),
                &self.index,
                self.struct_types,
                self.enum_types,
                self.target_symbols,
                self.interface_members,
                context.current_module.as_deref(),
                context.current_flow_path.as_deref(),
            ) else {
                return;
            };
            let Some(variable_types) =
                loop_variable_types(&iterable_type, for_loop.variables().len())
            else {
                return;
            };

            if let Some(flow_path) = &context.current_flow_path {
                self.index.insert_local(
                    context.current_module.as_deref(),
                    flow_path.clone(),
                    for_loop.index_name(),
                    Some(crate::parsed::TypeName::int()),
                );
                self.index.insert_local(
                    context.current_module.as_deref(),
                    flow_path.clone(),
                    for_loop.limit_name(),
                    Some(crate::parsed::TypeName::int()),
                );
                for (variable, variable_type) in for_loop.variables().iter().zip(variable_types) {
                    self.index.insert_local(
                        context.current_module.as_deref(),
                        flow_path.clone(),
                        variable.runtime_name().to_string(),
                        Some(variable_type),
                    );
                }
            } else {
                self.index.insert_global(
                    context.current_module.as_deref(),
                    for_loop.index_name(),
                    Some(crate::parsed::TypeName::int()),
                    VariableSymbolKind::GlobalVariable,
                );
                self.index.insert_global(
                    context.current_module.as_deref(),
                    for_loop.limit_name(),
                    Some(crate::parsed::TypeName::int()),
                    VariableSymbolKind::GlobalVariable,
                );
                for (variable, variable_type) in for_loop.variables().iter().zip(variable_types) {
                    self.index.insert_global(
                        context.current_module.as_deref(),
                        variable.runtime_name().to_string(),
                        Some(variable_type),
                        VariableSymbolKind::GlobalVariable,
                    );
                }
            }
        }
    }

    let mut visitor = VariableScopeVisitor {
        index: VariableScopeIndex::default(),
        struct_types,
        enum_types,
        target_symbols,
        interface_members,
    };
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

        assert!(index.contains_visible_variable("score", None, None));
        assert!(index.contains_visible_variable("score", None, Some("knot")));
        assert!(index.contains_visible_variable("arg", None, Some("knot")));
        assert!(index.contains_visible_variable("local", None, Some("knot")));
        assert!(!index.contains_visible_variable("arg", None, None));
        assert!(!index.contains_visible_variable("local", None, Some("other")));
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
            index.visible_variable_declared_type("score", None, None),
            Some(Some(&TypeName::int()))
        );
        assert_eq!(
            index.visible_variable_declared_type("score", None, Some("knot")),
            Some(Some(&TypeName::int()))
        );
        assert_eq!(
            index.visible_variable_declared_type("arg", None, Some("knot")),
            Some(Some(&TypeName::string()))
        );
        assert_eq!(
            index.visible_variable_declared_type("local", None, Some("knot")),
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
            index.visible_variable_declared_type("derived", None, None),
            Some(Some(&TypeName::int()))
        );
        assert_eq!(
            index.visible_variable_declared_type("arg", None, Some("knot")),
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
            index.visible_variable_declared_type("value", None, None),
            Some(Some(&TypeName::int()))
        );
        assert_eq!(
            index.visible_variable_declared_type("value", None, Some("knot")),
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
            index.visible_variable_declared_type("arg", None, Some("one")),
            Some(Some(&TypeName::int()))
        );
        assert_eq!(
            index.visible_variable_declared_type("local", None, Some("one")),
            Some(Some(&TypeName::bool()))
        );
        assert_eq!(
            index.visible_variable_declared_type("arg", None, Some("two")),
            Some(Some(&TypeName::string()))
        );
        assert_eq!(
            index.visible_variable_declared_type("local", None, Some("two")),
            None
        );
    }

    #[test]
    fn module_globals_are_visible_only_inside_their_module() {
        let story = parse_story(
            "=== module game ===\n\
             VAR value: int = 0\n\
             == main(arg: string) ==\n\
             ~ temp local: bool = true\n\
             -> DONE\n\
             === module items ===\n\
             VAR value: string = \"item\"\n\
             == helper ==\n\
             -> DONE",
        );

        let index = build_variable_scope_index(&story);

        assert_eq!(
            index.visible_variable_declared_type("value", Some("game"), Some("main")),
            Some(Some(&TypeName::int()))
        );
        assert_eq!(
            index.visible_variable_declared_type("value", Some("items"), Some("helper")),
            Some(Some(&TypeName::string()))
        );
        assert_eq!(
            index.visible_variable_declared_type("arg", Some("game"), Some("main")),
            Some(Some(&TypeName::string()))
        );
        assert_eq!(
            index.visible_variable_declared_type("local", Some("game"), Some("main")),
            Some(Some(&TypeName::bool()))
        );
        assert_eq!(
            index.visible_variable_declared_type("arg", Some("items"), Some("helper")),
            None
        );
    }

    #[test]
    fn qualified_global_lookups_distinguish_constants_from_variables() {
        let story = parse_story(
            "=== module items ===\n\
             CONST MAX_SCORE: int = 3\n\
             VAR score: int = 0\n\
             == main ==\n\
             -> DONE",
        );

        let index = build_variable_scope_index(&story);

        assert_eq!(
            index.qualified_constant_declared_type("items::MAX_SCORE"),
            Some(Some(&TypeName::int()))
        );
        assert_eq!(index.qualified_constant_declared_type("items::score"), None);
        assert_eq!(
            index.qualified_global_variable_declared_type("items::score"),
            Some(Some(&TypeName::int()))
        );
        assert_eq!(
            index.qualified_global_variable_declared_type("items::MAX_SCORE"),
            None
        );
        assert_eq!(
            index.qualified_constant_declared_type("missing::MAX_SCORE"),
            None
        );
        assert_eq!(
            index.qualified_global_variable_declared_type("missing::score"),
            None
        );
    }
}
