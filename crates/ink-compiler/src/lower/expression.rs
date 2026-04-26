use std::collections::HashSet;

use ink_story_json_format::{ControlCommand, Object as RuntimeObject};

use crate::parsed::{AssignmentTarget, BinaryOperator, Expression, FlowArgument};

use super::context::ChoicePathMode;
use super::indexes::{
    CallSignature, ConstantValue, ConstantValues, ExternalSignatures, StructDefinitions,
};
use super::path::LabelIndex;
use super::value::{lower_value_literal, resolve_divert_target_value};
use super::weave::lower_content_list_into_context;
use super::{
    collect_assignment_path, lower_assignment_path_update_value_into,
    lower_cached_assignment_indexes_into, push_reassignment_for_name, AssignmentUpdateValue,
};

pub(super) fn lower_output_expression_into(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    path_mode: &ChoicePathMode,
) {
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    lower_expression_into(
        content,
        expression,
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
        path_mode,
        false,
    );
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalOutput));
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
}

pub(super) fn lower_logic_line_into(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    path_mode: &ChoicePathMode,
) {
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    lower_expression_into(
        content,
        expression,
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
        path_mode,
        false,
    );
    content.push(RuntimeObject::ControlCommand(ControlCommand::Pop));
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    content.push(RuntimeObject::String("\n".to_string()));
}

pub(super) fn lower_expression_into(
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
) {
    let mut visiting_constants = HashSet::new();
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
            lower_content_list_into_context(
                content,
                string_content,
                path_mode,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
            );
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
            if let Some(constant) = constants.get(name) {
                if visiting_constants.insert(name.clone()) {
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
                    visiting_constants.remove(name);
                    return;
                }
            }

            content.push(RuntimeObject::VariableReference(name.clone()));
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
            content.push(RuntimeObject::NativeFunction("FIELD".to_string()));
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
            content.push(RuntimeObject::NativeFunction("INDEX".to_string()));
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
            content.push(RuntimeObject::NativeFunction(
                operator_runtime_name(*operator).to_string(),
            ));
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
            content.push(RuntimeObject::NativeFunction(
                operator.runtime_name().to_string(),
            ));
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
                    content.push(RuntimeObject::NativeFunction("&&".to_string()));
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
                lower_function_arg_into(
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
                lower_function_arg_into(
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
                lower_function_arg_into(
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
            content.push(RuntimeObject::NativeFunction(name.to_string()));
        }
        _ if matches!(
            external_signatures.get(name),
            Some(CallSignature::External { .. })
        ) =>
        {
            for arg in args {
                lower_function_arg_into(
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
                target: name.to_string(),
                args: args.len(),
            });
        }
        _ if matches!(
            external_signatures.get(name),
            Some(CallSignature::Ink { .. })
        ) =>
        {
            let expected_args = match external_signatures.get(name) {
                Some(CallSignature::Ink { args, .. }) => args.as_slice(),
                _ => &[],
            };
            for (index, arg) in args.iter().enumerate() {
                lower_function_arg_into(
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
                target: name.to_string(),
            });
        }
        _ => {
            for arg in args {
                lower_function_arg_into(
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
                target: name.to_string(),
            });
        }
    }
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
    let cached_components = lower_cached_assignment_indexes_into(
        content,
        &components,
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        path_mode,
        struct_definitions,
    );

    if cached_components.is_empty() {
        content.push(RuntimeObject::VariableReference(root_name.to_string()));
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
        content.push(RuntimeObject::NativeFunction("ARRAY_REMOVE".to_string()));
    } else {
        lower_assignment_path_update_value_into(
            content,
            root_name,
            &cached_components,
            0,
            AssignmentUpdateValue::ArrayRemove {
                index: index_expression,
            },
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            path_mode,
            struct_definitions,
        );
    }

    push_reassignment_for_name(content, root_name, path_mode);
    content.push(RuntimeObject::Void);
}

pub(super) fn lower_function_arg_into(
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
                name: name.clone(),
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
