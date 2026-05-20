use crate::{
    diagnostic::Diagnostic,
    parsed::{visit::VisitContext, Expression, TypeName},
    source::SourceSpan,
};

use super::super::{
    argument_resolution::{
        resolve_function_call_expected_arguments, FunctionCallArgumentResolution,
        ResolvedExpectedArguments,
    },
    expression_types::{infer_expression_type, is_typed_builtin_function, TypeInferenceError},
    interface_values::{check_expression_type_with_expected, ExpectedTypeCheckError},
};
use super::checker::{
    is_composite_literal, is_interface_module_literal_argument, CallTargetChecker,
};

impl<'a> CallTargetChecker<'a> {
    pub(super) fn check_function_call(
        &mut self,
        name: &str,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if self.check_cross_module_stitch_target(name, span, context) {
            for arg in args {
                self.check_expression(arg, span, context);
            }
            return;
        }

        if is_typed_builtin_function(name) {
            let skip_argument_indexes = self.check_typed_builtin_call(name, args, span, context);
            for (index, arg) in args.iter().enumerate() {
                if skip_argument_indexes.contains(&index) {
                    continue;
                }
                self.check_expression(arg, span, context);
            }
            return;
        }

        if is_runtime_builtin_function(name) {
            for arg in args {
                self.check_expression(arg, span, context);
            }
            return;
        }

        match resolve_function_call_expected_arguments(
            name,
            self.current_module(context),
            self.current_flow_path(context),
            self.target_symbols,
        ) {
            FunctionCallArgumentResolution::Function(expected_arguments) => {
                self.check_function_call_signature(name, args, &expected_arguments, span, context);
            }
            FunctionCallArgumentResolution::NonFunction => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "{name} hasn't been marked as a function, but it's being called as one. Do you need to declare the knot as '== function {name} =='?"
                    ),
                ));
                for arg in args {
                    self.check_expression(arg, span, context);
                }
            }
            FunctionCallArgumentResolution::Missing => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!("Function '{name}' is not declared"),
                ));
                for arg in args {
                    self.check_expression(arg, span, context);
                }
            }
        }
    }

    pub(super) fn infer_call_argument_type(
        &self,
        arg: &Expression,
        context: &VisitContext,
    ) -> Result<TypeName, TypeInferenceError> {
        infer_expression_type(
            arg,
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            self.interface_members,
            self.current_module(context),
            self.current_flow_path(context),
        )
    }

    fn check_function_call_signature(
        &mut self,
        name: &str,
        args: &[Expression],
        expected_arguments: &ResolvedExpectedArguments,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let parameters = expected_arguments.arguments();
        if args.len() != parameters.len() {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Function '{name}' expects {} arguments but got {}",
                    parameters.len(),
                    args.len()
                ),
            ));
            for arg in args {
                self.check_expression(arg, span, context);
            }
            return;
        }

        for (argument, parameter) in args.iter().zip(parameters) {
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
                &expected_type,
                self.expected_type_inference(),
                self.current_module(context),
                self.current_flow_path(context),
            );

            match argument_type {
                Err(ExpectedTypeCheckError::Mismatch(actual_type)) => {
                    self.diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!(
                            "Argument '{}' for function '{name}' has type {} but expected {}",
                            parameter.name(),
                            actual_type.display_name(),
                            expected_type.display_name()
                        ),
                    ));
                    self.check_expression(argument, span, context);
                }
                Ok(()) => {
                    if !is_interface_module_literal_argument(argument, &expected_type) {
                        self.check_expression(argument, span, context);
                    }
                }
                Err(ExpectedTypeCheckError::Inference(error)) => {
                    self.diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!(
                            "Cannot type-check argument '{}' for function '{name}': {}",
                            parameter.name(),
                            error.message()
                        ),
                    ))
                }
            }
        }
    }
}

fn is_runtime_builtin_function(name: &str) -> bool {
    matches!(
        name,
        "RANDOM" | "SEED_RANDOM" | "MIN" | "MAX" | "POW" | "FLOOR" | "CEILING" | "INT" | "FLOAT"
    )
}
