use std::collections::HashSet;

use ink_story_json_format::{ControlCommand, Object as RuntimeObject};

use crate::parsed::{AssignmentTarget, BinaryOperator, Expression, FlowArgument};

use super::assignment::{
    collect_assignment_path, lower_assignment_path_update_value_into,
    lower_cached_assignment_indexes_into, push_reassignment_for_name, AssignmentUpdateValue,
};
use super::context::{ChoicePathMode, LoweringContext};
use super::indexes::{
    CallSignature, ConstantValue, ConstantValues, ExternalSignatures, StructDefinitions,
};
use super::native_function;
use super::path::{
    module_scoped_source_path_to_runtime_path, source_path_to_runtime_path, LabelIndex,
};
use super::value::{lower_value_literal, resolve_divert_target_value};
use super::weave::lower_content_list_into_context;

pub(super) fn lower_output_expression_into(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    context: &LoweringContext<'_>,
) {
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    lower_expression_into(content, expression, context, false);
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalOutput));
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
}

pub(super) fn lower_logic_line_into(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    context: &LoweringContext<'_>,
) {
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    lower_expression_into(content, expression, context, false);
    content.push(RuntimeObject::ControlCommand(ControlCommand::Pop));
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    content.push(RuntimeObject::String("\n".to_string()));
}

pub(super) fn lower_expression_into(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    context: &LoweringContext<'_>,
    has_start_content: bool,
) {
    let mut visiting_constants = HashSet::new();
    lower_expression_into_with_constants(
        content,
        expression,
        context.choice_labels(),
        context.global_labels(),
        context.global_variables(),
        context.external_signatures(),
        context.constants(),
        context.struct_definitions(),
        context.path_mode(),
        has_start_content,
        &mut visiting_constants,
    );
}

fn lower_expression_into_with_constants(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    path_mode: &ChoicePathMode,
    has_start_content: bool,
    visiting_constants: &mut HashSet<String>,
) {
    match expression {
        Expression::String(value) => {
            content.push(RuntimeObject::ControlCommand(ControlCommand::BeginString));
            content.push(RuntimeObject::String(value.clone()));
            content.push(RuntimeObject::ControlCommand(ControlCommand::EndString));
        }
        Expression::StringContent(string_content) => {
            content.push(RuntimeObject::ControlCommand(ControlCommand::BeginString));
            let context = LoweringContext::new(
                path_mode.clone(),
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
            );
            lower_content_list_into_context(content, string_content, &context);
            content.push(RuntimeObject::ControlCommand(ControlCommand::EndString));
        }
        Expression::NumberInt(value) => content.push(RuntimeObject::Int(*value)),
        Expression::NumberFloat(value) => content.push(RuntimeObject::Float(value.value())),
        Expression::NumberBool(value) => content.push(RuntimeObject::Bool(*value)),
        Expression::DivertTarget(target) => {
            content.push(RuntimeObject::DivertTarget(resolve_divert_target_value(
                target,
                choice_labels,
                global_labels,
                path_mode,
            )));
        }
        Expression::VariableReference(name) => {
            let resolved_name = resolve_runtime_variable_name(name, path_mode, global_variables);
            if let Some(constant_name) = resolve_constant_name(name, path_mode, constants) {
                let constant = constants
                    .get(constant_name.as_str())
                    .expect("resolved constant name must exist");
                if visiting_constants.insert(constant_name.clone()) {
                    lower_constant_expression_into(
                        content,
                        constant,
                        choice_labels,
                        global_labels,
                        global_variables,
                        external_signatures,
                        constants,
                        struct_definitions,
                        path_mode,
                        has_start_content,
                        visiting_constants,
                    );
                    visiting_constants.remove(constant_name.as_str());
                    return;
                }
            }

            content.push(RuntimeObject::VariableReference(resolved_name));
        }
        Expression::QualifiedReference(name) => {
            if let Some(constant) = constants.get(name.as_str()) {
                if visiting_constants.insert(name.as_str().to_string()) {
                    lower_constant_expression_into(
                        content,
                        constant,
                        choice_labels,
                        global_labels,
                        global_variables,
                        external_signatures,
                        constants,
                        struct_definitions,
                        path_mode,
                        has_start_content,
                        visiting_constants,
                    );
                    visiting_constants.remove(name.as_str());
                    return;
                }
            }

            content.push(RuntimeObject::VariableReference(name.as_str().to_string()));
        }
        Expression::FunctionCall { name, args } => {
            lower_function_call_into(
                content,
                name,
                args,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
                path_mode,
                has_start_content,
                visiting_constants,
            );
        }
        Expression::QualifiedFunctionCall { name, args } => {
            lower_function_call_into(
                content,
                name.as_str(),
                args,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
                path_mode,
                has_start_content,
                visiting_constants,
            );
        }
        Expression::ArrayLiteral(_) | Expression::StructLiteral(_) => {
            if let Some(value) = lower_value_literal(
                expression,
                None,
                struct_definitions,
                choice_labels,
                global_labels,
                path_mode,
            ) {
                content.push(value);
            }
        }
        Expression::FieldAccess { base, field } => {
            lower_expression_into_with_constants(
                content,
                base,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
                path_mode,
                has_start_content,
                visiting_constants,
            );
            content.push(RuntimeObject::String(field.clone()));
            content.push(native_function("FIELD"));
        }
        Expression::IndexAccess { base, index } => {
            lower_expression_into_with_constants(
                content,
                base,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
                path_mode,
                has_start_content,
                visiting_constants,
            );
            lower_expression_into_with_constants(
                content,
                index,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
                path_mode,
                has_start_content,
                visiting_constants,
            );
            content.push(native_function("INDEX"));
        }
        Expression::Binary {
            operator,
            left,
            right,
        } => {
            lower_expression_into_with_constants(
                content,
                left,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
                path_mode,
                has_start_content,
                visiting_constants,
            );
            lower_expression_into_with_constants(
                content,
                right,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
                path_mode,
                has_start_content,
                visiting_constants,
            );
            content.push(native_function(operator_runtime_name(*operator)));
        }
        Expression::Unary {
            operator,
            expression,
        } => {
            lower_expression_into_with_constants(
                content,
                expression,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
                path_mode,
                has_start_content,
                visiting_constants,
            );
            content.push(native_function(operator.runtime_name()));
        }
        Expression::MultipleCondition(expressions) => {
            for (index, expression) in expressions.iter().enumerate() {
                lower_expression_into_with_constants(
                    content,
                    expression,
                    choice_labels,
                    global_labels,
                    global_variables,
                    external_signatures,
                    constants,
                    struct_definitions,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
                if index > 0 {
                    content.push(native_function("&&"));
                }
            }
        }
    }
}

fn lower_constant_expression_into(
    content: &mut Vec<RuntimeObject>,
    constant: &ConstantValue,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    path_mode: &ChoicePathMode,
    has_start_content: bool,
    visiting_constants: &mut HashSet<String>,
) {
    if let Some(value) = lower_value_literal(
        constant.expression(),
        Some(constant.declared_type()),
        struct_definitions,
        choice_labels,
        global_labels,
        path_mode,
    ) {
        content.push(value);
        return;
    }

    lower_expression_into_with_constants(
        content,
        constant.expression(),
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
        path_mode,
        has_start_content,
        visiting_constants,
    );
}

fn lower_function_call_into(
    content: &mut Vec<RuntimeObject>,
    name: &str,
    args: &[Expression],
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    path_mode: &ChoicePathMode,
    has_start_content: bool,
    visiting_constants: &mut HashSet<String>,
) {
    let resolved_name = resolve_callable_name(name, external_signatures, path_mode);
    match name {
        "ARRAY_REMOVE" => {
            lower_array_remove_call_into(
                content,
                args,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
                path_mode,
                visiting_constants,
            );
        }
        "RANDOM" => {
            for arg in args {
                lower_function_arg_into_parts(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    global_variables,
                    external_signatures,
                    constants,
                    struct_definitions,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::Random));
        }
        "SEED_RANDOM" => {
            for arg in args {
                lower_function_arg_into_parts(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    global_variables,
                    external_signatures,
                    constants,
                    struct_definitions,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::SeedRandom));
        }
        _ if is_builtin_function(name) => {
            for arg in args {
                lower_function_arg_into_parts(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    global_variables,
                    external_signatures,
                    constants,
                    struct_definitions,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(native_function(name));
        }
        _ if matches!(
            external_signatures.get(resolved_name.as_str()),
            Some(CallSignature::External { .. })
        ) =>
        {
            for arg in args {
                lower_function_arg_into_parts(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    global_variables,
                    external_signatures,
                    constants,
                    struct_definitions,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::ExternalFunction {
                target: resolved_name.clone(),
                args: args.len(),
            });
        }
        _ if matches!(
            external_signatures.get(resolved_name.as_str()),
            Some(CallSignature::Ink { .. })
        ) =>
        {
            let expected_args = match external_signatures.get(resolved_name.as_str()) {
                Some(CallSignature::Ink { args, .. }) => args.as_slice(),
                _ => &[],
            };
            for (index, arg) in args.iter().enumerate() {
                lower_function_arg_into_parts(
                    content,
                    arg,
                    expected_args.get(index),
                    choice_labels,
                    global_labels,
                    global_variables,
                    external_signatures,
                    constants,
                    struct_definitions,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::FunctionDivert {
                target: runtime_function_target(resolved_name.as_str(), path_mode),
            });
        }
        _ => {
            for arg in args {
                lower_function_arg_into_parts(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    global_variables,
                    external_signatures,
                    constants,
                    struct_definitions,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::FunctionDivert {
                target: runtime_function_target(resolved_name.as_str(), path_mode),
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

fn lower_array_remove_call_into(
    content: &mut Vec<RuntimeObject>,
    args: &[Expression],
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    path_mode: &ChoicePathMode,
    visiting_constants: &mut HashSet<String>,
) {
    let (Some(target_expression), Some(index_expression)) = (args.first(), args.get(1)) else {
        content.push(RuntimeObject::Void);
        return;
    };
    let Some(target) = AssignmentTarget::from_expression(target_expression.clone()) else {
        content.push(RuntimeObject::Void);
        return;
    };

    let mut components = Vec::new();
    let Some(root_name) = collect_assignment_path(&target, &mut components) else {
        content.push(RuntimeObject::Void);
        return;
    };
    let context = LoweringContext::new(
        path_mode.clone(),
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
    );
    let cached_components = lower_cached_assignment_indexes_into(content, &components, &context);
    let resolved_root_name = resolve_runtime_variable_name(root_name, path_mode, global_variables);

    if cached_components.is_empty() {
        content.push(RuntimeObject::VariableReference(resolved_root_name.clone()));
        lower_expression_into_with_constants(
            content,
            index_expression,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            struct_definitions,
            path_mode,
            false,
            visiting_constants,
        );
        content.push(native_function("ARRAY_REMOVE"));
    } else {
        lower_assignment_path_update_value_into(
            content,
            resolved_root_name.as_str(),
            &cached_components,
            0,
            AssignmentUpdateValue::ArrayRemove {
                index: index_expression,
            },
            &context,
        );
    }

    push_reassignment_for_name(content, resolved_root_name.as_str(), path_mode);
    content.push(RuntimeObject::Void);
}

pub(super) fn lower_function_arg_into(
    content: &mut Vec<RuntimeObject>,
    arg: &Expression,
    expected_arg: Option<&FlowArgument>,
    context: &LoweringContext<'_>,
    has_start_content: bool,
    visiting_constants: &mut HashSet<String>,
) {
    lower_function_arg_into_parts(
        content,
        arg,
        expected_arg,
        context.choice_labels(),
        context.global_labels(),
        context.global_variables(),
        context.external_signatures(),
        context.constants(),
        context.struct_definitions(),
        context.path_mode(),
        has_start_content,
        visiting_constants,
    );
}

fn lower_function_arg_into_parts(
    content: &mut Vec<RuntimeObject>,
    arg: &Expression,
    expected_arg: Option<&FlowArgument>,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    path_mode: &ChoicePathMode,
    has_start_content: bool,
    visiting_constants: &mut HashSet<String>,
) {
    if expected_arg.is_some_and(FlowArgument::is_by_reference) {
        if let Expression::VariableReference(name) = arg {
            content.push(RuntimeObject::VariablePointer {
                name: resolve_runtime_variable_name(name, path_mode, global_variables),
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

    if let Expression::ArrayLiteral(_) | Expression::StructLiteral(_) = arg {
        if let Some(expected_type) = expected_arg.and_then(FlowArgument::declared_type) {
            if let Some(value) = lower_value_literal(
                arg,
                Some(expected_type),
                struct_definitions,
                choice_labels,
                global_labels,
                path_mode,
            ) {
                content.push(value);
                return;
            }
        }
    }

    lower_expression_into_with_constants(
        content,
        arg,
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
        path_mode,
        has_start_content,
        visiting_constants,
    );
}

fn operator_runtime_name(operator: BinaryOperator) -> &'static str {
    operator.runtime_name()
}

fn is_builtin_function(name: &str) -> bool {
    matches!(
        name,
        "MIN" | "MAX" | "POW" | "FLOOR" | "CEILING" | "INT" | "FLOAT" | "LEN"
    )
}

fn resolve_runtime_variable_name(
    name: &str,
    path_mode: &ChoicePathMode,
    global_variables: &HashSet<String>,
) -> String {
    if name.contains("::") || path_mode.is_local_variable(name) {
        return name.to_string();
    }

    path_mode
        .current_module_name()
        .map(|module_name| format!("{module_name}::{name}"))
        .filter(|qualified_name| global_variables.contains(qualified_name))
        .unwrap_or_else(|| name.to_string())
}

fn resolve_constant_name(
    name: &str,
    path_mode: &ChoicePathMode,
    constants: &ConstantValues,
) -> Option<String> {
    if constants.contains_key(name) {
        return Some(name.to_string());
    }

    if name.contains("::") {
        return None;
    }

    path_mode
        .current_module_name()
        .map(|module_name| format!("{module_name}::{name}"))
        .filter(|qualified_name| constants.contains_key(qualified_name))
}

fn resolve_callable_name(
    name: &str,
    external_signatures: &ExternalSignatures,
    path_mode: &ChoicePathMode,
) -> String {
    if external_signatures.contains_key(name) || name.contains("::") {
        return name.to_string();
    }

    path_mode
        .current_module_name()
        .map(|module_name| format!("{module_name}::{name}"))
        .filter(|qualified_name| external_signatures.contains_key(qualified_name))
        .unwrap_or_else(|| name.to_string())
}
