use ink_story_json_format::{NativeFunction, Object as RuntimeObject};

use crate::parsed::{AssignmentTarget, Expression, TypeName};

use super::super::{
    assignment::{
        collect_assignment_path, lower_assignment_path_update_value_into,
        lower_cached_assignment_indexes_into, push_reassignment_for_name, AssignmentUpdateValue,
    },
    context::LoweringContext,
};
use super::name_resolution::resolve_runtime_variable_name;
use super::types::infer_lowered_expression_type;
use super::{
    lower_expression_into_with_constants, lower_expression_with_expected_type_into_with_constants,
    ExpressionLoweringContext,
};

pub(super) fn lower_array_remove_call_into(
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

pub(super) fn lower_array_push_call_into(
    content: &mut Vec<RuntimeObject>,
    args: &[Expression],
    lowering: &mut ExpressionLoweringContext<'_, '_>,
) {
    let context = lowering.context;
    let (Some(target_expression), Some(value_expression)) = (args.first(), args.get(1)) else {
        content.push(RuntimeObject::Void);
        return;
    };
    let Some(target) = AssignmentTarget::from_expression(target_expression.clone()) else {
        content.push(RuntimeObject::Void);
        return;
    };

    let expected_type = infer_array_element_type(target_expression, context);
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
        lower_expression_with_expected_type_into_with_constants(
            content,
            value_expression,
            expected_type.as_ref(),
            lowering,
        );
        content.push(RuntimeObject::NativeFunction(NativeFunction::ArrayPush));
    } else {
        lower_assignment_path_update_value_into(
            content,
            resolved_root_name.as_str(),
            &cached_components,
            0,
            AssignmentUpdateValue::ArrayPush {
                value: value_expression,
                expected_type: expected_type.as_ref(),
            },
            context,
        );
    }

    push_reassignment_for_name(content, resolved_root_name.as_str(), context.path_mode());
    content.push(RuntimeObject::Void);
}

pub(super) fn lower_array_insert_call_into(
    content: &mut Vec<RuntimeObject>,
    args: &[Expression],
    lowering: &mut ExpressionLoweringContext<'_, '_>,
) {
    let context = lowering.context;
    let (Some(target_expression), Some(index_expression), Some(value_expression)) =
        (args.first(), args.get(1), args.get(2))
    else {
        content.push(RuntimeObject::Void);
        return;
    };
    let Some(target) = AssignmentTarget::from_expression(target_expression.clone()) else {
        content.push(RuntimeObject::Void);
        return;
    };

    let expected_type = infer_array_element_type(target_expression, context);
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
        lower_expression_into_with_constants(content, index_expression, lowering);
        lower_expression_with_expected_type_into_with_constants(
            content,
            value_expression,
            expected_type.as_ref(),
            lowering,
        );
        content.push(RuntimeObject::NativeFunction(NativeFunction::ArrayInsert));
    } else {
        lower_assignment_path_update_value_into(
            content,
            resolved_root_name.as_str(),
            &cached_components,
            0,
            AssignmentUpdateValue::ArrayInsert {
                index: index_expression,
                value: value_expression,
                expected_type: expected_type.as_ref(),
            },
            context,
        );
    }

    push_reassignment_for_name(content, resolved_root_name.as_str(), context.path_mode());
    content.push(RuntimeObject::Void);
}

fn infer_array_element_type(
    expression: &Expression,
    context: &LoweringContext<'_>,
) -> Option<TypeName> {
    infer_lowered_expression_type(expression, context)
        .and_then(|type_name| type_name.array_element_type().cloned())
}

pub(super) fn lower_dict_remove_call_into(
    content: &mut Vec<RuntimeObject>,
    args: &[Expression],
    lowering: &mut ExpressionLoweringContext<'_, '_>,
) {
    let context = lowering.context;
    let (Some(target_expression), Some(key_expression)) = (args.first(), args.get(1)) else {
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
        lower_expression_into_with_constants(content, key_expression, lowering);
        lowering.has_start_content = previous_has_start_content;
        content.push(RuntimeObject::NativeFunction(NativeFunction::DictRemove));
    } else {
        lower_assignment_path_update_value_into(
            content,
            resolved_root_name.as_str(),
            &cached_components,
            0,
            AssignmentUpdateValue::DictRemove {
                key: key_expression,
            },
            context,
        );
    }

    push_reassignment_for_name(content, resolved_root_name.as_str(), context.path_mode());
    content.push(RuntimeObject::Void);
}
