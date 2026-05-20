use ink_story_json_format::Object as RuntimeObject;

use crate::parsed::{Expression, InterfaceMemberKind, InterfaceMemberSignature};

use super::super::context::LoweringContext;
use super::calls::lower_function_arg_into_parts;
use super::types::infer_lowered_expression_type;
use super::{lower_expression_into_with_constants, ExpressionLoweringContext};

pub(in crate::lower) fn dynamic_interface_knot_signature(
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

pub(super) fn dynamic_interface_function_signature(
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

pub(super) fn lower_dynamic_interface_target_into(
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

pub(super) fn lower_dynamic_interface_function_call_into(
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
