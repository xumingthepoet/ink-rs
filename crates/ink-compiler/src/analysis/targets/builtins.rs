use crate::{
    diagnostic::Diagnostic,
    parsed::{visit::VisitContext, DictKeyType, Expression, TypeName},
    source::SourceSpan,
};

use super::super::interface_values::{check_expression_type_with_expected, ExpectedTypeCheckError};
use super::checker::CallTargetChecker;
use super::context::{is_interface_module_literal_argument, is_mutable_lvalue};

impl<'a> CallTargetChecker<'a> {
    pub(super) fn check_typed_builtin_call(
        &mut self,
        name: &str,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) -> Vec<usize> {
        match name {
            "ARRAY_REMOVE" => {
                self.check_array_remove_call(args, span, context);
                Vec::new()
            }
            "ARRAY_PUSH" => self.check_array_push_call(args, span, context),
            "ARRAY_INSERT" => self.check_array_insert_call(args, span, context),
            "LEN" => {
                self.check_len_call(args, span, context);
                Vec::new()
            }
            "DICT_HAS" => {
                self.check_dict_has_call(args, span, context);
                Vec::new()
            }
            "DICT_SIZE" => {
                self.check_dict_size_call(args, span, context);
                Vec::new()
            }
            "DICT_REMOVE" => {
                self.check_dict_remove_call(args, span, context);
                Vec::new()
            }
            "DICT_KEYS" => {
                self.check_dict_keys_call(args, span, context);
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn check_array_remove_call(
        &mut self,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if args.len() != 2 {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Builtin 'ARRAY_REMOVE' expects 2 arguments but got {}",
                    args.len()
                ),
            ));
            return;
        }

        if !is_mutable_lvalue(&args[0]) {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                "First argument for builtin 'ARRAY_REMOVE' must be a mutable lvalue",
            ));
        }

        match self.infer_call_argument_type(&args[0], context) {
            Ok(argument_type) if argument_type.array_element_type().is_none() => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "First argument for builtin 'ARRAY_REMOVE' has type {} but expected array",
                        argument_type.display_name()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Cannot type-check first argument for builtin 'ARRAY_REMOVE': {}",
                    error.message()
                ),
            )),
        }

        match self.infer_call_argument_type(&args[1], context) {
            Ok(argument_type) if argument_type != TypeName::int() => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Second argument for builtin 'ARRAY_REMOVE' has type {} but expected int",
                        argument_type.display_name()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Cannot type-check second argument for builtin 'ARRAY_REMOVE': {}",
                    error.message()
                ),
            )),
        }
    }

    fn check_len_call(&mut self, args: &[Expression], span: &SourceSpan, context: &VisitContext) {
        if args.len() != 1 {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!("Builtin 'LEN' expects 1 argument but got {}", args.len()),
            ));
            return;
        }

        match self.infer_call_argument_type(&args[0], context) {
            Ok(argument_type) if argument_type.array_element_type().is_none() => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Argument for builtin 'LEN' has type {} but expected array",
                        argument_type.display_name()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Cannot type-check argument for builtin 'LEN': {}",
                    error.message()
                ),
            )),
        }
    }

    fn check_array_push_call(
        &mut self,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) -> Vec<usize> {
        if args.len() != 2 {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Builtin 'ARRAY_PUSH' expects 2 arguments but got {}",
                    args.len()
                ),
            ));
            return Vec::new();
        }

        if !is_mutable_lvalue(&args[0]) {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                "First argument for builtin 'ARRAY_PUSH' must be a mutable lvalue",
            ));
        }

        let Some(element_type) =
            self.check_array_argument("ARRAY_PUSH", "First", &args[0], span, context)
        else {
            return Vec::new();
        };
        self.check_array_value_argument(
            "ARRAY_PUSH",
            "Second",
            &element_type,
            &args[1],
            span,
            context,
        )
        .then_some(1)
        .into_iter()
        .collect()
    }

    fn check_array_insert_call(
        &mut self,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) -> Vec<usize> {
        if args.len() != 3 {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Builtin 'ARRAY_INSERT' expects 3 arguments but got {}",
                    args.len()
                ),
            ));
            return Vec::new();
        }

        if !is_mutable_lvalue(&args[0]) {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                "First argument for builtin 'ARRAY_INSERT' must be a mutable lvalue",
            ));
        }

        let Some(element_type) =
            self.check_array_argument("ARRAY_INSERT", "First", &args[0], span, context)
        else {
            return Vec::new();
        };
        self.check_int_argument("ARRAY_INSERT", "Second", &args[1], span, context);
        self.check_array_value_argument(
            "ARRAY_INSERT",
            "Third",
            &element_type,
            &args[2],
            span,
            context,
        )
        .then_some(2)
        .into_iter()
        .collect()
    }

    fn check_array_argument(
        &mut self,
        builtin: &str,
        ordinal: &str,
        arg: &Expression,
        span: &SourceSpan,
        context: &VisitContext,
    ) -> Option<TypeName> {
        match self.infer_call_argument_type(arg, context) {
            Ok(argument_type) => {
                let Some(element_type) = argument_type.array_element_type() else {
                    self.diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!(
                            "{} argument for builtin '{}' has type {} but expected array",
                            ordinal,
                            builtin,
                            argument_type.display_name()
                        ),
                    ));
                    return None;
                };
                Some(element_type.clone())
            }
            Err(error) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Cannot type-check {} argument for builtin '{}': {}",
                        ordinal.to_lowercase(),
                        builtin,
                        error.message()
                    ),
                ));
                None
            }
        }
    }

    fn check_int_argument(
        &mut self,
        builtin: &str,
        ordinal: &str,
        arg: &Expression,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        match self.infer_call_argument_type(arg, context) {
            Ok(argument_type) if argument_type != TypeName::int() => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "{} argument for builtin '{}' has type {} but expected int",
                        ordinal,
                        builtin,
                        argument_type.display_name()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Cannot type-check {} argument for builtin '{}': {}",
                    ordinal.to_lowercase(),
                    builtin,
                    error.message()
                ),
            )),
        }
    }

    fn check_array_value_argument(
        &mut self,
        builtin: &str,
        ordinal: &str,
        expected_type: &TypeName,
        arg: &Expression,
        span: &SourceSpan,
        context: &VisitContext,
    ) -> bool {
        let argument_type = check_expression_type_with_expected(
            arg,
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
                        "{} argument for builtin '{}' has type {} but expected {}",
                        ordinal,
                        builtin,
                        actual_type.display_name(),
                        expected_type.display_name()
                    ),
                ));
                false
            }
            Ok(()) => is_interface_module_literal_argument(arg, expected_type),
            Err(ExpectedTypeCheckError::Inference(error)) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Cannot type-check {} argument for builtin '{}': {}",
                        ordinal.to_lowercase(),
                        builtin,
                        error.message()
                    ),
                ));
                false
            }
        }
    }

    fn check_dict_has_call(
        &mut self,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if args.len() != 2 {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Builtin 'DICT_HAS' expects 2 arguments but got {}",
                    args.len()
                ),
            ));
            return;
        }

        let Some((key_type, _)) =
            self.check_dict_argument("DICT_HAS", "First", &args[0], span, context)
        else {
            return;
        };
        self.check_dict_key_argument("DICT_HAS", "Second", key_type, &args[1], span, context);
    }

    fn check_dict_size_call(
        &mut self,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if args.len() != 1 {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Builtin 'DICT_SIZE' expects 1 argument but got {}",
                    args.len()
                ),
            ));
            return;
        }

        self.check_dict_argument("DICT_SIZE", "First", &args[0], span, context);
    }

    fn check_dict_remove_call(
        &mut self,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if args.len() != 2 {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Builtin 'DICT_REMOVE' expects 2 arguments but got {}",
                    args.len()
                ),
            ));
            return;
        }

        if !is_mutable_lvalue(&args[0]) {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                "First argument for builtin 'DICT_REMOVE' must be a mutable lvalue",
            ));
        }

        let Some((key_type, _)) =
            self.check_dict_argument("DICT_REMOVE", "First", &args[0], span, context)
        else {
            return;
        };
        self.check_dict_key_argument("DICT_REMOVE", "Second", key_type, &args[1], span, context);
    }

    fn check_dict_keys_call(
        &mut self,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if args.len() != 1 {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Builtin 'DICT_KEYS' expects 1 argument but got {}",
                    args.len()
                ),
            ));
            return;
        }

        self.check_dict_argument("DICT_KEYS", "First", &args[0], span, context);
    }

    fn check_dict_argument(
        &mut self,
        builtin: &str,
        ordinal: &str,
        arg: &Expression,
        span: &SourceSpan,
        context: &VisitContext,
    ) -> Option<(DictKeyType, TypeName)> {
        match self.infer_call_argument_type(arg, context) {
            Ok(argument_type) => {
                let Some((key_type, value_type)) = argument_type.dict_key_value_types() else {
                    self.diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!(
                            "{} argument for builtin '{}' has type {} but expected Dict",
                            ordinal,
                            builtin,
                            argument_type.display_name()
                        ),
                    ));
                    return None;
                };
                Some((key_type, value_type.clone()))
            }
            Err(error) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Cannot type-check {} argument for builtin '{}': {}",
                        ordinal.to_lowercase(),
                        builtin,
                        error.message()
                    ),
                ));
                None
            }
        }
    }

    fn check_dict_key_argument(
        &mut self,
        builtin: &str,
        ordinal: &str,
        expected_key_type: DictKeyType,
        arg: &Expression,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let expected_type = dict_key_type_name(expected_key_type);
        match self.infer_call_argument_type(arg, context) {
            Ok(argument_type) if argument_type != expected_type => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "{} argument for builtin '{}' has type {} but expected {}",
                        ordinal,
                        builtin,
                        argument_type.display_name(),
                        expected_type.display_name()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Cannot type-check {} argument for builtin '{}': {}",
                    ordinal.to_lowercase(),
                    builtin,
                    error.message()
                ),
            )),
        }
    }
}

fn dict_key_type_name(key_type: DictKeyType) -> TypeName {
    match key_type {
        DictKeyType::String => TypeName::string(),
        DictKeyType::Int => TypeName::int(),
    }
}
