use ink_story_json_format::{NativeFunction, Object as RuntimeObject};

use crate::parsed::{Expression, TypeName};

use super::expression::{
    lower_expression_with_expected_type_into_with_constants, ExpressionLoweringContext,
};
use super::value::{runtime_default_for_type, struct_field_definitions_for_type};

pub(super) fn lower_dynamic_composite_literal_into(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    expected_type: Option<&TypeName>,
    lowering: &mut ExpressionLoweringContext<'_, '_>,
) -> bool {
    let context = lowering.context();
    match (expected_type, expression) {
        (Some(TypeName::Array(element_type)), Expression::ArrayLiteral(elements)) => {
            let Some(default_element) = runtime_default_for_type(
                element_type,
                context.struct_definitions(),
                context.path_mode().current_module_name(),
            ) else {
                return false;
            };
            let defaults = vec![default_element; elements.len()];
            let mut emitted = vec![RuntimeObject::ValueArray(defaults)];
            for (index, element) in elements.iter().enumerate() {
                let Ok(index) = i32::try_from(index) else {
                    return false;
                };
                emitted.push(RuntimeObject::Int(index));
                if !lower_expression_with_expected_type_into_with_constants(
                    &mut emitted,
                    element,
                    Some(element_type),
                    lowering,
                ) {
                    return false;
                }
                emitted.push(RuntimeObject::NativeFunction(NativeFunction::IndexWrite));
            }
            content.extend(emitted);
            true
        }
        (Some(expected_type), Expression::StructLiteral(fields))
            if matches!(
                expected_type,
                TypeName::Struct(_) | TypeName::QualifiedStruct(_)
            ) =>
        {
            let Some(default_object) = runtime_default_for_type(
                expected_type,
                context.struct_definitions(),
                context.path_mode().current_module_name(),
            ) else {
                return false;
            };
            let Some(field_definitions) = struct_field_definitions_for_type(
                expected_type,
                context.struct_definitions(),
                context.path_mode().current_module_name(),
            )
            .cloned() else {
                return false;
            };

            let mut emitted = vec![default_object];
            for field in fields {
                let Some(field_type) = field_definitions
                    .iter()
                    .find(|(field_name, _)| field_name == field.name())
                    .map(|(_, field_type)| field_type.clone())
                else {
                    return false;
                };
                emitted.push(RuntimeObject::String(field.name().to_string()));
                if !lower_expression_with_expected_type_into_with_constants(
                    &mut emitted,
                    field.expression(),
                    Some(&field_type),
                    lowering,
                ) {
                    return false;
                }
                emitted.push(RuntimeObject::NativeFunction(NativeFunction::FieldWrite));
            }
            content.extend(emitted);
            true
        }
        _ => false,
    }
}
