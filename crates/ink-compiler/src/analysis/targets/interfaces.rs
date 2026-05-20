use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::VisitContext, Expression, InterfaceMemberKind, InterfaceMemberSignature, TypeName,
    },
    source::SourceSpan,
};

use super::super::{
    argument_resolution::{
        resolve_dynamic_interface_signature, DynamicInterfaceSignatureError,
        DynamicInterfaceSignatureInputs,
    },
    interface_values::{check_expression_type_with_expected, ExpectedTypeCheckError},
};
use super::checker::{is_composite_literal, CallTargetChecker};

impl<'a> CallTargetChecker<'a> {
    fn dynamic_interface_signature_inputs(&self) -> DynamicInterfaceSignatureInputs<'_> {
        DynamicInterfaceSignatureInputs {
            variable_scopes: self.variable_scopes,
            struct_types: self.struct_types,
            enum_types: self.enum_types,
            target_symbols: self.target_symbols,
            interface_members: self.interface_members,
        }
    }

    pub(super) fn check_dynamic_interface_divert_target(
        &mut self,
        target: &Expression,
        member: &str,
        arguments: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        self.check_expression(target, span, context);

        match self.dynamic_interface_member_signature(target, member, span, context) {
            Some(signature) => {
                self.check_dynamic_interface_member_arguments(
                    "target", member, arguments, &signature, span, context,
                );
            }
            None => {
                for argument in arguments {
                    self.check_expression(argument, span, context);
                }
            }
        }
    }

    fn dynamic_interface_member_signature(
        &mut self,
        target: &Expression,
        member: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) -> Option<InterfaceMemberSignature> {
        match resolve_dynamic_interface_signature(
            target,
            member,
            InterfaceMemberKind::Knot,
            self.dynamic_interface_signature_inputs(),
            self.current_module(context),
            self.current_flow_path(context),
        ) {
            Ok(signature) => Some(signature),
            Err(DynamicInterfaceSignatureError::Inference(error)) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Cannot type-check dynamic interface target '{member}': {}",
                        error.message()
                    ),
                ));
                None
            }
            Err(DynamicInterfaceSignatureError::NonInterface(target_type)) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Dynamic interface target '{member}' has base type {} but expected interface",
                        target_type.display_name()
                    ),
                ));
                None
            }
            Err(DynamicInterfaceSignatureError::MissingMember { interface_name }) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!("Interface '{interface_name}' does not declare member '{member}'"),
                ));
                None
            }
            Err(DynamicInterfaceSignatureError::WrongKind { interface_name }) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Interface '{interface_name}' member '{member}' is a function but dynamic target access requires a knot"
                    ),
                ));
                None
            }
        }
    }

    pub(super) fn check_dynamic_interface_function_call(
        &mut self,
        target: &Expression,
        member: &str,
        arguments: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        self.check_expression(target, span, context);

        match self.dynamic_interface_function_signature(target, member, span, context) {
            Some(signature) => {
                self.check_dynamic_interface_member_arguments(
                    "function", member, arguments, &signature, span, context,
                );
            }
            None => {
                for argument in arguments {
                    self.check_expression(argument, span, context);
                }
            }
        }
    }

    fn dynamic_interface_function_signature(
        &mut self,
        target: &Expression,
        member: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) -> Option<InterfaceMemberSignature> {
        match resolve_dynamic_interface_signature(
            target,
            member,
            InterfaceMemberKind::Function,
            self.dynamic_interface_signature_inputs(),
            self.current_module(context),
            self.current_flow_path(context),
        ) {
            Ok(signature) => Some(signature),
            Err(DynamicInterfaceSignatureError::Inference(error)) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Cannot type-check dynamic interface function '{member}': {}",
                        error.message()
                    ),
                ));
                None
            }
            Err(DynamicInterfaceSignatureError::NonInterface(target_type)) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Dynamic interface function '{member}' has base type {} but expected interface",
                        target_type.display_name()
                    ),
                ));
                None
            }
            Err(DynamicInterfaceSignatureError::MissingMember { interface_name }) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!("Interface '{interface_name}' does not declare member '{member}'"),
                ));
                None
            }
            Err(DynamicInterfaceSignatureError::WrongKind { interface_name }) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Interface '{interface_name}' member '{member}' is a knot but dynamic function call requires a function"
                    ),
                ));
                None
            }
        }
    }

    fn check_dynamic_interface_member_arguments(
        &mut self,
        member_kind: &str,
        member: &str,
        arguments: &[Expression],
        signature: &InterfaceMemberSignature,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let parameters = signature.arguments();
        if arguments.len() != parameters.len() {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Dynamic interface {member_kind} '{member}' expects {} arguments but got {}",
                    parameters.len(),
                    arguments.len()
                ),
            ));
            return;
        }

        for (argument, parameter) in arguments.iter().zip(parameters) {
            let Some(expected_type) = parameter.declared_type() else {
                self.check_expression(argument, span, context);
                continue;
            };

            if is_composite_literal(argument) {
                self.check_expression(argument, span, context);
                continue;
            }

            let argument_type = check_expression_type_with_expected(
                argument,
                expected_type,
                self.expected_type_inference(),
                self.current_module(context),
                self.current_flow_path(context),
            );

            match argument_type {
                Err(ExpectedTypeCheckError::Mismatch(actual_type)) => {
                    self.diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!(
                            "Argument '{}' for dynamic interface {member_kind} '{member}' has type {} but expected {}",
                            parameter.name(),
                            actual_type.display_name(),
                            expected_type.display_name()
                        ),
                    ));
                    self.check_expression(argument, span, context);
                }
                Ok(()) => {
                    if !is_interface_module_literal_argument(argument, expected_type) {
                        self.check_expression(argument, span, context);
                    }
                }
                Err(ExpectedTypeCheckError::Inference(error)) => self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Cannot type-check argument '{}' for dynamic interface {member_kind} '{member}': {}",
                        parameter.name(),
                        error.message()
                    ),
                )),
            }
        }
    }
}

pub(super) fn is_interface_module_literal_argument(
    expression: &Expression,
    expected_type: &TypeName,
) -> bool {
    expected_type.as_interface_name().is_some()
        && matches!(expression, Expression::VariableReference(_))
}
