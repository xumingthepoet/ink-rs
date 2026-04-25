use std::collections::{HashMap, HashSet};

use crate::parsed::{BinaryOperator, Expression, FlowArgument};

use super::context::ChoicePathMode;
use super::indexes::{CallSignature, ExternalSignatures};
use super::ir::{ControlCommand, RuntimeObject};
use super::weave::lower_content_list_into_context;

pub(super) fn lower_output_expression_into(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
) {
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    lower_expression_into(
        content,
        expression,
        choice_labels,
        global_labels,
        external_signatures,
        constants,
        path_mode,
        false,
    );
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalOutput));
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
}

pub(super) fn lower_logic_line_into(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
) {
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    lower_expression_into(
        content,
        expression,
        choice_labels,
        global_labels,
        external_signatures,
        constants,
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
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
    has_start_content: bool,
) {
    let mut visiting_constants = HashSet::new();
    lower_expression_into_with_constants(
        content,
        expression,
        choice_labels,
        global_labels,
        external_signatures,
        constants,
        path_mode,
        has_start_content,
        &mut visiting_constants,
    );
}

fn lower_expression_into_with_constants(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
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
                &HashSet::new(),
                external_signatures,
                constants,
            );
            content.push(RuntimeObject::ControlCommand(ControlCommand::EndString));
        }
        Expression::NumberInt(value) => content.push(RuntimeObject::Int(*value)),
        Expression::NumberFloat(value) => content.push(RuntimeObject::Float(*value)),
        Expression::NumberBool(value) => content.push(RuntimeObject::Bool(*value)),
        Expression::DivertTarget(target) => {
            let resolved_target = if let Some(choice_target) = choice_labels.get(target) {
                choice_target.clone()
            } else if let Some(label_target) = path_mode
                .scoped_label_target(target, global_labels)
                .filter(|label_target| label_target.as_str() != target)
            {
                path_mode.resolve_label_target(label_target)
            } else {
                path_mode.resolve_divert_target(target)
            };
            content.push(RuntimeObject::DivertTarget(resolved_target));
        }
        Expression::VariableReference(name) => {
            if let Some(constant) = constants.get(name) {
                if visiting_constants.insert(name.clone()) {
                    lower_expression_into_with_constants(
                        content,
                        constant,
                        choice_labels,
                        global_labels,
                        external_signatures,
                        constants,
                        path_mode,
                        has_start_content,
                        visiting_constants,
                    );
                    visiting_constants.remove(name);
                    return;
                }
            }

            if let Some(choice_target) = choice_labels.get(name) {
                let _ = has_start_content;
                content.push(RuntimeObject::ReadCount(choice_target.clone()));
            } else if let Some(label_target) = path_mode.scoped_label_target(name, global_labels) {
                content.push(RuntimeObject::ReadCount(
                    path_mode.resolve_label_target(label_target),
                ));
            } else if path_mode.is_flow_sibling_stitch(name) {
                content.push(RuntimeObject::ReadCount(
                    path_mode.resolve_single_stitch_target(name),
                ));
            } else {
                content.push(RuntimeObject::VariableReference(name.clone()));
            }
        }
        Expression::FunctionCall { name, args } => {
            lower_function_call_into(
                content,
                name,
                args,
                choice_labels,
                global_labels,
                external_signatures,
                constants,
                path_mode,
                has_start_content,
                visiting_constants,
            );
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
                external_signatures,
                constants,
                path_mode,
                has_start_content,
                visiting_constants,
            );
            lower_expression_into_with_constants(
                content,
                right,
                choice_labels,
                global_labels,
                external_signatures,
                constants,
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
                external_signatures,
                constants,
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
                    external_signatures,
                    constants,
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

fn lower_function_call_into(
    content: &mut Vec<RuntimeObject>,
    name: &str,
    args: &[Expression],
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
    has_start_content: bool,
    visiting_constants: &mut HashSet<String>,
) {
    match name {
        "CHOICE_COUNT" => content.push(RuntimeObject::ControlCommand(ControlCommand::ChoiceCount)),
        "TURNS" => content.push(RuntimeObject::ControlCommand(ControlCommand::Turns)),
        "TURNS_SINCE" => {
            if let Some(arg) = args.first() {
                lower_function_arg_into(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::TurnsSince));
        }
        "READ_COUNT" => {
            if let Some(arg) = args.first() {
                lower_function_arg_into(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::ReadCount));
        }
        "RANDOM" => {
            for arg in args {
                lower_function_arg_into(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
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
                    external_signatures,
                    constants,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::SeedRandom));
        }
        "LIST_RANGE" => {
            for arg in args {
                lower_function_arg_into(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::ListRange));
        }
        "LIST_RANDOM" => {
            for arg in args {
                lower_function_arg_into(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::ListRandom));
        }
        _ if is_builtin_function(name) => {
            for arg in args {
                lower_function_arg_into(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
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
                    external_signatures,
                    constants,
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
                Some(CallSignature::Ink { args }) => args.as_slice(),
                _ => &[],
            };
            for (index, arg) in args.iter().enumerate() {
                lower_function_arg_into(
                    content,
                    arg,
                    expected_args.get(index),
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
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
                    external_signatures,
                    constants,
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

fn lower_function_arg_into(
    content: &mut Vec<RuntimeObject>,
    arg: &Expression,
    expected_arg: Option<&FlowArgument>,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
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

    lower_expression_into_with_constants(
        content,
        arg,
        choice_labels,
        global_labels,
        external_signatures,
        constants,
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
        "LIST_VALUE"
            | "MIN"
            | "MAX"
            | "POW"
            | "FLOOR"
            | "CEILING"
            | "INT"
            | "FLOAT"
            | "LIST_MIN"
            | "LIST_MAX"
            | "LIST_ALL"
            | "LIST_COUNT"
            | "LIST_INVERT"
    )
}
