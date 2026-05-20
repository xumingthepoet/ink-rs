use std::collections::HashSet;

use ink_story_json_format::{ControlCommand, NativeFunction, Object as RuntimeObject};

use crate::parsed::{
    AssignmentTarget, BinaryOperator, Expression, FlowArgument, InterfaceMemberKind,
    InterfaceMemberSignature, QualifiedName, TypeName,
};
use crate::source::SourceSpan;

use super::assignment::{
    collect_assignment_path, lower_assignment_path_update_value_into,
    lower_cached_assignment_indexes_into, push_reassignment_for_name, AssignmentUpdateValue,
};
use super::composite_literal::lower_dynamic_composite_literal_into;
use super::context::{ChoicePathMode, LoweringContext};
use super::indexes::{CallSignature, ConstantValue, ConstantValues, ExternalSignatures};
use super::path::{module_scoped_source_path_to_runtime_path, source_path_to_runtime_path};
use super::value::{
    lower_enum_member_expression_value, lower_value_literal, resolve_divert_target_value,
    struct_field_definitions_for_type,
};
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
    context: &'a LoweringContext<'ctx>,
    has_start_content: bool,
    visiting_constants: &'a mut HashSet<String>,
}

impl<'ctx> ExpressionLoweringContext<'_, 'ctx> {
    pub(super) fn context(&self) -> &LoweringContext<'ctx> {
        self.context
    }
}

fn lower_expression_into_with_constants(
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
        | Expression::StructLiteral(_)
        | Expression::DictLiteral(_)
        | Expression::EmptyCompositeLiteral => {
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

pub(super) fn dynamic_interface_knot_signature(
    expression: &Expression,
    context: &LoweringContext<'_>,
) -> Option<(String, InterfaceMemberSignature)> {
    let Expression::DynamicInterfaceAccess { target, member } = expression else {
        return None;
    };
    let interface_name = infer_lowered_expression_type(target, context)?
        .as_interface_name()?
        .to_string();
    let signature = context
        .interface_members()
        .get(&interface_name)?
        .get(member)?
        .clone();
    (signature.kind() == &InterfaceMemberKind::Knot).then_some((interface_name, signature))
}

fn dynamic_interface_function_signature(
    target: &Expression,
    member: &str,
    context: &LoweringContext<'_>,
) -> Option<(String, InterfaceMemberSignature)> {
    let interface_name = infer_lowered_expression_type(target, context)?
        .as_interface_name()?
        .to_string();
    let signature = context
        .interface_members()
        .get(&interface_name)?
        .get(member)?
        .clone();
    (signature.kind() == &InterfaceMemberKind::Function).then_some((interface_name, signature))
}

fn lower_dynamic_interface_target_into(
    content: &mut Vec<RuntimeObject>,
    target: &Expression,
    member: &str,
    lowering: &mut ExpressionLoweringContext<'_, '_>,
) {
    let interface_name = infer_lowered_expression_type(target, lowering.context)
        .and_then(|type_name| type_name.as_interface_name().map(str::to_string))
        .expect("dynamic interface target base must have interface type after analysis");
    lower_expression_into_with_constants(content, target, lowering);
    content.push(RuntimeObject::DynamicInterfaceTarget {
        interface: interface_name,
        member: member.to_string(),
    });
}

fn lower_dynamic_interface_function_call_into(
    content: &mut Vec<RuntimeObject>,
    target: &Expression,
    member: &str,
    args: &[Expression],
    lowering: &mut ExpressionLoweringContext<'_, '_>,
) {
    let (interface_name, signature) =
        dynamic_interface_function_signature(target, member, lowering.context).expect(
            "dynamic interface function must have an interface function signature after analysis",
        );

    for (index, arg) in args.iter().enumerate() {
        lower_function_arg_into_parts(content, arg, signature.arguments().get(index), lowering);
    }
    lower_expression_into_with_constants(content, target, lowering);
    content.push(RuntimeObject::DynamicInterfaceFunctionCall {
        interface: interface_name,
        member: member.to_string(),
        args: args.len(),
    });
}

fn infer_lowered_expression_type(
    expression: &Expression,
    context: &LoweringContext<'_>,
) -> Option<TypeName> {
    match expression {
        Expression::String(_) | Expression::StringContent(_) => Some(TypeName::string()),
        Expression::NumberInt(_) => Some(TypeName::int()),
        Expression::NumberFloat(_) => Some(TypeName::float()),
        Expression::NumberBool(_) => Some(TypeName::bool()),
        Expression::DivertTarget(_) | Expression::DynamicInterfaceAccess { .. } => {
            Some(TypeName::divert_target())
        }
        Expression::VariableReference(name) => visible_variable_or_constant_type(name, context),
        Expression::QualifiedReference(name) => context
            .constants()
            .get(name.as_str())
            .map(|constant| qualify_type_for_qualified_name(constant.declared_type(), name))
            .or_else(|| context.global_variable_types().get(name.as_str()).cloned()),
        Expression::FieldAccess { base, field } => {
            let base_type = infer_lowered_expression_type(base, context)?;
            let field_type = struct_field_definitions_for_type(
                &base_type,
                context.struct_definitions(),
                context.path_mode().current_module_name(),
            )?
            .iter()
            .find(|(field_name, _)| field_name == field)?
            .1
            .clone();
            Some(qualify_field_type_for_base(&field_type, &base_type))
        }
        Expression::DynamicInterfaceFunctionCall { target, member, .. } => {
            dynamic_interface_function_signature(target, member, context).map(|(_, signature)| {
                signature
                    .return_type()
                    .cloned()
                    .unwrap_or_else(TypeName::void)
            })
        }
        Expression::IndexAccess { base, .. } => {
            let base_type = infer_lowered_expression_type(base, context)?;
            base_type
                .array_element_type()
                .or_else(|| {
                    base_type
                        .dict_key_value_types()
                        .map(|(_, value_type)| value_type)
                })
                .cloned()
        }
        Expression::FunctionCall { name, .. } => callable_return_type(name, context),
        Expression::QualifiedFunctionCall { name, .. } => {
            callable_return_type(name.as_str(), context)
        }
        Expression::ArrayLiteral(_)
        | Expression::StructLiteral(_)
        | Expression::DictLiteral(_)
        | Expression::EmptyCompositeLiteral
        | Expression::Binary { .. }
        | Expression::Unary { .. }
        | Expression::MultipleCondition(_) => None,
    }
}

fn visible_variable_or_constant_type(
    name: &str,
    context: &LoweringContext<'_>,
) -> Option<TypeName> {
    if let Some(local_type) = context.path_mode().local_variable_type(name) {
        return Some(local_type.clone());
    }
    if let Some(constant_name) =
        resolve_constant_name(name, context.path_mode(), context.constants())
    {
        return context
            .constants()
            .get(constant_name.as_str())
            .map(|constant| constant.declared_type().clone());
    }

    let runtime_name =
        resolve_runtime_variable_name(name, context.path_mode(), context.global_variables());
    context.global_variable_types().get(&runtime_name).cloned()
}

fn callable_return_type(name: &str, context: &LoweringContext<'_>) -> Option<TypeName> {
    let resolved_name =
        resolve_callable_name(name, context.external_signatures(), context.path_mode());
    context
        .external_signatures()
        .get(resolved_name.as_str())
        .map(|signature| match signature {
            CallSignature::External { return_type, .. }
            | CallSignature::Ink { return_type, .. } => name
                .split_once("::")
                .map(|(module, _)| qualify_type_name_for_module(return_type, module))
                .unwrap_or_else(|| return_type.clone()),
        })
}

fn qualify_field_type_for_base(field_type: &TypeName, base_type: &TypeName) -> TypeName {
    match base_type {
        TypeName::QualifiedStruct(name) => qualify_type_name_for_module(field_type, name.module()),
        _ => field_type.clone(),
    }
}

fn qualify_type_for_qualified_name(type_name: &TypeName, name: &QualifiedName) -> TypeName {
    qualify_type_name_for_module(type_name, name.module())
}

fn qualify_type_name_for_module(type_name: &TypeName, module: &str) -> TypeName {
    match type_name {
        TypeName::Struct(name) => {
            TypeName::qualified_struct_type(qualified_type_name(module, name))
        }
        TypeName::Array(element_type) => {
            TypeName::array(qualify_type_name_for_module(element_type, module))
        }
        TypeName::Dict {
            key_type,
            value_type,
        } => TypeName::dict(*key_type, qualify_type_name_for_module(value_type, module)),
        TypeName::Primitive(_)
        | TypeName::QualifiedStruct(_)
        | TypeName::Interface { .. }
        | TypeName::Void => type_name.clone(),
    }
}

fn qualified_type_name(module: &str, name: &str) -> QualifiedName {
    let span = SourceSpan::new(None, 1, 1);
    QualifiedName::new(module, span.clone(), name, span)
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
        Expression::ArrayLiteral(_)
            | Expression::StructLiteral(_)
            | Expression::DictLiteral(_)
            | Expression::EmptyCompositeLiteral
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

fn lower_function_call_into(
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
            Some(CallSignature::Ink { .. })
        ) =>
        {
            let expected_args = match context.external_signatures().get(resolved_name.as_str()) {
                Some(CallSignature::Ink { args, .. }) => args.as_slice(),
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

fn lower_array_remove_call_into(
    content: &mut Vec<RuntimeObject>,
    args: &[Expression],
    lowering: &mut ExpressionLoweringContext<'_, '_>,
) {
    let context = lowering.context;
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
    let cached_components = lower_cached_assignment_indexes_into(content, &components, context);
    let resolved_root_name =
        resolve_runtime_variable_name(root_name, context.path_mode(), context.global_variables());

    if cached_components.is_empty() {
        content.push(RuntimeObject::VariableReference(resolved_root_name.clone()));
        let previous_has_start_content = lowering.has_start_content;
        lowering.has_start_content = false;
        lower_expression_into_with_constants(content, index_expression, lowering);
        lowering.has_start_content = previous_has_start_content;
        content.push(RuntimeObject::NativeFunction(NativeFunction::ArrayRemove));
    } else {
        lower_assignment_path_update_value_into(
            content,
            resolved_root_name.as_str(),
            &cached_components,
            0,
            AssignmentUpdateValue::ArrayRemove {
                index: index_expression,
            },
            context,
        );
    }

    push_reassignment_for_name(content, resolved_root_name.as_str(), context.path_mode());
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
    let mut lowering = ExpressionLoweringContext {
        context,
        has_start_content,
        visiting_constants,
    };
    lower_function_arg_into_parts(content, arg, expected_arg, &mut lowering);
}

fn lower_function_arg_into_parts(
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

fn native_function_for_binary_operator(operator: BinaryOperator) -> NativeFunction {
    match operator {
        BinaryOperator::And | BinaryOperator::AndSymbol => NativeFunction::And,
        BinaryOperator::Or | BinaryOperator::OrSymbol => NativeFunction::Or,
        BinaryOperator::Equals => NativeFunction::Equal,
        BinaryOperator::NotEquals => NativeFunction::NotEquals,
        BinaryOperator::GreaterThan => NativeFunction::Greater,
        BinaryOperator::LessThan => NativeFunction::Less,
        BinaryOperator::GreaterThanOrEquals => NativeFunction::GreaterThanOrEquals,
        BinaryOperator::LessThanOrEquals => NativeFunction::LessThanOrEquals,
        BinaryOperator::Has => NativeFunction::Has,
        BinaryOperator::Hasnt => NativeFunction::Hasnt,
        BinaryOperator::Add => NativeFunction::Add,
        BinaryOperator::Subtract => NativeFunction::Subtract,
        BinaryOperator::Multiply => NativeFunction::Multiply,
        BinaryOperator::Divide => NativeFunction::Divide,
        BinaryOperator::Modulo => NativeFunction::Mod,
    }
}

fn native_function_for_unary_operator(operator: crate::parsed::UnaryOperator) -> NativeFunction {
    match operator {
        crate::parsed::UnaryOperator::Negate => NativeFunction::Negate,
        crate::parsed::UnaryOperator::Not => NativeFunction::Not,
    }
}

fn builtin_native_function(name: &str) -> Option<NativeFunction> {
    match name {
        "MIN" => Some(NativeFunction::Min),
        "MAX" => Some(NativeFunction::Max),
        "POW" => Some(NativeFunction::Pow),
        "FLOOR" => Some(NativeFunction::Floor),
        "CEILING" => Some(NativeFunction::Ceiling),
        "INT" => Some(NativeFunction::Int),
        "FLOAT" => Some(NativeFunction::Float),
        "LEN" => Some(NativeFunction::Len),
        _ => None,
    }
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
