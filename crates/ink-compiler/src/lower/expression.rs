use std::collections::HashSet;

use ink_story_json_format::{ControlCommand, NativeFunction, Object as RuntimeObject};

use crate::parsed::{Expression, TypeName};

use super::composite_literal::lower_dynamic_composite_literal_into;
use super::context::{ChoicePathMode, LoweringContext};
use super::indexes::ConstantValue;
use super::value::{
    lower_enum_member_expression_value, lower_value_literal, resolve_divert_target_value,
};
use super::weave::lower_content_list_into_context;

mod builtins;
mod calls;
mod dynamic_interface;
mod name_resolution;
mod operators;
mod types;

pub(super) use self::calls::lower_function_arg_into;
use self::calls::lower_function_call_into;
pub(super) use self::dynamic_interface::dynamic_interface_knot_signature;
use self::dynamic_interface::{
    lower_dynamic_interface_function_call_into, lower_dynamic_interface_target_into,
};
use self::name_resolution::{resolve_constant_name, resolve_runtime_variable_name};
use self::operators::{native_function_for_binary_operator, native_function_for_unary_operator};

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
    let mut lowering = ExpressionLoweringContext {
        context,
        has_start_content,
        visiting_constants: &mut visiting_constants,
    };
    lower_expression_into_with_constants(content, expression, &mut lowering);
}

pub(super) fn lower_expression_with_expected_type_into(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    expected_type: Option<&TypeName>,
    context: &LoweringContext<'_>,
) -> bool {
    let mut visiting_constants = HashSet::new();
    let mut lowering = ExpressionLoweringContext {
        context,
        has_start_content: false,
        visiting_constants: &mut visiting_constants,
    };
    lower_expression_with_expected_type_into_with_constants(
        content,
        expression,
        expected_type,
        &mut lowering,
    )
}

pub(super) struct ExpressionLoweringContext<'a, 'ctx> {
    pub(super) context: &'a LoweringContext<'ctx>,
    pub(super) has_start_content: bool,
    pub(super) visiting_constants: &'a mut HashSet<String>,
}

impl<'ctx> ExpressionLoweringContext<'_, 'ctx> {
    pub(super) fn context(&self) -> &LoweringContext<'ctx> {
        self.context
    }
}

pub(super) fn lower_expression_into_with_constants(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    lowering: &mut ExpressionLoweringContext<'_, '_>,
) {
    let context = lowering.context;
    match expression {
        Expression::String(value) => {
            content.push(RuntimeObject::ControlCommand(ControlCommand::BeginString));
            content.push(RuntimeObject::String(value.clone()));
            content.push(RuntimeObject::ControlCommand(ControlCommand::EndString));
        }
        Expression::StringContent(string_content) => {
            content.push(RuntimeObject::ControlCommand(ControlCommand::BeginString));
            lower_content_list_into_context(content, string_content, context);
            content.push(RuntimeObject::ControlCommand(ControlCommand::EndString));
        }
        Expression::NumberInt(value) => content.push(RuntimeObject::Int(*value)),
        Expression::NumberFloat(value) => content.push(RuntimeObject::Float(value.value())),
        Expression::NumberBool(value) => content.push(RuntimeObject::Bool(*value)),
        Expression::DivertTarget(target) => {
            content.push(RuntimeObject::DivertTarget(resolve_divert_target_value(
                target,
                context.choice_labels(),
                context.global_labels(),
                context.path_mode(),
            )));
        }
        Expression::VariableReference(name) => {
            let resolved_name = resolve_runtime_variable_name(
                name,
                context.path_mode(),
                context.global_variables(),
            );
            if let Some(constant_name) =
                resolve_constant_name(name, context.path_mode(), context.constants())
            {
                let constant = context
                    .constants()
                    .get(constant_name.as_str())
                    .expect("resolved constant name must exist");
                if lowering.visiting_constants.insert(constant_name.clone()) {
                    lower_constant_expression_into(content, constant, lowering);
                    lowering.visiting_constants.remove(constant_name.as_str());
                    return;
                }
            }

            content.push(RuntimeObject::VariableReference(resolved_name));
        }
        Expression::QualifiedReference(name) => {
            if let Some(constant) = context.constants().get(name.as_str()) {
                if lowering
                    .visiting_constants
                    .insert(name.as_str().to_string())
                {
                    lower_constant_expression_into(content, constant, lowering);
                    lowering.visiting_constants.remove(name.as_str());
                    return;
                }
            }

            content.push(RuntimeObject::VariableReference(name.as_str().to_string()));
        }
        Expression::FunctionCall { name, args } => {
            lower_function_call_into(content, name, args, lowering);
        }
        Expression::QualifiedFunctionCall { name, args } => {
            lower_function_call_into(content, name.as_str(), args, lowering);
        }
        Expression::DynamicInterfaceAccess { target, member } => {
            lower_dynamic_interface_target_into(content, target, member, lowering);
        }
        Expression::DynamicInterfaceFunctionCall {
            target,
            member,
            args,
        } => {
            lower_dynamic_interface_function_call_into(content, target, member, args, lowering);
        }
        Expression::ArrayLiteral(_)
        | Expression::StructLiteral { .. }
        | Expression::DictLiteral(_) => {
            if let Some(value) = lower_value_literal(
                expression,
                None,
                context.struct_definitions(),
                context.enum_definitions(),
                context.constants(),
                context.global_variables(),
                context.choice_labels(),
                context.global_labels(),
                context.path_mode(),
            ) {
                content.push(value);
            }
        }
        Expression::FieldAccess { base, field } => {
            if !field_access_base_is_visible_value(base, context) {
                if let Some(value) = lower_enum_member_expression_value(
                    base,
                    field,
                    context.enum_definitions(),
                    context.path_mode(),
                ) {
                    content.push(value);
                    return;
                }
            }
            lower_expression_into_with_constants(content, base, lowering);
            content.push(RuntimeObject::String(field.clone()));
            content.push(RuntimeObject::NativeFunction(NativeFunction::FieldRead));
        }
        Expression::IndexAccess { base, index } => {
            lower_expression_into_with_constants(content, base, lowering);
            lower_expression_into_with_constants(content, index, lowering);
            content.push(RuntimeObject::NativeFunction(NativeFunction::IndexRead));
        }
        Expression::Binary {
            operator,
            left,
            right,
        } => {
            lower_expression_into_with_constants(content, left, lowering);
            lower_expression_into_with_constants(content, right, lowering);
            content.push(RuntimeObject::NativeFunction(
                native_function_for_binary_operator(*operator),
            ));
        }
        Expression::Unary {
            operator,
            expression,
        } => {
            lower_expression_into_with_constants(content, expression, lowering);
            content.push(RuntimeObject::NativeFunction(
                native_function_for_unary_operator(*operator),
            ));
        }
        Expression::MultipleCondition(expressions) => {
            for (index, expression) in expressions.iter().enumerate() {
                lower_expression_into_with_constants(content, expression, lowering);
                if index > 0 {
                    content.push(RuntimeObject::NativeFunction(NativeFunction::And));
                }
            }
        }
    }
}

fn lower_constant_expression_into(
    content: &mut Vec<RuntimeObject>,
    constant: &ConstantValue,
    lowering: &mut ExpressionLoweringContext<'_, '_>,
) {
    let scoped_context;
    let context = if let Some(module_name) = constant.module_name() {
        scoped_context = lowering.context.with_path_mode(ChoicePathMode::Module {
            module_name: module_name.to_string(),
        });
        &scoped_context
    } else {
        lowering.context
    };
    let mut constant_lowering = ExpressionLoweringContext {
        context,
        has_start_content: lowering.has_start_content,
        visiting_constants: &mut *lowering.visiting_constants,
    };
    if let Some(value) = lower_value_literal(
        constant.expression(),
        Some(constant.declared_type()),
        context.struct_definitions(),
        context.enum_definitions(),
        context.constants(),
        context.global_variables(),
        context.choice_labels(),
        context.global_labels(),
        context.path_mode(),
    ) {
        content.push(value);
        return;
    }

    lower_expression_into_with_constants(content, constant.expression(), &mut constant_lowering);
}

pub(super) fn lower_expression_with_expected_type_into_with_constants(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    expected_type: Option<&TypeName>,
    lowering: &mut ExpressionLoweringContext<'_, '_>,
) -> bool {
    let context = lowering.context;
    if expected_type
        .and_then(TypeName::as_interface_name)
        .is_some()
    {
        if let Some(value) = lower_value_literal(
            expression,
            expected_type,
            context.struct_definitions(),
            context.enum_definitions(),
            context.constants(),
            context.global_variables(),
            context.choice_labels(),
            context.global_labels(),
            context.path_mode(),
        ) {
            content.push(value);
            return true;
        }
    }

    if matches!(
        expression,
        Expression::ArrayLiteral(_) | Expression::StructLiteral { .. } | Expression::DictLiteral(_)
    ) {
        if let Some(value) = lower_value_literal(
            expression,
            expected_type,
            context.struct_definitions(),
            context.enum_definitions(),
            context.constants(),
            context.global_variables(),
            context.choice_labels(),
            context.global_labels(),
            context.path_mode(),
        ) {
            content.push(value);
            return true;
        }

        if lower_dynamic_composite_literal_into(content, expression, expected_type, lowering) {
            return true;
        }
    }

    let before = content.len();
    lower_expression_into_with_constants(content, expression, lowering);
    content.len() > before
}

fn field_access_base_is_visible_value(base: &Expression, context: &LoweringContext<'_>) -> bool {
    match base {
        Expression::VariableReference(name) => {
            context.path_mode().is_local_variable(name)
                || resolve_constant_name(name, context.path_mode(), context.constants()).is_some()
                || context
                    .global_variables()
                    .contains(&resolve_runtime_variable_name(
                        name,
                        context.path_mode(),
                        context.global_variables(),
                    ))
        }
        Expression::QualifiedReference(name) => {
            context.constants().contains_key(name.as_str())
                || context.global_variables().contains(name.as_str())
        }
        _ => true,
    }
}
