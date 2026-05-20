use crate::{
    parsed::{Expression, QualifiedName, TypeName},
    source::SourceSpan,
};

use super::super::{
    context::LoweringContext, indexes::CallSignature, value::struct_field_definitions_for_type,
};
use super::dynamic_interface::dynamic_interface_function_signature;
use super::name_resolution::{
    resolve_callable_name, resolve_constant_name, resolve_runtime_variable_name,
};

pub(super) fn infer_lowered_expression_type(
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
        Expression::FunctionCall { name, args } => {
            builtin_return_type_for_call(name, args, context)
                .or_else(|| callable_return_type(name, context))
        }
        Expression::QualifiedFunctionCall { name, args } => {
            builtin_return_type_for_call(name.as_str(), args, context)
                .or_else(|| callable_return_type(name.as_str(), context))
        }
        Expression::ArrayLiteral(_)
        | Expression::DictLiteral(_)
        | Expression::Binary { .. }
        | Expression::Unary { .. }
        | Expression::MultipleCondition(_) => None,
        Expression::StructLiteral { type_name, .. } => Some(type_name.clone()),
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

fn builtin_return_type_for_call(
    name: &str,
    args: &[Expression],
    context: &LoweringContext<'_>,
) -> Option<TypeName> {
    match name {
        "ARRAY_REMOVE" | "ARRAY_PUSH" | "ARRAY_INSERT" | "DICT_REMOVE" => Some(TypeName::void()),
        "LEN" | "DICT_SIZE" => Some(TypeName::int()),
        "DICT_HAS" => Some(TypeName::bool()),
        "DICT_KEYS" => {
            let dict_type = infer_lowered_expression_type(args.first()?, context)?;
            let (key_type, _) = dict_type.dict_key_value_types()?;
            Some(TypeName::array(match key_type {
                crate::parsed::DictKeyType::String => TypeName::string(),
                crate::parsed::DictKeyType::Int => TypeName::int(),
            }))
        }
        _ => None,
    }
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
