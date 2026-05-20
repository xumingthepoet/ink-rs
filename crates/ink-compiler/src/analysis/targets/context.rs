use crate::parsed::{visit::VisitContext, Expression, FlowArgument, TypeName};

use super::super::{
    argument_resolution::DynamicInterfaceSignatureInputs, context::FlowContext,
    interface_values::ExpectedTypeInference,
};
use super::checker::CallTargetChecker;

impl<'a> CallTargetChecker<'a> {
    pub(super) fn current_flow_path<'context>(
        &self,
        context: &'context VisitContext,
    ) -> Option<&'context str> {
        context.current_flow_path.as_deref()
    }

    pub(super) fn current_module<'context>(
        &self,
        context: &'context VisitContext,
    ) -> Option<&'context str> {
        context.current_module.as_deref()
    }

    pub(super) fn expected_type_inference(&self) -> ExpectedTypeInference<'_> {
        ExpectedTypeInference {
            variable_scopes: self.variable_scopes,
            struct_types: self.struct_types,
            enum_types: self.enum_types,
            target_symbols: self.target_symbols,
            module_implementations: self.module_implementations,
            module_imports: self.module_imports,
            interface_members: self.interface_members,
        }
    }

    pub(super) fn current_flow_context(&self, context: &VisitContext) -> Option<&FlowContext> {
        self.current_flow_path(context).and_then(|flow_path| {
            self.flow_contexts_by_path
                .get(&scoped_context_key(self.current_module(context), flow_path))
        })
    }

    pub(super) fn dynamic_interface_signature_inputs(&self) -> DynamicInterfaceSignatureInputs<'_> {
        DynamicInterfaceSignatureInputs {
            variable_scopes: self.variable_scopes,
            struct_types: self.struct_types,
            enum_types: self.enum_types,
            target_symbols: self.target_symbols,
            interface_members: self.interface_members,
        }
    }

    pub(super) fn current_flow_arguments(&self, context: &VisitContext) -> Option<&[FlowArgument]> {
        self.current_flow_context(context)
            .map(FlowContext::arguments)
    }

    pub(super) fn current_flow_is_function(&self, context: &VisitContext) -> bool {
        self.current_flow_context(context)
            .is_some_and(FlowContext::is_function)
    }
}

pub(super) fn is_mutable_lvalue(expression: &Expression) -> bool {
    match expression {
        Expression::VariableReference(_) => true,
        Expression::QualifiedReference(_) => true,
        Expression::FieldAccess { base, .. } | Expression::IndexAccess { base, .. } => {
            is_mutable_lvalue(base)
        }
        Expression::DynamicInterfaceAccess { .. }
        | Expression::DynamicInterfaceFunctionCall { .. } => false,
        _ => false,
    }
}

pub(super) fn is_composite_literal(expression: &Expression) -> bool {
    matches!(
        expression,
        Expression::ArrayLiteral(_) | Expression::StructLiteral { .. } | Expression::DictLiteral(_)
    )
}

pub(super) fn is_runtime_builtin_function(name: &str) -> bool {
    matches!(
        name,
        "RANDOM" | "SEED_RANDOM" | "MIN" | "MAX" | "POW" | "FLOOR" | "CEILING" | "INT" | "FLOAT"
    )
}

pub(super) fn is_interface_module_literal_argument(
    expression: &Expression,
    expected_type: &TypeName,
) -> bool {
    expected_type.as_interface_name().is_some()
        && matches!(expression, Expression::VariableReference(_))
}

pub(super) fn expression_root_variable_name(expression: &Expression) -> Option<&str> {
    match expression {
        Expression::VariableReference(name) => Some(name),
        Expression::QualifiedReference(name) => Some(name.as_str()),
        Expression::FieldAccess { base, .. } | Expression::IndexAccess { base, .. } => {
            expression_root_variable_name(base)
        }
        Expression::DynamicInterfaceAccess { .. }
        | Expression::DynamicInterfaceFunctionCall { .. } => None,
        _ => None,
    }
}

pub(super) fn resolve_current_flow_argument<'a>(
    target: &str,
    current_flow_arguments: Option<&'a [FlowArgument]>,
) -> Option<&'a FlowArgument> {
    let variable_target_name = target.split('.').next()?;
    current_flow_arguments?
        .iter()
        .find(|argument| argument.name() == variable_target_name)
}

pub(super) fn scoped_context_key(module: Option<&str>, flow_path: &str) -> String {
    module
        .map(|module| format!("{module}::{flow_path}"))
        .unwrap_or_else(|| flow_path.to_string())
}
