use crate::parsed::{
    BinaryOperator, DictKeyType, Expression, InterfaceMemberKind, TypeName, UnaryOperator,
};

use super::{
    context::{EnumTypeIndex, StructTypeIndex, TargetSymbolIndex, VariableScopeIndex},
    enums::resolve_enum_member_type,
    interfaces::InterfaceMemberIndex,
    structs::resolve_struct_symbol,
    target_symbols::resolve_target_symbol,
    type_names::{qualify_type_name_for_module, type_name_module},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TypeInferenceError {
    message: String,
}

impl TypeInferenceError {
    pub(super) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub(super) fn message(&self) -> &str {
        &self.message
    }
}

pub(super) fn infer_primitive_expression_type(
    expression: &Expression,
    variable_scopes: &VariableScopeIndex,
    current_module: Option<&str>,
    current_flow_path: Option<&str>,
) -> Result<TypeName, TypeInferenceError> {
    match expression {
        Expression::String(_) | Expression::StringContent(_) => Ok(TypeName::string()),
        Expression::NumberInt(_) => Ok(TypeName::int()),
        Expression::NumberFloat(_) => Ok(TypeName::float()),
        Expression::NumberBool(_) => Ok(TypeName::bool()),
        Expression::DivertTarget(_) => Ok(TypeName::divert_target()),
        Expression::VariableReference(name) => {
            infer_variable_type(name, variable_scopes, current_module, current_flow_path)
        }
        Expression::QualifiedReference(name) => {
            infer_qualified_global_type(name.as_str(), variable_scopes)
        }
        Expression::Unary {
            operator,
            expression,
        } => infer_unary_type(
            *operator,
            expression,
            variable_scopes,
            current_module,
            current_flow_path,
        ),
        Expression::Binary {
            operator,
            left,
            right,
        } => infer_binary_type(
            *operator,
            left,
            right,
            variable_scopes,
            current_module,
            current_flow_path,
        ),
        Expression::MultipleCondition(expressions) => {
            for expression in expressions {
                let expression_type = infer_primitive_expression_type(
                    expression,
                    variable_scopes,
                    current_module,
                    current_flow_path,
                )?;
                if expression_type != TypeName::bool() {
                    return Err(operator_type_error(
                        "multiple condition",
                        &[expression_type],
                    ));
                }
            }
            Ok(TypeName::bool())
        }
        Expression::FunctionCall { name, .. } => Err(TypeInferenceError::new(format!(
            "Cannot infer return type for function call '{name}' yet"
        ))),
        Expression::QualifiedFunctionCall { name, .. } => Err(TypeInferenceError::new(format!(
            "Cannot infer return type for function call '{}' yet",
            name.as_str()
        ))),
        Expression::DynamicInterfaceAccess { member, .. } => Err(TypeInferenceError::new(format!(
            "Cannot infer type for dynamic interface member '{member}' yet"
        ))),
        Expression::DynamicInterfaceFunctionCall { member, .. } => Err(TypeInferenceError::new(
            format!("Cannot infer return type for dynamic interface function '{member}' yet"),
        )),
        Expression::ArrayLiteral(_)
        | Expression::StructLiteral { .. }
        | Expression::DictLiteral(_)
        | Expression::FieldAccess { .. }
        | Expression::IndexAccess { .. } => Err(TypeInferenceError::new(
            "Expression does not have a primitive type",
        )),
    }
}

pub(super) fn infer_expression_type(
    expression: &Expression,
    variable_scopes: &VariableScopeIndex,
    struct_types: &StructTypeIndex,
    enum_types: &EnumTypeIndex,
    target_symbols: &TargetSymbolIndex,
    interface_members: &InterfaceMemberIndex,
    current_module: Option<&str>,
    current_flow_path: Option<&str>,
) -> Result<TypeName, TypeInferenceError> {
    let context = TypeInferenceContext {
        variable_scopes,
        struct_types,
        enum_types,
        target_symbols,
        interface_members,
        current_module,
        current_flow_path,
    };
    infer_expression_type_in_context(expression, &context)
}

struct TypeInferenceContext<'a> {
    variable_scopes: &'a VariableScopeIndex,
    struct_types: &'a StructTypeIndex,
    enum_types: &'a EnumTypeIndex,
    target_symbols: &'a TargetSymbolIndex,
    interface_members: &'a InterfaceMemberIndex,
    current_module: Option<&'a str>,
    current_flow_path: Option<&'a str>,
}

fn infer_expression_type_in_context(
    expression: &Expression,
    context: &TypeInferenceContext<'_>,
) -> Result<TypeName, TypeInferenceError> {
    match expression {
        Expression::FieldAccess { base, field } => {
            match infer_expression_type_in_context(base, context) {
                Ok(base_type) => infer_field_type(
                    &base_type,
                    field,
                    context.struct_types,
                    context.current_module,
                ),
                Err(error) => {
                    if let Some(member_type) = resolve_enum_member_type(
                        base,
                        field,
                        context.enum_types,
                        context.current_module,
                    ) {
                        return member_type.map_err(TypeInferenceError::new);
                    }
                    Err(error)
                }
            }
        }
        Expression::FunctionCall { name, args } => {
            typed_builtin_return_type_in_context(name, args, context).unwrap_or_else(|| {
                infer_function_return_type(
                    name,
                    context.target_symbols,
                    context.current_module,
                    context.current_flow_path,
                )
            })
        }
        Expression::QualifiedFunctionCall { name, args } => {
            typed_builtin_return_type_in_context(name.as_str(), args, context).unwrap_or_else(
                || {
                    infer_function_return_type(
                        name.as_str(),
                        context.target_symbols,
                        context.current_module,
                        context.current_flow_path,
                    )
                },
            )
        }
        Expression::DynamicInterfaceAccess { target, member } => {
            infer_dynamic_interface_target_type(target, member, context)
        }
        Expression::DynamicInterfaceFunctionCall { target, member, .. } => {
            infer_dynamic_interface_function_call_type(target, member, context)
        }
        Expression::Unary {
            operator,
            expression,
        } => infer_unary_expression_type(*operator, expression, context),
        Expression::Binary {
            operator,
            left,
            right,
        } => infer_binary_expression_type(*operator, left, right, context),
        Expression::MultipleCondition(expressions) => {
            for expression in expressions {
                let expression_type = infer_expression_type_in_context(expression, context)?;
                if expression_type != TypeName::bool() {
                    return Err(operator_type_error(
                        "multiple condition",
                        &[expression_type],
                    ));
                }
            }
            Ok(TypeName::bool())
        }
        Expression::IndexAccess { base, index } => {
            let base_type = infer_expression_type_in_context(base, context)?;
            let index_type = infer_expression_type_in_context(index, context)?;
            infer_index_type(&base_type, &index_type)
        }
        Expression::StructLiteral { type_name, .. } => Ok(type_name.clone()),
        Expression::String(_)
        | Expression::StringContent(_)
        | Expression::NumberInt(_)
        | Expression::NumberFloat(_)
        | Expression::NumberBool(_)
        | Expression::VariableReference(_)
        | Expression::QualifiedReference(_)
        | Expression::DivertTarget(_)
        | Expression::ArrayLiteral(_)
        | Expression::DictLiteral(_) => infer_primitive_expression_type(
            expression,
            context.variable_scopes,
            context.current_module,
            context.current_flow_path,
        ),
    }
}

fn infer_dynamic_interface_target_type(
    target: &Expression,
    member: &str,
    context: &TypeInferenceContext<'_>,
) -> Result<TypeName, TypeInferenceError> {
    let target_type = infer_expression_type_in_context(target, context)?;
    let Some(interface_name) = target_type.as_interface_name() else {
        return Err(TypeInferenceError::new(format!(
            "Dynamic interface target '{member}' has base type {} but expected interface",
            target_type.display_name()
        )));
    };

    let Some(signature) = context.interface_members.member(interface_name, member) else {
        return Err(TypeInferenceError::new(format!(
            "Interface '{interface_name}' does not declare member '{member}'"
        )));
    };

    if signature.kind() != &InterfaceMemberKind::Knot {
        return Err(TypeInferenceError::new(format!(
            "Interface '{interface_name}' member '{member}' is a function but dynamic target access requires a knot"
        )));
    }

    Ok(TypeName::divert_target())
}

fn infer_dynamic_interface_function_call_type(
    target: &Expression,
    member: &str,
    context: &TypeInferenceContext<'_>,
) -> Result<TypeName, TypeInferenceError> {
    let target_type = infer_expression_type_in_context(target, context)?;
    let Some(interface_name) = target_type.as_interface_name() else {
        return Err(TypeInferenceError::new(format!(
            "Dynamic interface function '{member}' has base type {} but expected interface",
            target_type.display_name()
        )));
    };

    let Some(signature) = context.interface_members.member(interface_name, member) else {
        return Err(TypeInferenceError::new(format!(
            "Interface '{interface_name}' does not declare member '{member}'"
        )));
    };

    if signature.kind() != &InterfaceMemberKind::Function {
        return Err(TypeInferenceError::new(format!(
            "Interface '{interface_name}' member '{member}' is a knot but dynamic function call requires a function"
        )));
    }

    Ok(signature
        .return_type()
        .cloned()
        .unwrap_or_else(TypeName::void))
}

pub(super) fn is_typed_builtin_function(name: &str) -> bool {
    matches!(
        name,
        "ARRAY_REMOVE"
            | "ARRAY_PUSH"
            | "ARRAY_INSERT"
            | "LEN"
            | "DICT_HAS"
            | "DICT_SIZE"
            | "DICT_REMOVE"
            | "DICT_KEYS"
    )
}

pub(super) fn typed_builtin_return_type(name: &str) -> Option<TypeName> {
    match name {
        "ARRAY_REMOVE" | "ARRAY_PUSH" | "ARRAY_INSERT" => Some(TypeName::void()),
        "LEN" => Some(TypeName::int()),
        "DICT_HAS" => Some(TypeName::bool()),
        "DICT_SIZE" => Some(TypeName::int()),
        "DICT_REMOVE" => Some(TypeName::void()),
        _ => None,
    }
}

fn typed_builtin_return_type_in_context(
    name: &str,
    args: &[Expression],
    context: &TypeInferenceContext<'_>,
) -> Option<Result<TypeName, TypeInferenceError>> {
    if name == "DICT_KEYS" {
        return Some(infer_dict_keys_return_type(args, context));
    }

    typed_builtin_return_type(name).map(Ok)
}

fn infer_dict_keys_return_type(
    args: &[Expression],
    context: &TypeInferenceContext<'_>,
) -> Result<TypeName, TypeInferenceError> {
    if args.len() != 1 {
        return Err(TypeInferenceError::new(format!(
            "Builtin 'DICT_KEYS' expects 1 argument but got {}",
            args.len()
        )));
    }

    let dict_type = infer_expression_type_in_context(&args[0], context)?;
    let Some((key_type, _)) = dict_type.dict_key_value_types() else {
        return Err(TypeInferenceError::new(format!(
            "First argument for builtin 'DICT_KEYS' has type {} but expected Dict",
            dict_type.display_name()
        )));
    };

    Ok(TypeName::array(match key_type {
        DictKeyType::String => TypeName::string(),
        DictKeyType::Int => TypeName::int(),
    }))
}

fn infer_variable_type(
    name: &str,
    variable_scopes: &VariableScopeIndex,
    current_module: Option<&str>,
    current_flow_path: Option<&str>,
) -> Result<TypeName, TypeInferenceError> {
    match variable_scopes.visible_variable_declared_type(name, current_module, current_flow_path) {
        Some(Some(type_name)) => Ok(type_name.clone()),
        Some(None) => Err(TypeInferenceError::new(format!(
            "Variable '{name}' has no declared type"
        ))),
        None => Err(TypeInferenceError::new(format!(
            "Unknown variable '{name}'"
        ))),
    }
}

fn infer_qualified_global_type(
    name: &str,
    variable_scopes: &VariableScopeIndex,
) -> Result<TypeName, TypeInferenceError> {
    let module = name.split_once("::").map(|(module, _)| module);
    match variable_scopes
        .qualified_constant_declared_type(name)
        .or_else(|| variable_scopes.qualified_global_variable_declared_type(name))
    {
        Some(Some(type_name)) => Ok(module
            .map(|module| qualify_type_name_for_module(type_name, module))
            .unwrap_or_else(|| type_name.clone())),
        Some(None) => Err(TypeInferenceError::new(format!(
            "Qualified global '{name}' has no declared type"
        ))),
        None => Err(TypeInferenceError::new(format!(
            "Unknown qualified global '{name}'"
        ))),
    }
}

fn infer_field_type(
    base_type: &TypeName,
    field: &str,
    struct_types: &StructTypeIndex,
    current_module: Option<&str>,
) -> Result<TypeName, TypeInferenceError> {
    let Some(struct_name) = base_type.as_struct_name() else {
        return Err(TypeInferenceError::new(format!(
            "Cannot access field '{field}' on non-struct type {}",
            base_type.display_name()
        )));
    };

    let Some(symbol) = resolve_struct_symbol(struct_types, struct_name, current_module) else {
        return Err(TypeInferenceError::new(format!(
            "Unknown struct type '{struct_name}' for field access"
        )));
    };

    let field_type = symbol.fields().get(field).cloned().ok_or_else(|| {
        TypeInferenceError::new(format!("Unknown field '{field}' on struct '{struct_name}'"))
    })?;
    Ok(type_name_module(base_type)
        .map(|module| qualify_type_name_for_module(&field_type, module))
        .unwrap_or(field_type))
}

fn infer_function_return_type(
    name: &str,
    target_symbols: &TargetSymbolIndex,
    current_module: Option<&str>,
    current_flow_path: Option<&str>,
) -> Result<TypeName, TypeInferenceError> {
    let qualified_module = name.split_once("::").map(|(module, _)| module);
    let Some(symbol) =
        resolve_target_symbol(name, current_module, current_flow_path, target_symbols)
    else {
        return Err(TypeInferenceError::new(format!(
            "Unknown function '{name}'"
        )));
    };

    if !symbol.is_function() {
        return Err(TypeInferenceError::new(format!(
            "'{name}' is not a function"
        )));
    }

    Ok(qualified_module
        .map(|module| qualify_type_name_for_module(symbol.return_type(), module))
        .unwrap_or_else(|| symbol.return_type().clone()))
}

fn infer_index_type(
    base_type: &TypeName,
    index_type: &TypeName,
) -> Result<TypeName, TypeInferenceError> {
    if let Some(element_type) = base_type.array_element_type() {
        expect_index_type(index_type, &TypeName::int())?;
        return Ok(element_type.clone());
    }

    if let Some((key_type, value_type)) = base_type.dict_key_value_types() {
        let expected_index_type = dict_key_expression_type(key_type);
        expect_index_type(index_type, &expected_index_type)?;
        return Ok(value_type.clone());
    }

    Err(TypeInferenceError::new(format!(
        "Cannot index non-array/non-Dict type {}",
        base_type.display_name()
    )))
}

fn expect_index_type(
    index_type: &TypeName,
    expected_type: &TypeName,
) -> Result<(), TypeInferenceError> {
    if index_type == expected_type {
        Ok(())
    } else {
        Err(TypeInferenceError::new(format!(
            "Index expression has type {} but expected {}",
            index_type.display_name(),
            expected_type.display_name()
        )))
    }
}

fn dict_key_expression_type(key_type: DictKeyType) -> TypeName {
    match key_type {
        DictKeyType::String => TypeName::string(),
        DictKeyType::Int => TypeName::int(),
    }
}

fn infer_unary_type(
    operator: UnaryOperator,
    expression: &Expression,
    variable_scopes: &VariableScopeIndex,
    current_module: Option<&str>,
    current_flow_path: Option<&str>,
) -> Result<TypeName, TypeInferenceError> {
    let expression_type = infer_primitive_expression_type(
        expression,
        variable_scopes,
        current_module,
        current_flow_path,
    )?;
    match operator {
        UnaryOperator::Negate if expression_type == TypeName::int() => Ok(TypeName::int()),
        UnaryOperator::Negate if expression_type == TypeName::float() => Ok(TypeName::float()),
        UnaryOperator::Not if expression_type == TypeName::bool() => Ok(TypeName::bool()),
        _ => Err(operator_type_error(
            unary_operator_text(operator),
            &[expression_type],
        )),
    }
}

fn infer_unary_expression_type(
    operator: UnaryOperator,
    expression: &Expression,
    context: &TypeInferenceContext<'_>,
) -> Result<TypeName, TypeInferenceError> {
    let expression_type = infer_expression_type_in_context(expression, context)?;
    match operator {
        UnaryOperator::Negate if expression_type == TypeName::int() => Ok(TypeName::int()),
        UnaryOperator::Negate if expression_type == TypeName::float() => Ok(TypeName::float()),
        UnaryOperator::Not if expression_type == TypeName::bool() => Ok(TypeName::bool()),
        _ => Err(operator_type_error(
            unary_operator_text(operator),
            &[expression_type],
        )),
    }
}

fn infer_binary_type(
    operator: BinaryOperator,
    left: &Expression,
    right: &Expression,
    variable_scopes: &VariableScopeIndex,
    current_module: Option<&str>,
    current_flow_path: Option<&str>,
) -> Result<TypeName, TypeInferenceError> {
    let left_type =
        infer_primitive_expression_type(left, variable_scopes, current_module, current_flow_path)?;
    let right_type =
        infer_primitive_expression_type(right, variable_scopes, current_module, current_flow_path)?;

    match operator {
        BinaryOperator::Add => infer_add_type(left_type, right_type),
        BinaryOperator::Subtract | BinaryOperator::Multiply | BinaryOperator::Divide => {
            infer_numeric_type(binary_operator_text(operator), left_type, right_type)
        }
        BinaryOperator::Modulo => {
            if left_type == TypeName::int() && right_type == TypeName::int() {
                Ok(TypeName::int())
            } else {
                Err(operator_type_error(
                    binary_operator_text(operator),
                    &[left_type, right_type],
                ))
            }
        }
        BinaryOperator::And
        | BinaryOperator::AndSymbol
        | BinaryOperator::Or
        | BinaryOperator::OrSymbol => {
            if left_type == TypeName::bool() && right_type == TypeName::bool() {
                Ok(TypeName::bool())
            } else {
                Err(operator_type_error(
                    binary_operator_text(operator),
                    &[left_type, right_type],
                ))
            }
        }
        BinaryOperator::Equals | BinaryOperator::NotEquals => {
            if left_type == right_type && supports_equality(&left_type) {
                Ok(TypeName::bool())
            } else {
                Err(operator_type_error(
                    binary_operator_text(operator),
                    &[left_type, right_type],
                ))
            }
        }
        BinaryOperator::GreaterThan
        | BinaryOperator::LessThan
        | BinaryOperator::GreaterThanOrEquals
        | BinaryOperator::LessThanOrEquals => {
            if is_numeric_type(&left_type) && left_type == right_type {
                Ok(TypeName::bool())
            } else {
                Err(operator_type_error(
                    binary_operator_text(operator),
                    &[left_type, right_type],
                ))
            }
        }
        BinaryOperator::Has | BinaryOperator::Hasnt => Err(operator_type_error(
            binary_operator_text(operator),
            &[left_type, right_type],
        )),
    }
}

fn infer_binary_expression_type(
    operator: BinaryOperator,
    left: &Expression,
    right: &Expression,
    context: &TypeInferenceContext<'_>,
) -> Result<TypeName, TypeInferenceError> {
    let left_type = infer_expression_type_in_context(left, context)?;
    let right_type = infer_expression_type_in_context(right, context)?;

    infer_binary_operator_type(operator, left_type, right_type)
}

pub(super) fn infer_binary_operator_type(
    operator: BinaryOperator,
    left_type: TypeName,
    right_type: TypeName,
) -> Result<TypeName, TypeInferenceError> {
    match operator {
        BinaryOperator::Add => infer_add_type(left_type, right_type),
        BinaryOperator::Subtract | BinaryOperator::Multiply | BinaryOperator::Divide => {
            infer_numeric_type(binary_operator_text(operator), left_type, right_type)
        }
        BinaryOperator::Modulo => {
            if left_type == TypeName::int() && right_type == TypeName::int() {
                Ok(TypeName::int())
            } else {
                Err(operator_type_error(
                    binary_operator_text(operator),
                    &[left_type, right_type],
                ))
            }
        }
        BinaryOperator::And
        | BinaryOperator::AndSymbol
        | BinaryOperator::Or
        | BinaryOperator::OrSymbol => {
            if left_type == TypeName::bool() && right_type == TypeName::bool() {
                Ok(TypeName::bool())
            } else {
                Err(operator_type_error(
                    binary_operator_text(operator),
                    &[left_type, right_type],
                ))
            }
        }
        BinaryOperator::Equals | BinaryOperator::NotEquals => {
            if left_type == right_type && supports_equality(&left_type) {
                Ok(TypeName::bool())
            } else {
                Err(operator_type_error(
                    binary_operator_text(operator),
                    &[left_type, right_type],
                ))
            }
        }
        BinaryOperator::GreaterThan
        | BinaryOperator::LessThan
        | BinaryOperator::GreaterThanOrEquals
        | BinaryOperator::LessThanOrEquals => {
            if is_numeric_type(&left_type) && left_type == right_type {
                Ok(TypeName::bool())
            } else {
                Err(operator_type_error(
                    binary_operator_text(operator),
                    &[left_type, right_type],
                ))
            }
        }
        BinaryOperator::Has | BinaryOperator::Hasnt => Err(operator_type_error(
            binary_operator_text(operator),
            &[left_type, right_type],
        )),
    }
}

fn binary_operator_text(operator: BinaryOperator) -> &'static str {
    match operator {
        BinaryOperator::And => "and",
        BinaryOperator::AndSymbol => "&&",
        BinaryOperator::Or => "or",
        BinaryOperator::OrSymbol => "||",
        BinaryOperator::Equals => "==",
        BinaryOperator::NotEquals => "!=",
        BinaryOperator::GreaterThan => ">",
        BinaryOperator::LessThan => "<",
        BinaryOperator::GreaterThanOrEquals => ">=",
        BinaryOperator::LessThanOrEquals => "<=",
        BinaryOperator::Has => "?",
        BinaryOperator::Hasnt => "!?",
        BinaryOperator::Add => "+",
        BinaryOperator::Subtract => "-",
        BinaryOperator::Multiply => "*",
        BinaryOperator::Divide => "/",
        BinaryOperator::Modulo => "%",
    }
}

fn unary_operator_text(operator: UnaryOperator) -> &'static str {
    match operator {
        UnaryOperator::Negate => "-",
        UnaryOperator::Not => "not",
    }
}

fn infer_add_type(
    left_type: TypeName,
    right_type: TypeName,
) -> Result<TypeName, TypeInferenceError> {
    if (left_type == TypeName::int() && right_type == TypeName::int())
        || (left_type == TypeName::float() && right_type == TypeName::float())
        || (left_type == TypeName::string() && right_type == TypeName::string())
    {
        Ok(left_type)
    } else {
        Err(operator_type_error("+", &[left_type, right_type]))
    }
}

fn infer_numeric_type(
    operator: &str,
    left_type: TypeName,
    right_type: TypeName,
) -> Result<TypeName, TypeInferenceError> {
    if is_numeric_type(&left_type) && left_type == right_type {
        Ok(left_type)
    } else {
        Err(operator_type_error(operator, &[left_type, right_type]))
    }
}

fn is_numeric_type(type_name: &TypeName) -> bool {
    type_name == &TypeName::int() || type_name == &TypeName::float()
}

fn supports_equality(type_name: &TypeName) -> bool {
    match type_name {
        TypeName::Primitive(_)
        | TypeName::Struct(_)
        | TypeName::QualifiedStruct(_)
        | TypeName::Interface { .. } => true,
        TypeName::Array(element_type) => supports_equality(element_type),
        TypeName::Dict { value_type, .. } => supports_equality(value_type),
        TypeName::Void => false,
    }
}

fn operator_type_error(operator: &str, types: &[TypeName]) -> TypeInferenceError {
    let type_list = types
        .iter()
        .map(TypeName::display_name)
        .collect::<Vec<_>>()
        .join(" and ");
    TypeInferenceError::new(format!(
        "Operator '{operator}' is not defined for type{} {type_list}",
        if types.len() == 1 { "" } else { "s" }
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        parsed::{FloatLiteral, QualifiedName},
        source::SourceSpan,
    };

    use super::{
        super::{
            enums::build_enum_type_index, interfaces::build_interface_member_index,
            structs::build_struct_type_index, target_symbols::build_target_symbol_index,
            test_support::parse_story, variables::build_variable_scope_index,
        },
        *,
    };

    fn empty_scopes() -> VariableScopeIndex {
        VariableScopeIndex::default()
    }

    fn binary(operator: BinaryOperator, left: Expression, right: Expression) -> Expression {
        Expression::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn unary(operator: UnaryOperator, expression: Expression) -> Expression {
        Expression::Unary {
            operator,
            expression: Box::new(expression),
        }
    }

    fn variable(name: &str) -> Expression {
        Expression::VariableReference(name.to_string())
    }

    fn qualified_reference(module: &str, symbol: &str) -> Expression {
        Expression::QualifiedReference(QualifiedName::new(
            module,
            SourceSpan::new(None, 1, 1),
            symbol,
            SourceSpan::new(None, 1, 1),
        ))
    }

    fn qualified_type(module: &str, symbol: &str) -> TypeName {
        TypeName::qualified_struct_type(QualifiedName::new(
            module,
            SourceSpan::new(None, 1, 1),
            symbol,
            SourceSpan::new(None, 1, 1),
        ))
    }

    fn dynamic_interface_access(target: Expression, member: &str) -> Expression {
        Expression::DynamicInterfaceAccess {
            target: Box::new(target),
            member: member.to_string(),
        }
    }

    fn dynamic_interface_call(
        target: Expression,
        member: &str,
        args: Vec<Expression>,
    ) -> Expression {
        Expression::DynamicInterfaceFunctionCall {
            target: Box::new(target),
            member: member.to_string(),
            args,
        }
    }

    #[test]
    fn infers_literal_and_variable_primitive_types() {
        let story = parse_story(
            "VAR score: int = 0\n\
             == knot(flag: bool) ==\n\
             -> DONE",
        );
        let scopes = build_variable_scope_index(&story);

        assert_eq!(
            infer_primitive_expression_type(
                &Expression::VariableReference("score".to_string()),
                &scopes,
                None,
                Some("knot")
            ),
            Ok(TypeName::int())
        );
        assert_eq!(
            infer_primitive_expression_type(
                &Expression::VariableReference("flag".to_string()),
                &scopes,
                None,
                Some("knot")
            ),
            Ok(TypeName::bool())
        );
        assert_eq!(
            infer_primitive_expression_type(
                &Expression::String("text".to_string()),
                &scopes,
                None,
                None
            ),
            Ok(TypeName::string())
        );
    }

    #[test]
    fn infers_qualified_constant_and_variable_types() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             -> DONE\n\
             === module items ===\n\
             CONST MAX_SCORE: int = 3\n\
             VAR score: int = 0\n\
             == helper ==\n\
             -> DONE",
        );
        let scopes = build_variable_scope_index(&story);

        assert_eq!(
            infer_primitive_expression_type(
                &qualified_reference("items", "MAX_SCORE"),
                &scopes,
                Some("game"),
                Some("main")
            ),
            Ok(TypeName::int())
        );
        assert_eq!(
            infer_primitive_expression_type(
                &qualified_reference("items", "score"),
                &scopes,
                Some("game"),
                Some("main")
            ),
            Ok(TypeName::int())
        );
    }

    #[test]
    fn infers_qualified_global_named_types_in_declaring_module_scope() {
        let story = parse_story(
            "=== module game ===\n\
             FROM data IMPORT actor, DEFAULT_STATE, State\n\
             == main ==\n\
             -> DONE\n\
             === module data ===\n\
             ENUM State { Idle Busy }\n\
             STRUCT Actor {\n\
             state: State\n\
             history: State[]\n\
             }\n\
             VAR actor: Actor\n\
             CONST DEFAULT_STATE: State = State.Idle\n\
             == helper ==\n\
             -> DONE",
        );
        let scopes = build_variable_scope_index(&story);
        let structs = build_struct_type_index(&story);
        let enums = build_enum_type_index(&story);
        let targets = build_target_symbol_index(&story);
        let interface_members = build_interface_member_index(&story);
        let actor_state = Expression::FieldAccess {
            base: Box::new(qualified_reference("data", "actor")),
            field: "state".to_string(),
        };
        let actor_history = Expression::FieldAccess {
            base: Box::new(qualified_reference("data", "actor")),
            field: "history".to_string(),
        };

        assert_eq!(
            infer_expression_type(
                &qualified_reference("data", "actor"),
                &scopes,
                &structs,
                &enums,
                &targets,
                &interface_members,
                Some("game"),
                Some("main")
            ),
            Ok(qualified_type("data", "Actor"))
        );
        assert_eq!(
            infer_expression_type(
                &actor_state,
                &scopes,
                &structs,
                &enums,
                &targets,
                &interface_members,
                Some("game"),
                Some("main")
            ),
            Ok(qualified_type("data", "State"))
        );
        assert_eq!(
            infer_expression_type(
                &actor_history,
                &scopes,
                &structs,
                &enums,
                &targets,
                &interface_members,
                Some("game"),
                Some("main")
            ),
            Ok(TypeName::array(qualified_type("data", "State")))
        );
        assert_eq!(
            infer_expression_type(
                &qualified_reference("data", "DEFAULT_STATE"),
                &scopes,
                &structs,
                &enums,
                &targets,
                &interface_members,
                Some("game"),
                Some("main")
            ),
            Ok(qualified_type("data", "State"))
        );
    }

    #[test]
    fn infers_numeric_and_unary_operator_types_without_widening() {
        let scopes = empty_scopes();
        let int_add = binary(
            BinaryOperator::Add,
            Expression::NumberInt(1),
            Expression::NumberInt(2),
        );
        let float_negate = unary(
            UnaryOperator::Negate,
            Expression::NumberFloat(FloatLiteral::new(2.0)),
        );

        assert_eq!(
            infer_primitive_expression_type(&int_add, &scopes, None, None),
            Ok(TypeName::int())
        );
        assert_eq!(
            infer_primitive_expression_type(&float_negate, &scopes, None, None),
            Ok(TypeName::float())
        );
    }

    #[test]
    fn rejects_implicit_numeric_conversion() {
        let scopes = empty_scopes();
        let mixed_add = binary(
            BinaryOperator::Add,
            Expression::NumberInt(1),
            Expression::NumberFloat(FloatLiteral::new(2.0)),
        );

        let error = infer_primitive_expression_type(&mixed_add, &scopes, None, None).unwrap_err();

        assert_eq!(
            error.message(),
            "Operator '+' is not defined for types int and float"
        );
    }

    #[test]
    fn infers_string_concatenation_and_rejects_mixed_string_addition() {
        let scopes = empty_scopes();
        let concat = binary(
            BinaryOperator::Add,
            Expression::String("a".to_string()),
            Expression::String("b".to_string()),
        );
        let mixed = binary(
            BinaryOperator::Add,
            Expression::String("a".to_string()),
            Expression::NumberInt(1),
        );

        assert_eq!(
            infer_primitive_expression_type(&concat, &scopes, None, None),
            Ok(TypeName::string())
        );
        assert_eq!(
            infer_primitive_expression_type(&mixed, &scopes, None, None)
                .unwrap_err()
                .message(),
            "Operator '+' is not defined for types string and int"
        );
    }

    #[test]
    fn infers_boolean_operator_types() {
        let scopes = empty_scopes();
        let and_expression = binary(
            BinaryOperator::AndSymbol,
            Expression::NumberBool(true),
            Expression::NumberBool(false),
        );
        let not_expression = unary(UnaryOperator::Not, Expression::NumberBool(true));

        assert_eq!(
            infer_primitive_expression_type(&and_expression, &scopes, None, None),
            Ok(TypeName::bool())
        );
        assert_eq!(
            infer_primitive_expression_type(&not_expression, &scopes, None, None),
            Ok(TypeName::bool())
        );
    }

    #[test]
    fn rejects_boolean_operators_for_divert_targets() {
        let story = parse_story(
            "VAR next: -> = -> knot\n\
             -> DONE\n\
             == knot ==\n\
             -> DONE",
        );
        let scopes = build_variable_scope_index(&story);
        let structs = build_struct_type_index(&story);
        let enums = build_enum_type_index(&story);
        let targets = build_target_symbol_index(&story);
        let interface_members = build_interface_member_index(&story);
        let cases = [
            (
                unary(UnaryOperator::Not, variable("next")),
                "Operator 'not' is not defined for type ->",
            ),
            (
                binary(
                    BinaryOperator::AndSymbol,
                    variable("next"),
                    Expression::NumberBool(true),
                ),
                "Operator '&&' is not defined for types -> and bool",
            ),
        ];

        for (expression, expected_message) in cases {
            let error = infer_expression_type(
                &expression,
                &scopes,
                &structs,
                &enums,
                &targets,
                &interface_members,
                None,
                None,
            )
            .unwrap_err();

            assert_eq!(error.message(), expected_message);
        }
    }

    #[test]
    fn infers_equality_for_primitive_array_struct_and_nested_types() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR score: int = 1\n\
             VAR other_score: int = 2\n\
             VAR first_target: -> = -> knot\n\
             VAR second_target: -> = -> other\n\
             VAR source_player: Player = %Player{ hp: 10 }\n\
             VAR other_player: Player = %Player{ hp: 20 }\n\
             VAR scores: int[] = [score]\n\
             VAR other_scores: int[] = [other_score]\n\
             VAR nested_scores: int[][] = [scores]\n\
             VAR other_nested_scores: int[][] = [other_scores]\n\
             -> DONE\n\
             == knot ==\n\
             -> DONE\n\
             == other ==\n\
             -> DONE",
        );
        let scopes = build_variable_scope_index(&story);
        let structs = build_struct_type_index(&story);
        let enums = build_enum_type_index(&story);
        let targets = build_target_symbol_index(&story);
        let interface_members = build_interface_member_index(&story);
        let expressions = [
            binary(
                BinaryOperator::Equals,
                variable("score"),
                variable("other_score"),
            ),
            binary(
                BinaryOperator::Equals,
                variable("first_target"),
                variable("second_target"),
            ),
            binary(
                BinaryOperator::NotEquals,
                variable("first_target"),
                variable("second_target"),
            ),
            binary(
                BinaryOperator::Equals,
                variable("scores"),
                variable("other_scores"),
            ),
            binary(
                BinaryOperator::NotEquals,
                variable("scores"),
                variable("other_scores"),
            ),
            binary(
                BinaryOperator::Equals,
                variable("source_player"),
                variable("other_player"),
            ),
            binary(
                BinaryOperator::Equals,
                variable("nested_scores"),
                variable("other_nested_scores"),
            ),
        ];

        for expression in expressions {
            assert_eq!(
                infer_expression_type(
                    &expression,
                    &scopes,
                    &structs,
                    &enums,
                    &targets,
                    &interface_members,
                    None,
                    None,
                ),
                Ok(TypeName::bool())
            );
        }
    }

    #[test]
    fn infers_enum_member_and_equality_types() {
        let story = parse_story(
            "ENUM State { Idle Busy }\n\
             VAR state: State = State.Idle\n\
             -> DONE",
        );
        let scopes = build_variable_scope_index(&story);
        let structs = build_struct_type_index(&story);
        let enums = build_enum_type_index(&story);
        let targets = build_target_symbol_index(&story);
        let interface_members = build_interface_member_index(&story);
        let idle = Expression::FieldAccess {
            base: Box::new(variable("State")),
            field: "Idle".to_string(),
        };
        let busy = Expression::FieldAccess {
            base: Box::new(variable("State")),
            field: "Busy".to_string(),
        };

        assert_eq!(
            infer_expression_type(
                &idle,
                &scopes,
                &structs,
                &enums,
                &targets,
                &interface_members,
                None,
                None,
            ),
            Ok(TypeName::struct_type("State"))
        );
        assert_eq!(
            infer_expression_type(
                &binary(BinaryOperator::Equals, idle, busy),
                &scopes,
                &structs,
                &enums,
                &targets,
                &interface_members,
                None,
                None
            ),
            Ok(TypeName::bool())
        );
    }

    #[test]
    fn rejects_unknown_enum_members() {
        let story = parse_story("ENUM State { Idle Busy }\n-> DONE");
        let scopes = build_variable_scope_index(&story);
        let structs = build_struct_type_index(&story);
        let enums = build_enum_type_index(&story);
        let targets = build_target_symbol_index(&story);
        let interface_members = build_interface_member_index(&story);
        let missing = Expression::FieldAccess {
            base: Box::new(variable("State")),
            field: "Missing".to_string(),
        };

        let error = infer_expression_type(
            &missing,
            &scopes,
            &structs,
            &enums,
            &targets,
            &interface_members,
            None,
            None,
        )
        .unwrap_err();

        assert_eq!(error.message(), "Unknown member 'Missing' in enum 'State'");
    }

    #[test]
    fn infers_dynamic_interface_target_access_type() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target(amount: int) ==\n\
             == function score() => int ==\n\
             === module game ===\n\
             STRUCT RouteState {\n\
             current: interface<IItem>\n\
             routes: interface<IItem>[]\n\
             }\n\
             VAR route: interface<IItem>\n\
             VAR state: RouteState\n\
             VAR routes: interface<IItem>[]\n\
             == main ==\n\
             -> END",
        );
        let scopes = build_variable_scope_index(&story);
        let structs = build_struct_type_index(&story);
        let enums = build_enum_type_index(&story);
        let targets = build_target_symbol_index(&story);
        let interface_members = build_interface_member_index(&story);
        let cases = [
            dynamic_interface_access(variable("route"), "target"),
            dynamic_interface_access(
                Expression::FieldAccess {
                    base: Box::new(variable("state")),
                    field: "current".to_string(),
                },
                "target",
            ),
            dynamic_interface_access(
                Expression::IndexAccess {
                    base: Box::new(variable("routes")),
                    index: Box::new(Expression::NumberInt(0)),
                },
                "target",
            ),
        ];

        for expression in cases {
            assert_eq!(
                infer_expression_type(
                    &expression,
                    &scopes,
                    &structs,
                    &enums,
                    &targets,
                    &interface_members,
                    Some("game"),
                    Some("main")
                ),
                Ok(TypeName::divert_target())
            );
        }
    }

    #[test]
    fn rejects_invalid_dynamic_interface_target_access() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target(amount: int) ==\n\
             == function score() => int ==\n\
             === module game ===\n\
             VAR route: interface<IItem>\n\
             VAR label: string = \"x\"\n\
             == main ==\n\
             -> END",
        );
        let scopes = build_variable_scope_index(&story);
        let structs = build_struct_type_index(&story);
        let enums = build_enum_type_index(&story);
        let targets = build_target_symbol_index(&story);
        let interface_members = build_interface_member_index(&story);
        let cases = [
            (
                dynamic_interface_access(variable("label"), "target"),
                "Dynamic interface target 'target' has base type string but expected interface",
            ),
            (
                dynamic_interface_access(variable("route"), "missing"),
                "Interface 'IItem' does not declare member 'missing'",
            ),
            (
                dynamic_interface_access(variable("route"), "score"),
                "Interface 'IItem' member 'score' is a function but dynamic target access requires a knot",
            ),
        ];

        for (expression, expected_message) in cases {
            let error = infer_expression_type(
                &expression,
                &scopes,
                &structs,
                &enums,
                &targets,
                &interface_members,
                Some("game"),
                Some("main"),
            )
            .unwrap_err();

            assert_eq!(error.message(), expected_message);
        }
    }

    #[test]
    fn infers_dynamic_interface_function_call_return_type() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target(amount: int) ==\n\
             == function score(amount: int) => int ==\n\
             === module game ===\n\
             STRUCT RouteState {\n\
             current: interface<IItem>\n\
             routes: interface<IItem>[]\n\
             }\n\
             VAR route: interface<IItem>\n\
             VAR state: RouteState\n\
             VAR routes: interface<IItem>[]\n\
             == main ==\n\
             -> END",
        );
        let scopes = build_variable_scope_index(&story);
        let structs = build_struct_type_index(&story);
        let enums = build_enum_type_index(&story);
        let targets = build_target_symbol_index(&story);
        let interface_members = build_interface_member_index(&story);
        let cases = [
            dynamic_interface_call(variable("route"), "score", vec![Expression::NumberInt(1)]),
            dynamic_interface_call(
                Expression::FieldAccess {
                    base: Box::new(variable("state")),
                    field: "current".to_string(),
                },
                "score",
                vec![Expression::NumberInt(1)],
            ),
            dynamic_interface_call(
                Expression::IndexAccess {
                    base: Box::new(variable("routes")),
                    index: Box::new(Expression::NumberInt(0)),
                },
                "score",
                vec![Expression::NumberInt(1)],
            ),
        ];

        for expression in cases {
            assert_eq!(
                infer_expression_type(
                    &expression,
                    &scopes,
                    &structs,
                    &enums,
                    &targets,
                    &interface_members,
                    Some("game"),
                    Some("main")
                ),
                Ok(TypeName::int())
            );
        }
    }

    #[test]
    fn rejects_invalid_dynamic_interface_function_calls() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target(amount: int) ==\n\
             == function score(amount: int) => int ==\n\
             === module game ===\n\
             VAR route: interface<IItem>\n\
             VAR label: string = \"x\"\n\
             == main ==\n\
             -> END",
        );
        let scopes = build_variable_scope_index(&story);
        let structs = build_struct_type_index(&story);
        let enums = build_enum_type_index(&story);
        let targets = build_target_symbol_index(&story);
        let interface_members = build_interface_member_index(&story);
        let cases = [
            (
                dynamic_interface_call(variable("label"), "score", vec![Expression::NumberInt(1)]),
                "Dynamic interface function 'score' has base type string but expected interface",
            ),
            (
                dynamic_interface_call(variable("route"), "missing", vec![]),
                "Interface 'IItem' does not declare member 'missing'",
            ),
            (
                dynamic_interface_call(variable("route"), "target", vec![Expression::NumberInt(1)]),
                "Interface 'IItem' member 'target' is a knot but dynamic function call requires a function",
            ),
        ];

        for (expression, expected_message) in cases {
            let error = infer_expression_type(
                &expression,
                &scopes,
                &structs,
                &enums,
                &targets,
                &interface_members,
                Some("game"),
                Some("main"),
            )
            .unwrap_err();

            assert_eq!(error.message(), expected_message);
        }
    }

    #[test]
    fn rejects_equality_between_unrelated_types() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR score: int = 1\n\
             VAR ratio: float = 1.0\n\
             VAR source_player: Player = %Player{ hp: 10 }\n\
             VAR scores: int[] = [score]\n\
             -> DONE",
        );
        let scopes = build_variable_scope_index(&story);
        let structs = build_struct_type_index(&story);
        let enums = build_enum_type_index(&story);
        let targets = build_target_symbol_index(&story);
        let interface_members = build_interface_member_index(&story);
        let cases = [
            (
                binary(
                    BinaryOperator::Equals,
                    variable("scores"),
                    variable("source_player"),
                ),
                "Operator '==' is not defined for types int[] and Player",
            ),
            (
                binary(
                    BinaryOperator::NotEquals,
                    variable("score"),
                    variable("ratio"),
                ),
                "Operator '!=' is not defined for types int and float",
            ),
        ];

        for (expression, expected_message) in cases {
            let error = infer_expression_type(
                &expression,
                &scopes,
                &structs,
                &enums,
                &targets,
                &interface_members,
                None,
                None,
            )
            .unwrap_err();

            assert_eq!(error.message(), expected_message);
        }
    }

    #[test]
    fn rejects_ordered_comparisons_for_non_numeric_types() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             ENUM State { Idle Busy }\n\
             VAR score: int = 1\n\
             VAR source_player: Player = %Player{ hp: 10 }\n\
             VAR other_player: Player = %Player{ hp: 20 }\n\
             VAR first_target: -> = -> knot\n\
             VAR second_target: -> = -> other\n\
             VAR scores: int[] = [score]\n\
             VAR other_scores: int[] = [score]\n\
             -> DONE\n\
             == knot ==\n\
             -> DONE\n\
             == other ==\n\
             -> DONE",
        );
        let scopes = build_variable_scope_index(&story);
        let structs = build_struct_type_index(&story);
        let enums = build_enum_type_index(&story);
        let targets = build_target_symbol_index(&story);
        let interface_members = build_interface_member_index(&story);
        let cases = [
            (
                binary(
                    BinaryOperator::GreaterThan,
                    variable("scores"),
                    variable("other_scores"),
                ),
                "Operator '>' is not defined for types int[] and int[]",
            ),
            (
                binary(
                    BinaryOperator::LessThan,
                    variable("source_player"),
                    variable("other_player"),
                ),
                "Operator '<' is not defined for types Player and Player",
            ),
            (
                binary(
                    BinaryOperator::GreaterThan,
                    variable("first_target"),
                    variable("second_target"),
                ),
                "Operator '>' is not defined for types -> and ->",
            ),
            (
                binary(
                    BinaryOperator::GreaterThan,
                    Expression::FieldAccess {
                        base: Box::new(variable("State")),
                        field: "Idle".to_string(),
                    },
                    Expression::FieldAccess {
                        base: Box::new(variable("State")),
                        field: "Busy".to_string(),
                    },
                ),
                "Operator '>' is not defined for types State and State",
            ),
            (
                binary(
                    BinaryOperator::GreaterThan,
                    Expression::String("a".to_string()),
                    Expression::String("b".to_string()),
                ),
                "Operator '>' is not defined for types string and string",
            ),
        ];

        for (expression, expected_message) in cases {
            let error = infer_expression_type(
                &expression,
                &scopes,
                &structs,
                &enums,
                &targets,
                &interface_members,
                None,
                None,
            )
            .unwrap_err();

            assert_eq!(error.message(), expected_message);
        }
    }
}
