use std::collections::HashSet;

use ink_story_json_format::{ControlCommand, Object as RuntimeObject};

use crate::parsed::{Expression, FlowArgument};

use super::super::{
    context::{ChoicePathMode, LoweringContext},
    indexes::CallSignature,
    path::{module_scoped_source_path_to_runtime_path, source_path_to_runtime_path},
};
use super::builtins::{
    lower_array_insert_call_into, lower_array_push_call_into, lower_array_remove_call_into,
    lower_dict_remove_call_into,
};
use super::name_resolution::{resolve_callable_name, resolve_runtime_variable_name};
use super::operators::builtin_native_function;
use super::{
    lower_expression_into_with_constants, lower_expression_with_expected_type_into_with_constants,
    ExpressionLoweringContext,
};

pub(super) fn lower_function_call_into(
    content: &mut Vec<RuntimeObject>,
    name: &str,
    args: &[Expression],
    lowering: &mut ExpressionLoweringContext<'_, '_>,
) {
    let context = lowering.context;
    let resolved_name =
        resolve_callable_name(name, context.external_signatures(), context.path_mode());
    let builtin_function = builtin_native_function(name);
    match name {
        "ARRAY_REMOVE" => {
            lower_array_remove_call_into(content, args, lowering);
        }
        "ARRAY_PUSH" => {
            lower_array_push_call_into(content, args, lowering);
        }
        "ARRAY_INSERT" => {
            lower_array_insert_call_into(content, args, lowering);
        }
        "DICT_REMOVE" => {
            lower_dict_remove_call_into(content, args, lowering);
        }
        "RANDOM" => {
            for arg in args {
                lower_function_arg_into_parts(content, arg, None, lowering);
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::Random));
        }
        "SEED_RANDOM" => {
            for arg in args {
                lower_function_arg_into_parts(content, arg, None, lowering);
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::SeedRandom));
        }
        _ if builtin_function.is_some() => {
            for arg in args {
                lower_function_arg_into_parts(content, arg, None, lowering);
            }
            content.push(RuntimeObject::NativeFunction(
                builtin_function.expect("builtin function guard should provide a native function"),
            ));
        }
        _ if matches!(
            context.external_signatures().get(resolved_name.as_str()),
            Some(CallSignature::External { .. })
        ) =>
        {
            for arg in args {
                lower_function_arg_into_parts(content, arg, None, lowering);
            }
            content.push(RuntimeObject::ExternalFunction {
                target: resolved_name.clone(),
                args: args.len(),
            });
        }
        _ if matches!(
            context.external_signatures().get(resolved_name.as_str()),
            Some(CallSignature::Internal { .. })
        ) =>
        {
            let expected_args = match context.external_signatures().get(resolved_name.as_str()) {
                Some(CallSignature::Internal { args, .. }) => args.as_slice(),
                _ => &[],
            };
            for (index, arg) in args.iter().enumerate() {
                lower_function_arg_into_parts(content, arg, expected_args.get(index), lowering);
            }
            content.push(RuntimeObject::FunctionDivert {
                target: runtime_function_target(resolved_name.as_str(), context.path_mode()),
            });
        }
        _ => {
            for arg in args {
                lower_function_arg_into_parts(content, arg, None, lowering);
            }
            content.push(RuntimeObject::FunctionDivert {
                target: runtime_function_target(resolved_name.as_str(), context.path_mode()),
            });
        }
    }
}

fn runtime_function_target(name: &str, path_mode: &ChoicePathMode) -> String {
    if name.contains("::") {
        return source_path_to_runtime_path(name);
    }

    module_scoped_source_path_to_runtime_path(path_mode.current_module_name(), name)
}

pub(in crate::lower) fn lower_function_arg_into(
    content: &mut Vec<RuntimeObject>,
    arg: &Expression,
    expected_arg: Option<&FlowArgument>,
    context: &LoweringContext<'_>,
    has_start_content: bool,
    visiting_constants: &mut HashSet<String>,
) {
    let mut lowering = ExpressionLoweringContext {
        context,
        has_start_content,
        visiting_constants,
    };
    lower_function_arg_into_parts(content, arg, expected_arg, &mut lowering);
}

pub(super) fn lower_function_arg_into_parts(
    content: &mut Vec<RuntimeObject>,
    arg: &Expression,
    expected_arg: Option<&FlowArgument>,
    lowering: &mut ExpressionLoweringContext<'_, '_>,
) {
    let context = lowering.context;
    if expected_arg.is_some_and(FlowArgument::is_by_reference) {
        if let Expression::VariableReference(name) = arg {
            content.push(RuntimeObject::VariablePointer {
                name: resolve_runtime_variable_name(
                    name,
                    context.path_mode(),
                    context.global_variables(),
                ),
                context_index: -1,
            });
            return;
        }
        if let Expression::QualifiedReference(name) = arg {
            content.push(RuntimeObject::VariablePointer {
                name: name.as_str().to_string(),
                context_index: -1,
            });
            return;
        }
    }

    if let Some(expected_type) = expected_arg.and_then(FlowArgument::declared_type) {
        if lower_expression_with_expected_type_into_with_constants(
            content,
            arg,
            Some(expected_type),
            lowering,
        ) {
            return;
        }
    }

    lower_expression_into_with_constants(content, arg, lowering);
}
