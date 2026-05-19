use std::collections::{BTreeMap, HashMap, HashSet};

use ink_story_json_format::Object as RuntimeObject;

use crate::parsed::{DefaultValue, Expression, StructLiteralField, TypeName};

use super::{
    context::ChoicePathMode,
    indexes::{ConstantValues, EnumDefinitions, StructDefinitions},
    path::LabelIndex,
};

pub(super) fn lower_value_literal(
    expression: &Expression,
    expected_type: Option<&TypeName>,
    struct_definitions: &StructDefinitions,
    enum_definitions: &EnumDefinitions,
    constants: &ConstantValues,
    global_variables: &HashSet<String>,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    path_mode: &ChoicePathMode,
) -> Option<RuntimeObject> {
    match (expected_type, expression) {
        (Some(TypeName::Array(element_type)), Expression::ArrayLiteral(elements)) => elements
            .iter()
            .map(|element| {
                lower_value_literal(
                    element,
                    Some(element_type),
                    struct_definitions,
                    enum_definitions,
                    constants,
                    global_variables,
                    choice_labels,
                    global_labels,
                    path_mode,
                )
            })
            .collect::<Option<Vec<_>>>()
            .map(RuntimeObject::ValueArray),
        (Some(TypeName::Struct(struct_name)), Expression::StructLiteral(fields)) => {
            lower_struct_literal(
                fields,
                struct_name,
                struct_definitions,
                enum_definitions,
                constants,
                global_variables,
                choice_labels,
                global_labels,
                path_mode,
            )
        }
        (Some(TypeName::Struct(struct_name)), Expression::EmptyCompositeLiteral) => {
            lower_struct_literal(
                &[],
                struct_name,
                struct_definitions,
                enum_definitions,
                constants,
                global_variables,
                choice_labels,
                global_labels,
                path_mode,
            )
        }
        (Some(TypeName::QualifiedStruct(struct_name)), Expression::StructLiteral(fields)) => {
            lower_struct_literal(
                fields,
                struct_name.as_str(),
                struct_definitions,
                enum_definitions,
                constants,
                global_variables,
                choice_labels,
                global_labels,
                path_mode,
            )
        }
        (Some(TypeName::QualifiedStruct(struct_name)), Expression::EmptyCompositeLiteral) => {
            lower_struct_literal(
                &[],
                struct_name.as_str(),
                struct_definitions,
                enum_definitions,
                constants,
                global_variables,
                choice_labels,
                global_labels,
                path_mode,
            )
        }
        (Some(TypeName::Interface { .. }), Expression::VariableReference(name))
            if is_module_literal_reference(name, constants, global_variables, path_mode) =>
        {
            Some(RuntimeObject::String(name.clone()))
        }
        (_, Expression::String(value)) => Some(RuntimeObject::String(value.clone())),
        (_, Expression::NumberInt(value)) => Some(RuntimeObject::Int(*value)),
        (_, Expression::NumberFloat(value)) => Some(RuntimeObject::Float(value.value())),
        (_, Expression::NumberBool(value)) => Some(RuntimeObject::Bool(*value)),
        (_, Expression::DivertTarget(target)) => Some(RuntimeObject::DivertTarget(
            resolve_divert_target_value(target, choice_labels, global_labels, path_mode),
        )),
        (None, Expression::ArrayLiteral(elements)) => elements
            .iter()
            .map(|element| {
                lower_value_literal(
                    element,
                    None,
                    struct_definitions,
                    enum_definitions,
                    constants,
                    global_variables,
                    choice_labels,
                    global_labels,
                    path_mode,
                )
            })
            .collect::<Option<Vec<_>>>()
            .map(RuntimeObject::ValueArray),
        (None, Expression::StructLiteral(fields)) => lower_dynamic_struct_literal(
            fields,
            struct_definitions,
            enum_definitions,
            constants,
            global_variables,
            choice_labels,
            global_labels,
            path_mode,
        ),
        (Some(_), Expression::ArrayLiteral(_) | Expression::StructLiteral(_))
        | (_, Expression::StringContent(_))
        | (_, Expression::VariableReference(_))
        | (_, Expression::QualifiedReference(_))
        | (_, Expression::FunctionCall { .. })
        | (_, Expression::QualifiedFunctionCall { .. })
        | (_, Expression::DynamicInterfaceAccess { .. })
        | (_, Expression::DynamicInterfaceFunctionCall { .. })
        | (_, Expression::FieldAccess { .. })
        | (_, Expression::IndexAccess { .. })
        | (_, Expression::Binary { .. })
        | (_, Expression::Unary { .. })
        | (_, Expression::MultipleCondition(_))
        | (_, Expression::DictLiteral(_))
        | (_, Expression::EmptyCompositeLiteral) => None,
    }
}

pub(super) fn runtime_default_for_type(
    type_name: &TypeName,
    struct_definitions: &StructDefinitions,
    enum_definitions: &EnumDefinitions,
    module_name: Option<&str>,
) -> Option<RuntimeObject> {
    if let Some(default_value) =
        runtime_enum_default_for_type(type_name, enum_definitions, module_name)
    {
        return Some(default_value);
    }

    let mut visiting_structs = HashSet::new();
    runtime_default_for_type_with_seen(
        type_name,
        struct_definitions,
        enum_definitions,
        &mut visiting_structs,
        module_name,
    )
}

pub(super) fn runtime_composite_placeholder_for_type(
    type_name: &TypeName,
    struct_definitions: &StructDefinitions,
    enum_definitions: &EnumDefinitions,
    module_name: Option<&str>,
) -> Option<RuntimeObject> {
    let mut visiting_structs = HashSet::new();
    runtime_composite_placeholder_for_type_with_seen(
        type_name,
        struct_definitions,
        enum_definitions,
        &mut visiting_structs,
        module_name,
    )
}

pub(super) fn struct_field_definitions_for_type<'a>(
    type_name: &TypeName,
    struct_definitions: &'a StructDefinitions,
    module_name: Option<&str>,
) -> Option<&'a Vec<(String, TypeName)>> {
    let struct_name = type_name.as_struct_name()?;
    let definition_name =
        resolve_struct_definition_name(struct_name, module_name, struct_definitions)?;
    struct_definitions.get(&definition_name)
}

fn runtime_composite_placeholder_for_type_with_seen(
    type_name: &TypeName,
    struct_definitions: &StructDefinitions,
    enum_definitions: &EnumDefinitions,
    visiting_structs: &mut HashSet<String>,
    module_name: Option<&str>,
) -> Option<RuntimeObject> {
    if let Some(default_value) =
        runtime_enum_default_for_type(type_name, enum_definitions, module_name)
    {
        return Some(default_value);
    }

    match type_name {
        TypeName::Interface { .. } => Some(RuntimeObject::String(String::new())),
        TypeName::Array(_) => Some(RuntimeObject::ValueArray(Vec::new())),
        TypeName::Dict { .. } => None,
        TypeName::Struct(struct_name) => {
            let definition_name =
                resolve_struct_definition_name(struct_name, module_name, struct_definitions)?;
            runtime_struct_placeholder_for_definition(
                definition_name,
                struct_definitions,
                enum_definitions,
                visiting_structs,
                module_name,
            )
        }
        TypeName::QualifiedStruct(struct_name) => {
            let definition_name = resolve_struct_definition_name(
                struct_name.as_str(),
                module_name,
                struct_definitions,
            )?;
            runtime_struct_placeholder_for_definition(
                definition_name,
                struct_definitions,
                enum_definitions,
                visiting_structs,
                module_name,
            )
        }
        TypeName::Primitive(_) | TypeName::Void => runtime_default_for_type_with_seen(
            type_name,
            struct_definitions,
            enum_definitions,
            visiting_structs,
            module_name,
        ),
    }
}

fn runtime_struct_placeholder_for_definition(
    definition_name: String,
    struct_definitions: &StructDefinitions,
    enum_definitions: &EnumDefinitions,
    visiting_structs: &mut HashSet<String>,
    module_name: Option<&str>,
) -> Option<RuntimeObject> {
    if !visiting_structs.insert(definition_name.clone()) {
        return None;
    }

    let Some(fields) = struct_definitions.get(&definition_name) else {
        visiting_structs.remove(&definition_name);
        return None;
    };

    let mut object_fields = BTreeMap::new();
    for (field_name, field_type) in fields {
        let Some(field_value) = runtime_composite_placeholder_for_type_with_seen(
            field_type,
            struct_definitions,
            enum_definitions,
            visiting_structs,
            module_name,
        ) else {
            visiting_structs.remove(&definition_name);
            return None;
        };
        object_fields.insert(field_name.clone(), field_value);
    }

    visiting_structs.remove(&definition_name);
    Some(RuntimeObject::ValueObject(object_fields))
}

fn runtime_default_for_type_with_seen(
    type_name: &TypeName,
    struct_definitions: &StructDefinitions,
    enum_definitions: &EnumDefinitions,
    visiting_structs: &mut HashSet<String>,
    module_name: Option<&str>,
) -> Option<RuntimeObject> {
    if let Some(default_value) =
        runtime_enum_default_for_type(type_name, enum_definitions, module_name)
    {
        return Some(default_value);
    }

    let default_value = type_name.default_value()?;
    runtime_default_value(
        &default_value,
        struct_definitions,
        enum_definitions,
        visiting_structs,
        module_name,
    )
}

fn runtime_default_value(
    default_value: &DefaultValue,
    struct_definitions: &StructDefinitions,
    enum_definitions: &EnumDefinitions,
    visiting_structs: &mut HashSet<String>,
    module_name: Option<&str>,
) -> Option<RuntimeObject> {
    match default_value {
        DefaultValue::Int(value) => Some(RuntimeObject::Int(*value)),
        DefaultValue::Float(value) => Some(RuntimeObject::Float(*value)),
        DefaultValue::Bool(value) => Some(RuntimeObject::Bool(*value)),
        DefaultValue::String(value) => Some(RuntimeObject::String(value.clone())),
        DefaultValue::Array { .. } => Some(RuntimeObject::ValueArray(Vec::new())),
        DefaultValue::Dict { .. } => None,
        DefaultValue::Struct { type_name } => {
            let definition_name =
                resolve_struct_definition_name(type_name, module_name, struct_definitions)?;
            if !visiting_structs.insert(definition_name.clone()) {
                return None;
            }

            let Some(fields) = struct_definitions.get(&definition_name) else {
                visiting_structs.remove(&definition_name);
                return None;
            };

            let mut object_fields = BTreeMap::new();
            for (field_name, field_type) in fields {
                let Some(field_value) = runtime_default_for_type_with_seen(
                    field_type,
                    struct_definitions,
                    enum_definitions,
                    visiting_structs,
                    module_name,
                ) else {
                    visiting_structs.remove(&definition_name);
                    return None;
                };
                object_fields.insert(field_name.clone(), field_value);
            }

            visiting_structs.remove(&definition_name);
            Some(RuntimeObject::ValueObject(object_fields))
        }
    }
}

fn lower_struct_literal(
    fields: &[StructLiteralField],
    struct_name: &str,
    struct_definitions: &StructDefinitions,
    enum_definitions: &EnumDefinitions,
    constants: &ConstantValues,
    global_variables: &HashSet<String>,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    path_mode: &ChoicePathMode,
) -> Option<RuntimeObject> {
    let definition_name = resolve_struct_definition_name(
        struct_name,
        path_mode.current_module_name(),
        struct_definitions,
    )?;
    let field_definitions = struct_definitions.get(&definition_name)?;
    let provided_fields = fields
        .iter()
        .map(|field| (field.name(), field.expression()))
        .collect::<HashMap<_, _>>();

    let mut object_fields = BTreeMap::new();
    for (field_name, field_type) in field_definitions {
        let field_value = if let Some(expression) = provided_fields.get(field_name.as_str()) {
            lower_value_literal(
                expression,
                Some(field_type),
                struct_definitions,
                enum_definitions,
                constants,
                global_variables,
                choice_labels,
                global_labels,
                path_mode,
            )?
        } else {
            runtime_default_for_type(
                field_type,
                struct_definitions,
                enum_definitions,
                path_mode.current_module_name(),
            )?
        };
        object_fields.insert(field_name.clone(), field_value);
    }

    Some(RuntimeObject::ValueObject(object_fields))
}

fn lower_dynamic_struct_literal(
    fields: &[StructLiteralField],
    struct_definitions: &StructDefinitions,
    enum_definitions: &EnumDefinitions,
    constants: &ConstantValues,
    global_variables: &HashSet<String>,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    path_mode: &ChoicePathMode,
) -> Option<RuntimeObject> {
    let mut object_fields = BTreeMap::new();
    for field in fields {
        object_fields.insert(
            field.name().to_string(),
            lower_value_literal(
                field.expression(),
                None,
                struct_definitions,
                enum_definitions,
                constants,
                global_variables,
                choice_labels,
                global_labels,
                path_mode,
            )?,
        );
    }
    Some(RuntimeObject::ValueObject(object_fields))
}

fn is_module_literal_reference(
    name: &str,
    constants: &ConstantValues,
    global_variables: &HashSet<String>,
    path_mode: &ChoicePathMode,
) -> bool {
    if name.contains("::")
        || path_mode.is_local_variable(name)
        || constants.contains_key(name)
        || global_variables.contains(name)
    {
        return false;
    }

    if let Some(module_name) = path_mode.current_module_name() {
        let scoped_name = format!("{module_name}::{name}");
        if constants.contains_key(&scoped_name) || global_variables.contains(&scoped_name) {
            return false;
        }
    }

    true
}

pub(super) fn lower_enum_member_expression_value(
    base: &Expression,
    member: &str,
    enum_definitions: &EnumDefinitions,
    path_mode: &ChoicePathMode,
) -> Option<RuntimeObject> {
    let enum_name = resolve_enum_definition_name_for_expression(
        base,
        path_mode.current_module_name(),
        enum_definitions,
    )?;
    enum_definitions
        .get(&enum_name)?
        .iter()
        .any(|candidate| candidate == member)
        .then(|| RuntimeObject::String(format!("{enum_name}.{member}")))
}

pub(super) fn runtime_enum_default_for_type(
    type_name: &TypeName,
    enum_definitions: &EnumDefinitions,
    module_name: Option<&str>,
) -> Option<RuntimeObject> {
    let enum_name = resolve_enum_definition_name_for_type(type_name, module_name)?;
    let first_member = enum_definitions.get(&enum_name)?.first()?;
    Some(RuntimeObject::String(format!("{enum_name}.{first_member}")))
}

fn resolve_enum_definition_name_for_type(
    type_name: &TypeName,
    module_name: Option<&str>,
) -> Option<String> {
    match type_name {
        TypeName::Struct(name) => Some(
            module_name
                .map(|module| format!("{module}::{name}"))
                .unwrap_or_else(|| name.clone()),
        ),
        TypeName::QualifiedStruct(name) => Some(name.as_str().to_string()),
        TypeName::Primitive(_)
        | TypeName::Interface { .. }
        | TypeName::Void
        | TypeName::Dict { .. }
        | TypeName::Array(_) => None,
    }
}

fn resolve_enum_definition_name_for_expression(
    expression: &Expression,
    module_name: Option<&str>,
    enum_definitions: &EnumDefinitions,
) -> Option<String> {
    match expression {
        Expression::VariableReference(name) => {
            let scoped_name = module_name
                .map(|module| format!("{module}::{name}"))
                .unwrap_or_else(|| name.clone());
            enum_definitions
                .contains_key(&scoped_name)
                .then_some(scoped_name)
        }
        Expression::QualifiedReference(name) => {
            let scoped_name = name.as_str().to_string();
            enum_definitions
                .contains_key(&scoped_name)
                .then_some(scoped_name)
        }
        _ => None,
    }
}

pub(super) fn resolve_divert_target_value(
    target: &str,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    path_mode: &ChoicePathMode,
) -> String {
    if let Some(choice_target) = choice_labels.get(target) {
        choice_target.to_string()
    } else if let Some(label_target) = path_mode
        .scoped_label_target(target, global_labels)
        .filter(|label_target| *label_target != target)
    {
        path_mode.resolve_label_target(label_target)
    } else {
        path_mode.resolve_divert_target(target)
    }
}

fn resolve_struct_definition_name(
    struct_name: &str,
    module_name: Option<&str>,
    struct_definitions: &StructDefinitions,
) -> Option<String> {
    if struct_definitions.contains_key(struct_name) {
        return Some(struct_name.to_string());
    }

    if struct_name.contains("::") {
        return None;
    }

    module_name
        .map(|module_name| format!("{module_name}::{struct_name}"))
        .filter(|qualified_name| struct_definitions.contains_key(qualified_name))
}
