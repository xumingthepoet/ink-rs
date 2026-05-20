use std::collections::{BTreeMap, BTreeSet};

use crate::parsed::{
    visit::{walk_story, ParsedVisitor, VisitContext},
    AssignmentTarget, ConstantDeclaration, DivertTarget, Expression, InterfaceMemberKind, Object,
    Story, TypeName, VariableAssignment,
};

use super::{
    context::{EnumTypeIndex, StructTypeIndex, TargetSymbolIndex, VariableScopeIndex},
    enums::build_enum_type_index,
    expression_types::{infer_expression_type, TypeInferenceError},
    interfaces::{build_interface_member_index, InterfaceMemberIndex},
    modules::ModuleImportIndex,
    structs::{build_struct_type_index, resolve_struct_symbol},
    target_symbols::build_target_symbol_index,
    type_names::{qualify_type_name_for_module, type_name_module},
    variables::build_variable_scope_index,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ModuleImplementationIndex {
    interfaces_by_module: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct InterfaceModuleLiteralUses {
    candidate_expression_ids: BTreeSet<usize>,
    modules_by_importing_module: BTreeMap<String, BTreeSet<String>>,
}

pub(super) fn build_module_implementation_index(story: &Story) -> ModuleImplementationIndex {
    let mut index = ModuleImplementationIndex::default();

    for module in story.modules() {
        index
            .interfaces_by_module
            .entry(module.name().to_string())
            .or_default()
            .extend(
                module
                    .implemented_interfaces()
                    .iter()
                    .map(|interface| interface.name().to_string()),
            );
    }

    index
}

pub(super) fn infer_expected_interface_expression_type(
    expression: &Expression,
    expected_type: &TypeName,
    variable_scopes: &VariableScopeIndex,
    struct_types: &StructTypeIndex,
    enum_types: &EnumTypeIndex,
    target_symbols: &TargetSymbolIndex,
    module_implementations: &ModuleImplementationIndex,
    module_imports: &ModuleImportIndex,
    interface_members: &InterfaceMemberIndex,
    current_module: Option<&str>,
    current_flow_path: Option<&str>,
) -> Option<Result<TypeName, TypeInferenceError>> {
    let interface_name = expected_type.as_interface_name()?;

    match infer_expression_type(
        expression,
        variable_scopes,
        struct_types,
        enum_types,
        target_symbols,
        interface_members,
        current_module,
        current_flow_path,
    ) {
        Ok(actual_type) => Some(Ok(actual_type)),
        Err(error) => {
            let Expression::VariableReference(module_name) = expression else {
                return Some(Err(error));
            };

            if variable_scopes.contains_visible_variable(
                module_name,
                current_module,
                current_flow_path,
            ) {
                return Some(Err(error));
            }

            Some(infer_module_literal_type(
                module_name,
                interface_name,
                expected_type,
                module_implementations,
                module_imports,
                current_module,
            ))
        }
    }
}

pub(super) fn collect_interface_module_literal_uses_for_story(
    story: &Story,
) -> InterfaceModuleLiteralUses {
    let struct_types = build_struct_type_index(story);
    let enum_types = build_enum_type_index(story);
    let variable_scopes = build_variable_scope_index(story);
    let target_symbols = build_target_symbol_index(story);
    let module_implementations = build_module_implementation_index(story);
    let interface_members = build_interface_member_index(story);

    collect_interface_module_literal_uses(
        story,
        &variable_scopes,
        &struct_types,
        &enum_types,
        &target_symbols,
        &module_implementations,
        &interface_members,
    )
}

fn collect_interface_module_literal_uses(
    story: &Story,
    variable_scopes: &VariableScopeIndex,
    struct_types: &StructTypeIndex,
    enum_types: &EnumTypeIndex,
    target_symbols: &TargetSymbolIndex,
    module_implementations: &ModuleImplementationIndex,
    interface_members: &InterfaceMemberIndex,
) -> InterfaceModuleLiteralUses {
    let mut collector = InterfaceModuleLiteralUseCollector::new(
        variable_scopes,
        struct_types,
        enum_types,
        target_symbols,
        module_implementations,
        interface_members,
    );
    walk_story(story, &mut collector);
    collector.uses
}

fn infer_module_literal_type(
    module_name: &str,
    interface_name: &str,
    expected_type: &TypeName,
    module_implementations: &ModuleImplementationIndex,
    module_imports: &ModuleImportIndex,
    current_module: Option<&str>,
) -> Result<TypeName, TypeInferenceError> {
    if !module_implementations.has_module(module_name) {
        return Err(TypeInferenceError::new(format!(
            "Unknown module '{module_name}' for interface value"
        )));
    }

    let Some(current_module) = current_module else {
        return Err(TypeInferenceError::new(format!(
            "Module literal '{module_name}' cannot be used outside a module"
        )));
    };

    if current_module != module_name && !module_imports.imports_module(current_module, module_name)
    {
        return Err(TypeInferenceError::new(format!(
            "Module literal '{module_name}' requires a bare import in module '{current_module}': FROM {module_name}"
        )));
    }

    if !module_implementations.module_implements(module_name, interface_name) {
        return Err(TypeInferenceError::new(format!(
            "Module '{module_name}' does not implement interface '{interface_name}'"
        )));
    }

    Ok(expected_type.clone())
}

impl ModuleImplementationIndex {
    pub(super) fn has_module(&self, module: &str) -> bool {
        self.interfaces_by_module.contains_key(module)
    }

    pub(super) fn module_implements(&self, module: &str, interface: &str) -> bool {
        self.interfaces_by_module
            .get(module)
            .is_some_and(|interfaces| interfaces.contains(interface))
    }
}

impl InterfaceModuleLiteralUses {
    pub(super) fn contains_expression(&self, expression: &Expression) -> bool {
        self.candidate_expression_ids
            .contains(&(expression as *const Expression as usize))
    }

    pub(super) fn uses_module(&self, importing_module: &str, source_module: &str) -> bool {
        self.modules_by_importing_module
            .get(importing_module)
            .is_some_and(|modules| modules.contains(source_module))
    }
}

struct InterfaceModuleLiteralUseCollector<'a> {
    variable_scopes: &'a VariableScopeIndex,
    struct_types: &'a StructTypeIndex,
    enum_types: &'a EnumTypeIndex,
    target_symbols: &'a TargetSymbolIndex,
    module_implementations: &'a ModuleImplementationIndex,
    interface_members: &'a InterfaceMemberIndex,
    uses: InterfaceModuleLiteralUses,
}

impl<'a> InterfaceModuleLiteralUseCollector<'a> {
    fn new(
        variable_scopes: &'a VariableScopeIndex,
        struct_types: &'a StructTypeIndex,
        enum_types: &'a EnumTypeIndex,
        target_symbols: &'a TargetSymbolIndex,
        module_implementations: &'a ModuleImplementationIndex,
        interface_members: &'a InterfaceMemberIndex,
    ) -> Self {
        Self {
            variable_scopes,
            struct_types,
            enum_types,
            target_symbols,
            module_implementations,
            interface_members,
            uses: InterfaceModuleLiteralUses::default(),
        }
    }

    fn check_constant(&mut self, declaration: &ConstantDeclaration, context: &VisitContext) {
        self.check_expression_against_type(
            declaration.expression(),
            declaration.declared_type(),
            context,
        );
    }

    fn check_assignment(&mut self, assignment: &VariableAssignment, context: &VisitContext) {
        let Some(expression) = assignment.expression() else {
            return;
        };

        let expected_type = if assignment.is_global() || assignment.is_temporary() {
            assignment.declared_type().cloned()
        } else {
            self.resolve_assignment_target_type(assignment.target(), context)
        };

        let Some(expected_type) = expected_type else {
            return;
        };

        self.check_expression_against_type(expression, &expected_type, context);
    }

    fn check_expression_against_type(
        &mut self,
        expression: &Expression,
        expected_type: &TypeName,
        context: &VisitContext,
    ) {
        match (expected_type, expression) {
            (TypeName::Interface { .. }, _) => {
                self.record_module_literal_candidate(expression, context);
            }
            (TypeName::Array(element_type), Expression::ArrayLiteral(elements)) => {
                for element in elements {
                    self.check_expression_against_type(element, element_type, context);
                }
            }
            (TypeName::Dict { value_type, .. }, Expression::DictLiteral(entries)) => {
                for entry in entries {
                    self.check_expression_against_type(entry.value(), value_type, context);
                }
            }
            (TypeName::Struct(struct_name), Expression::StructLiteral { fields, .. }) => {
                self.check_struct_literal(struct_name, fields, context);
            }
            (TypeName::QualifiedStruct(struct_name), Expression::StructLiteral { fields, .. }) => {
                self.check_struct_literal(struct_name.as_str(), fields, context);
            }
            _ => {}
        }
    }

    fn check_struct_literal(
        &mut self,
        struct_name: &str,
        fields: &[crate::parsed::StructLiteralField],
        context: &VisitContext,
    ) {
        let Some(symbol) = resolve_struct_symbol(
            self.struct_types,
            struct_name,
            context.current_module.as_deref(),
        ) else {
            return;
        };

        for field in fields {
            let Some(field_type) = symbol.fields().get(field.name()) else {
                continue;
            };
            self.check_expression_against_type(field.expression(), field_type, context);
        }
    }

    fn record_module_literal_candidate(&mut self, expression: &Expression, context: &VisitContext) {
        let Expression::VariableReference(module_name) = expression else {
            return;
        };

        let Some(current_module) = context.current_module.as_deref() else {
            return;
        };

        if self.variable_scopes.contains_visible_variable(
            module_name,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) {
            return;
        }

        self.uses
            .candidate_expression_ids
            .insert(expression as *const Expression as usize);

        if !self.module_implementations.has_module(module_name) {
            return;
        }

        self.uses
            .modules_by_importing_module
            .entry(current_module.to_string())
            .or_default()
            .insert(module_name.to_string());
    }

    fn check_dynamic_interface_member_arguments(
        &mut self,
        target: &Expression,
        member: &str,
        expected_kind: &InterfaceMemberKind,
        arguments: &[Expression],
        context: &VisitContext,
    ) {
        let Ok(target_type) = infer_expression_type(
            target,
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            self.interface_members,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) else {
            return;
        };

        let Some(interface_name) = target_type.as_interface_name() else {
            return;
        };
        let Some(signature) = self.interface_members.member(interface_name, member) else {
            return;
        };
        if signature.kind() != expected_kind {
            return;
        }

        for (argument, parameter) in arguments.iter().zip(signature.arguments()) {
            let Some(expected_type) = parameter.declared_type() else {
                continue;
            };
            self.check_expression_against_type(argument, expected_type, context);
        }
    }

    fn resolve_assignment_target_type(
        &self,
        target: &AssignmentTarget,
        context: &VisitContext,
    ) -> Option<TypeName> {
        match target {
            AssignmentTarget::Variable(name) => self
                .variable_scopes
                .visible_variable_declared_type(
                    name,
                    context.current_module.as_deref(),
                    context.current_flow_path.as_deref(),
                )
                .and_then(|declared_type| declared_type.cloned()),
            AssignmentTarget::QualifiedVariable(name) => {
                self.qualified_assignment_declared_type(name.as_str())
            }
            AssignmentTarget::FieldAccess { base, field } => {
                let base_type = self.resolve_assignment_target_type(base, context)?;
                let struct_name = base_type.as_struct_name()?;
                let symbol = resolve_struct_symbol(
                    self.struct_types,
                    struct_name,
                    context.current_module.as_deref(),
                )?;
                let field_type = symbol.fields().get(field).cloned()?;
                Some(
                    type_name_module(&base_type)
                        .map(|module| qualify_type_name_for_module(&field_type, module))
                        .unwrap_or(field_type),
                )
            }
            AssignmentTarget::IndexAccess { base, index } => {
                let base_type = self.resolve_assignment_target_type(base, context)?;
                let Ok(index_type) = infer_expression_type(
                    index,
                    self.variable_scopes,
                    self.struct_types,
                    self.enum_types,
                    self.target_symbols,
                    self.interface_members,
                    context.current_module.as_deref(),
                    context.current_flow_path.as_deref(),
                ) else {
                    return None;
                };
                if index_type != TypeName::int() {
                    return None;
                }
                base_type.array_element_type().cloned()
            }
        }
    }

    fn qualified_assignment_declared_type(&self, name: &str) -> Option<TypeName> {
        let module = name.split_once("::").map(|(module, _)| module);
        self.variable_scopes
            .qualified_global_variable_declared_type(name)
            .and_then(|declared_type| {
                declared_type.map(|type_name| {
                    module
                        .map(|module| qualify_type_name_for_module(type_name, module))
                        .unwrap_or_else(|| type_name.clone())
                })
            })
    }
}

impl ParsedVisitor for InterfaceModuleLiteralUseCollector<'_> {
    fn visit_object(&mut self, object: &Object, context: &VisitContext) {
        match object {
            Object::ConstantDeclaration(declaration) => self.check_constant(declaration, context),
            Object::VariableAssignment(assignment) => self.check_assignment(assignment, context),
            Object::Divert(divert) => {
                if let DivertTarget::Dynamic(Expression::DynamicInterfaceAccess {
                    target,
                    member,
                }) = divert.target()
                {
                    self.check_dynamic_interface_member_arguments(
                        target,
                        member,
                        &InterfaceMemberKind::Knot,
                        divert.arguments(),
                        context,
                    );
                }
            }
            Object::TunnelOnwards(tunnel_onwards) => {
                if let Some(DivertTarget::Dynamic(Expression::DynamicInterfaceAccess {
                    target,
                    member,
                })) = tunnel_onwards.override_target()
                {
                    self.check_dynamic_interface_member_arguments(
                        target,
                        member,
                        &InterfaceMemberKind::Knot,
                        tunnel_onwards.arguments(),
                        context,
                    );
                }
            }
            _ => {}
        }
    }

    fn visit_expression(&mut self, expression: &Expression, context: &VisitContext) {
        if let Expression::DynamicInterfaceFunctionCall {
            target,
            member,
            args,
        } = expression
        {
            self.check_dynamic_interface_member_arguments(
                target,
                member,
                &InterfaceMemberKind::Function,
                args,
                context,
            );
        }
    }
}
