use std::collections::{BTreeMap, HashMap, HashSet};

use ink_story_json_format::Object as RuntimeObject;

use crate::parsed::{DefaultValue, Expression, StructLiteralField, TypeName};

use super::indexes::StructDefinitions;

pub(super) fn lower_value_literal(
    expression: &Expression,
    expected_type: Option<&TypeName>,
    struct_definitions: &StructDefinitions,
) -> Option<RuntimeObject> {
    match (expected_type, expression) {
        (Some(TypeName::Array(element_type)), Expression::ArrayLiteral(elements)) => elements
            .iter()
            .map(|element| lower_value_literal(element, Some(element_type), struct_definitions))
            .collect::<Option<Vec<_>>>()
            .map(RuntimeObject::ValueArray),
        (Some(TypeName::Struct(struct_name)), Expression::StructLiteral(fields)) => {
            lower_struct_literal(fields, struct_name, struct_definitions)
        }
        (_, Expression::String(value)) => Some(RuntimeObject::String(value.clone())),
        (_, Expression::NumberInt(value)) => Some(RuntimeObject::Int(*value)),
        (_, Expression::NumberFloat(value)) => Some(RuntimeObject::Float(value.value())),
        (_, Expression::NumberBool(value)) => Some(RuntimeObject::Bool(*value)),
        (None, Expression::ArrayLiteral(elements)) => elements
            .iter()
            .map(|element| lower_value_literal(element, None, struct_definitions))
            .collect::<Option<Vec<_>>>()
            .map(RuntimeObject::ValueArray),
        (None, Expression::StructLiteral(fields)) => {
            lower_dynamic_struct_literal(fields, struct_definitions)
        }
        (Some(_), Expression::ArrayLiteral(_) | Expression::StructLiteral(_))
        | (_, Expression::StringContent(_))
        | (_, Expression::DivertTarget(_))
        | (_, Expression::VariableReference(_))
        | (_, Expression::FunctionCall { .. })
        | (_, Expression::FieldAccess { .. })
        | (_, Expression::IndexAccess { .. })
        | (_, Expression::Binary { .. })
        | (_, Expression::Unary { .. })
        | (_, Expression::MultipleCondition(_)) => None,
    }
}

pub(super) fn runtime_default_for_type(
    type_name: &TypeName,
    struct_definitions: &StructDefinitions,
) -> Option<RuntimeObject> {
    let mut visiting_structs = HashSet::new();
    runtime_default_for_type_with_seen(type_name, struct_definitions, &mut visiting_structs)
}

fn runtime_default_for_type_with_seen(
    type_name: &TypeName,
    struct_definitions: &StructDefinitions,
    visiting_structs: &mut HashSet<String>,
) -> Option<RuntimeObject> {
    let default_value = type_name.default_value()?;
    runtime_default_value(&default_value, struct_definitions, visiting_structs)
}

fn runtime_default_value(
    default_value: &DefaultValue,
    struct_definitions: &StructDefinitions,
    visiting_structs: &mut HashSet<String>,
) -> Option<RuntimeObject> {
    match default_value {
        DefaultValue::Int(value) => Some(RuntimeObject::Int(*value)),
        DefaultValue::Float(value) => Some(RuntimeObject::Float(*value)),
        DefaultValue::Bool(value) => Some(RuntimeObject::Bool(*value)),
        DefaultValue::String(value) => Some(RuntimeObject::String(value.clone())),
        DefaultValue::Array { .. } => Some(RuntimeObject::ValueArray(Vec::new())),
        DefaultValue::Struct { type_name } => {
            if !visiting_structs.insert(type_name.clone()) {
                return None;
            }

            let Some(fields) = struct_definitions.get(type_name) else {
                visiting_structs.remove(type_name);
                return None;
            };

            let mut object_fields = BTreeMap::new();
            for (field_name, field_type) in fields {
                let Some(field_value) = runtime_default_for_type_with_seen(
                    field_type,
                    struct_definitions,
                    visiting_structs,
                ) else {
                    visiting_structs.remove(type_name);
                    return None;
                };
                object_fields.insert(field_name.clone(), field_value);
            }

            visiting_structs.remove(type_name);
            Some(RuntimeObject::ValueObject(object_fields))
        }
    }
}

fn lower_struct_literal(
    fields: &[StructLiteralField],
    struct_name: &str,
    struct_definitions: &StructDefinitions,
) -> Option<RuntimeObject> {
    let field_definitions = struct_definitions.get(struct_name)?;
    let provided_fields = fields
        .iter()
        .map(|field| (field.name(), field.expression()))
        .collect::<HashMap<_, _>>();

    let mut object_fields = BTreeMap::new();
    for (field_name, field_type) in field_definitions {
        let field_value = if let Some(expression) = provided_fields.get(field_name.as_str()) {
            lower_value_literal(expression, Some(field_type), struct_definitions)?
        } else {
            runtime_default_for_type(field_type, struct_definitions)?
        };
        object_fields.insert(field_name.clone(), field_value);
    }

    Some(RuntimeObject::ValueObject(object_fields))
}

fn lower_dynamic_struct_literal(
    fields: &[StructLiteralField],
    struct_definitions: &StructDefinitions,
) -> Option<RuntimeObject> {
    let mut object_fields = BTreeMap::new();
    for field in fields {
        object_fields.insert(
            field.name().to_string(),
            lower_value_literal(field.expression(), None, struct_definitions)?,
        );
    }
    Some(RuntimeObject::ValueObject(object_fields))
}
