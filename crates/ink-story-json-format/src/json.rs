use std::collections::BTreeMap;

use serde_json::{Map, Number, Value as JsonValue};

use crate::{
    Container, DictKey, DictKeyType, DictValue, FormatError, InterfaceDefinition,
    InterfaceMemberKind, InternalFunction, NamedContainer, NativeFunction, Object, Program,
    DICT_VALUE_MARKER, DYNAMIC_INTERFACE_ARGS_KEY, DYNAMIC_INTERFACE_FUNCTION_KEY,
    DYNAMIC_INTERFACE_NAME_KEY, DYNAMIC_INTERFACE_TARGET_KEY, INTERFACES_METADATA_KEY,
    INTERFACE_IMPLEMENTATIONS_KEY, INTERFACE_MEMBERS_KEY,
};

pub(crate) fn program_from_str(input: &str) -> Result<Program, FormatError> {
    let value = serde_json::from_str(input)
        .map_err(|error| FormatError::new(format!("story JSON is not valid JSON: {error}")))?;
    program_from_value(value)
}

pub(crate) fn program_from_value(value: JsonValue) -> Result<Program, FormatError> {
    let obj = value
        .as_object()
        .ok_or_else(|| FormatError::new("compiled story JSON must be an object"))?;

    let ink_version = required_i32(obj, "inkVersion")?;
    let root_value = obj
        .get("root")
        .ok_or_else(|| FormatError::new("compiled story JSON is missing root"))?;
    let root = container_from_value(root_value, None)?;
    let internal_functions = match obj.get("internalFunctions") {
        Some(value) => internal_functions_from_value(value)?,
        None => BTreeMap::new(),
    };
    let interfaces = match obj.get(INTERFACES_METADATA_KEY) {
        Some(value) => interfaces_from_value(value)?,
        None => BTreeMap::new(),
    };

    Ok(Program {
        ink_version,
        root,
        internal_functions,
        interfaces,
    })
}

pub(crate) fn program_to_string(program: &Program) -> Result<String, FormatError> {
    serde_json::to_string(&program_to_value(program)).map_err(|error| {
        FormatError::new(format!("failed to serialize compiled story JSON: {error}"))
    })
}

pub(crate) fn program_to_value(program: &Program) -> JsonValue {
    let mut obj = Map::new();
    obj.insert(
        "inkVersion".to_string(),
        JsonValue::Number(program.ink_version.into()),
    );
    obj.insert("root".to_string(), container_to_value(&program.root, true));
    if !program.internal_functions.is_empty() {
        obj.insert(
            "internalFunctions".to_string(),
            internal_functions_to_value(&program.internal_functions),
        );
    }
    if !program.interfaces.is_empty() {
        obj.insert(
            INTERFACES_METADATA_KEY.to_string(),
            interfaces_to_value(&program.interfaces),
        );
    }
    JsonValue::Object(obj)
}

fn internal_functions_from_value(
    value: &JsonValue,
) -> Result<BTreeMap<String, InternalFunction>, FormatError> {
    let obj = value
        .as_object()
        .ok_or_else(|| FormatError::new("internalFunctions must be a JSON object"))?;
    let mut functions = BTreeMap::new();
    for (name, value) in obj {
        functions.insert(name.clone(), internal_function_from_value(name, value)?);
    }
    Ok(functions)
}

fn internal_function_from_value(
    name: &str,
    value: &JsonValue,
) -> Result<InternalFunction, FormatError> {
    let obj = value
        .as_object()
        .ok_or_else(|| FormatError::new(format!("internal function '{name}' must be an object")))?;
    let path = required_json_string(obj, "path")?.to_string();
    let return_type = required_json_string(obj, "returnType")?.to_string();
    let args = usize::try_from(required_i32(obj, "args")?)
        .map_err(|_| FormatError::new(format!("internal function '{name}' args must be >= 0")))?;
    let arg_types = match obj.get("argTypes") {
        Some(value) => string_array_from_value(value, "argTypes")?,
        None if args == 0 => Vec::new(),
        None => {
            return Err(FormatError::new(format!(
                "internal function '{name}' is missing argTypes"
            )))
        }
    };
    if arg_types.len() != args {
        return Err(FormatError::new(format!(
            "internal function '{name}' args does not match argTypes length"
        )));
    }
    Ok(InternalFunction {
        path,
        args,
        arg_types,
        return_type,
    })
}

fn string_array_from_value(value: &JsonValue, field: &str) -> Result<Vec<String>, FormatError> {
    value
        .as_array()
        .ok_or_else(|| FormatError::new(format!("{field} must be an array")))?
        .iter()
        .map(|value| Ok(json_value_to_string(value, field)?.to_string()))
        .collect()
}

fn internal_functions_to_value(functions: &BTreeMap<String, InternalFunction>) -> JsonValue {
    let mut obj = Map::new();
    for (name, function) in functions {
        let mut function_obj = Map::new();
        function_obj.insert("path".to_string(), JsonValue::String(function.path.clone()));
        function_obj.insert(
            "args".to_string(),
            JsonValue::Number((function.args as u64).into()),
        );
        function_obj.insert(
            "argTypes".to_string(),
            JsonValue::Array(
                function
                    .arg_types
                    .iter()
                    .map(|arg| JsonValue::String(arg.clone()))
                    .collect(),
            ),
        );
        function_obj.insert(
            "returnType".to_string(),
            JsonValue::String(function.return_type.clone()),
        );
        obj.insert(name.clone(), JsonValue::Object(function_obj));
    }
    JsonValue::Object(obj)
}

fn interfaces_from_value(
    value: &JsonValue,
) -> Result<BTreeMap<String, InterfaceDefinition>, FormatError> {
    let obj = value
        .as_object()
        .ok_or_else(|| FormatError::new("interfaces must be a JSON object"))?;
    let mut interfaces = BTreeMap::new();
    for (name, value) in obj {
        interfaces.insert(name.clone(), interface_definition_from_value(name, value)?);
    }
    Ok(interfaces)
}

fn interface_definition_from_value(
    name: &str,
    value: &JsonValue,
) -> Result<InterfaceDefinition, FormatError> {
    let obj = value
        .as_object()
        .ok_or_else(|| FormatError::new(format!("interface '{name}' must be an object")))?;
    let members_value = obj.get(INTERFACE_MEMBERS_KEY).ok_or_else(|| {
        FormatError::new(format!(
            "interface '{name}' is missing {INTERFACE_MEMBERS_KEY}"
        ))
    })?;
    let implementations_value = obj.get(INTERFACE_IMPLEMENTATIONS_KEY).ok_or_else(|| {
        FormatError::new(format!(
            "interface '{name}' is missing {INTERFACE_IMPLEMENTATIONS_KEY}"
        ))
    })?;
    Ok(InterfaceDefinition {
        members: interface_members_from_value(name, members_value)?,
        implementations: string_array_from_value(
            implementations_value,
            INTERFACE_IMPLEMENTATIONS_KEY,
        )?,
    })
}

fn interface_members_from_value(
    interface_name: &str,
    value: &JsonValue,
) -> Result<BTreeMap<String, InterfaceMemberKind>, FormatError> {
    let obj = value.as_object().ok_or_else(|| {
        FormatError::new(format!(
            "interface '{interface_name}' {INTERFACE_MEMBERS_KEY} must be a JSON object"
        ))
    })?;
    let mut members = BTreeMap::new();
    for (member, kind_value) in obj {
        let kind_token = json_value_to_string(kind_value, INTERFACE_MEMBERS_KEY)?;
        let kind = InterfaceMemberKind::from_token(kind_token).ok_or_else(|| {
            FormatError::new(format!(
                "interface '{interface_name}' member '{member}' has unsupported kind: {kind_token}"
            ))
        })?;
        members.insert(member.clone(), kind);
    }
    Ok(members)
}

fn interfaces_to_value(interfaces: &BTreeMap<String, InterfaceDefinition>) -> JsonValue {
    let mut obj = Map::new();
    for (name, interface) in interfaces {
        let mut interface_obj = Map::new();
        interface_obj.insert(
            INTERFACE_MEMBERS_KEY.to_string(),
            interface_members_to_value(&interface.members),
        );
        interface_obj.insert(
            INTERFACE_IMPLEMENTATIONS_KEY.to_string(),
            JsonValue::Array(
                interface
                    .implementations
                    .iter()
                    .map(|implementation| JsonValue::String(implementation.clone()))
                    .collect(),
            ),
        );
        obj.insert(name.clone(), JsonValue::Object(interface_obj));
    }
    JsonValue::Object(obj)
}

fn interface_members_to_value(members: &BTreeMap<String, InterfaceMemberKind>) -> JsonValue {
    let mut obj = Map::new();
    for (member, kind) in members {
        obj.insert(member.clone(), JsonValue::String(kind.token().to_string()));
    }
    JsonValue::Object(obj)
}

pub(crate) fn container_from_value(
    value: &JsonValue,
    name_hint: Option<String>,
) -> Result<Container, FormatError> {
    let array = value
        .as_array()
        .ok_or_else(|| FormatError::new("container must be a JSON array"))?;
    let Some((terminator, content_values)) = array.split_last() else {
        return Err(FormatError::new(
            "container array must include a terminator",
        ));
    };

    let mut content = Vec::with_capacity(content_values.len());
    for value in content_values {
        content.push(object_from_value(value)?);
    }

    let mut name = name_hint;
    let mut flags = None;
    let mut named_content = Vec::new();

    match terminator {
        JsonValue::Null => {}
        JsonValue::Object(obj) if obj.is_empty() => {
            return Err(FormatError::new(
                "container terminator object must contain metadata or named content",
            ));
        }
        JsonValue::Object(obj) => {
            for (key, value) in obj {
                match key.as_str() {
                    "#f" => flags = Some(json_value_to_i32(value, "#f")?),
                    "#n" => name = Some(json_value_to_string(value, "#n")?.to_string()),
                    name => {
                        let container = container_from_value(value, Some(name.to_string()))?;
                        named_content.push(NamedContainer::new(name, container));
                    }
                }
            }
        }
        _ => {
            return Err(FormatError::new(
                "container terminator must be null or an object",
            ))
        }
    }

    Ok(Container {
        content,
        named_content,
        name,
        flags,
    })
}

pub(crate) fn container_to_value(container: &Container, include_name: bool) -> JsonValue {
    let mut values = container
        .content
        .iter()
        .map(object_to_value)
        .collect::<Vec<_>>();
    values.push(container_terminator_to_value(container, include_name));
    JsonValue::Array(values)
}

fn container_terminator_to_value(container: &Container, include_name: bool) -> JsonValue {
    let mut obj = Map::new();

    for named in &container.named_content {
        obj.insert(
            named.name.clone(),
            container_to_value(&named.container, false),
        );
    }

    if let Some(flags) = container.flags {
        obj.insert("#f".to_string(), JsonValue::Number(flags.into()));
    }

    if include_name {
        if let Some(name) = &container.name {
            obj.insert("#n".to_string(), JsonValue::String(name.clone()));
        }
    }

    if obj.is_empty() {
        JsonValue::Null
    } else {
        JsonValue::Object(obj)
    }
}

pub(crate) fn object_from_value(value: &JsonValue) -> Result<Object, FormatError> {
    match value {
        JsonValue::Null => Err(FormatError::new(
            "null is only valid as a container terminator",
        )),
        JsonValue::Bool(value) => Ok(Object::Bool(*value)),
        JsonValue::Number(number) => {
            if let Some(value) = number.as_i64() {
                let value = i32::try_from(value)
                    .map_err(|_| FormatError::new("integer value is outside i32 range"))?;
                Ok(Object::Int(value))
            } else {
                let value = number
                    .as_f64()
                    .ok_or_else(|| FormatError::new("number value is not representable as f64"))?;
                Ok(Object::Float(value))
            }
        }
        JsonValue::String(value) => string_object_from_token(value),
        JsonValue::Array(_) => array_object_from_value(value),
        JsonValue::Object(obj) => object_from_map(obj),
    }
}

pub(crate) fn object_to_value(object: &Object) -> JsonValue {
    match object {
        Object::Container(container) => container_to_value(container, true),
        Object::String(text) if text == "\n" => JsonValue::String("\n".to_string()),
        Object::String(text) => JsonValue::String(format!("^{text}")),
        Object::ControlCommand(command) => JsonValue::String(command.token().to_string()),
        Object::Divert { target, variable } => {
            divert_to_value("->", target, *variable, false, None)
        }
        Object::TunnelDivert { target, variable } => {
            divert_to_value("->t->", target, *variable, false, None)
        }
        Object::FunctionDivert { target } => divert_to_value("f()", target, false, false, None),
        Object::ExternalFunction { target, args } => {
            divert_to_value("x()", target, false, false, Some(*args))
        }
        Object::DynamicInterfaceTarget { interface, member } => {
            dynamic_interface_to_value(DYNAMIC_INTERFACE_TARGET_KEY, interface, member, None)
        }
        Object::DynamicInterfaceFunctionCall {
            interface,
            member,
            args,
        } => dynamic_interface_to_value(
            DYNAMIC_INTERFACE_FUNCTION_KEY,
            interface,
            member,
            Some(*args),
        ),
        Object::ConditionalDivert { target } => divert_to_value("->", target, false, true, None),
        Object::DivertTarget(target) => {
            let mut obj = Map::new();
            obj.insert("^->".to_string(), JsonValue::String(target.clone()));
            JsonValue::Object(obj)
        }
        Object::ReadCount(target) => single_property_object("CNT?", target),
        Object::VariableAssignment(name) => single_property_object("temp=", name),
        Object::GlobalVariableAssignment(name) => single_property_object("VAR=", name),
        Object::TempVariableReassignment(name) => variable_assignment_to_value("temp=", name),
        Object::VariableReassignment(name) => variable_assignment_to_value("VAR=", name),
        Object::VariableReference(name) => single_property_object("VAR?", name),
        Object::VariablePointer {
            name,
            context_index,
        } => {
            let mut obj = Map::new();
            obj.insert("^var".to_string(), JsonValue::String(name.clone()));
            obj.insert("ci".to_string(), JsonValue::Number((*context_index).into()));
            JsonValue::Object(obj)
        }
        Object::ChoicePoint { target, flags } => {
            let mut obj = Map::new();
            obj.insert("*".to_string(), JsonValue::String(target.clone()));
            obj.insert("flg".to_string(), JsonValue::Number((*flags).into()));
            JsonValue::Object(obj)
        }
        Object::Glue => JsonValue::String("<>".to_string()),
        Object::Tag { is_start } => {
            JsonValue::String(if *is_start { "#" } else { "/#" }.to_string())
        }
        Object::Bool(value) => JsonValue::Bool(*value),
        Object::Int(value) => JsonValue::Number((*value).into()),
        Object::Float(value) => JsonValue::Number(
            Number::from_f64(*value).expect("compiled story float values must be finite"),
        ),
        Object::ValueArray(values) => {
            JsonValue::Array(values.iter().map(object_to_value).collect())
        }
        Object::ValueObject(fields) => {
            let mut obj = Map::new();
            for (key, value) in fields {
                obj.insert(key.clone(), object_to_value(value));
            }
            JsonValue::Object(obj)
        }
        Object::ValueDict(value) => dict_to_value(value),
        Object::Void => JsonValue::String("void".to_string()),
        Object::NativeFunction(function) => JsonValue::String(function.token().to_string()),
    }
}

fn array_object_from_value(value: &JsonValue) -> Result<Object, FormatError> {
    match container_from_value(value, None) {
        Ok(container) => Ok(Object::Container(container)),
        Err(container_error) => match value {
            JsonValue::Array(values) => {
                if let Some(object) = dict_from_array(values)? {
                    return Ok(object);
                }
                let values = values
                    .iter()
                    .map(object_from_value)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Object::ValueArray(values))
            }
            _ => Err(container_error),
        },
    }
}

fn string_object_from_token(token: &str) -> Result<Object, FormatError> {
    if let Some(text) = token.strip_prefix('^') {
        return Ok(Object::String(text.to_string()));
    }

    if token == "\n" {
        return Ok(Object::String("\n".to_string()));
    }

    if token == "<>" {
        return Ok(Object::Glue);
    }

    if token == "void" {
        return Ok(Object::Void);
    }

    if token == "#" {
        return Ok(Object::Tag { is_start: true });
    }

    if token == "/#" {
        return Ok(Object::Tag { is_start: false });
    }

    if let Some(command) = crate::ControlCommand::from_token(token) {
        return Ok(Object::ControlCommand(command));
    }

    NativeFunction::from_token(token)
        .map(Object::NativeFunction)
        .ok_or_else(|| FormatError::new(format!("unsupported native function token: {token}")))
}

fn object_from_map(obj: &Map<String, JsonValue>) -> Result<Object, FormatError> {
    if let Some(target) = obj.get("^->") {
        return Ok(Object::DivertTarget(
            json_value_to_string(target, "^->")?.to_string(),
        ));
    }

    if let Some(name) = obj.get("^var") {
        let context_index = match obj.get("ci") {
            Some(value) => json_value_to_i32(value, "ci")?,
            None => -1,
        };
        return Ok(Object::VariablePointer {
            name: json_value_to_string(name, "^var")?.to_string(),
            context_index,
        });
    }

    if let Some(object) = dynamic_interface_from_map(obj)? {
        return Ok(object);
    }

    if let Some(object) = divert_from_map(obj)? {
        return Ok(object);
    }

    if let Some(target) = obj.get("*") {
        let flags = match obj.get("flg") {
            Some(value) => json_value_to_i32(value, "flg")?,
            None => 0,
        };
        return Ok(Object::ChoicePoint {
            target: json_value_to_string(target, "*")?.to_string(),
            flags,
        });
    }

    if let Some(name) = obj.get("VAR?") {
        return Ok(Object::VariableReference(
            json_value_to_string(name, "VAR?")?.to_string(),
        ));
    }

    if let Some(name) = obj.get("CNT?") {
        return Ok(Object::ReadCount(
            json_value_to_string(name, "CNT?")?.to_string(),
        ));
    }

    if let Some(assignment) = variable_assignment_from_map(obj)? {
        return Ok(assignment);
    }

    value_object_from_map(obj)
}

fn dict_from_array(values: &[JsonValue]) -> Result<Option<Object>, FormatError> {
    let Some(JsonValue::String(marker)) = values.first() else {
        return Ok(None);
    };
    if marker != DICT_VALUE_MARKER {
        return Ok(None);
    }
    if values.len() != 3 {
        return Err(FormatError::new(
            "Dict value marker must be [\"dict\", keyType, entries]",
        ));
    }

    let key_type_token = json_value_to_string(&values[1], "Dict key type")?;
    let key_type = DictKeyType::from_token(key_type_token).ok_or_else(|| {
        FormatError::new(format!(
            "Dict key type must be 'string' or 'int', got {key_type_token}"
        ))
    })?;
    let entry_values = values[2]
        .as_array()
        .ok_or_else(|| FormatError::new("Dict entries must be an array"))?;
    let mut entries = BTreeMap::new();
    for (index, entry_value) in entry_values.iter().enumerate() {
        let entry = entry_value
            .as_array()
            .ok_or_else(|| FormatError::new(format!("Dict entry {index} must be an array")))?;
        if entry.len() != 2 {
            return Err(FormatError::new(format!(
                "Dict entry {index} must contain a key and value"
            )));
        }
        let key = dict_key_from_value(key_type, &entry[0], index)?;
        let value = object_from_value(&entry[1])?;
        if entries.insert(key.clone(), value).is_some() {
            return Err(FormatError::new(format!(
                "Dict entry {index} duplicates key {key:?}"
            )));
        }
    }

    Ok(Some(Object::ValueDict(DictValue { key_type, entries })))
}

fn dict_key_from_value(
    key_type: DictKeyType,
    value: &JsonValue,
    index: usize,
) -> Result<DictKey, FormatError> {
    match key_type {
        DictKeyType::String => Ok(DictKey::String(
            value
                .as_str()
                .ok_or_else(|| {
                    FormatError::new(format!("Dict entry {index} key must be a string"))
                })?
                .to_string(),
        )),
        DictKeyType::Int => Ok(DictKey::Int(json_value_to_i32(
            value,
            &format!("Dict entry {index} key"),
        )?)),
    }
}

fn dict_to_value(value: &DictValue) -> JsonValue {
    let entries = value
        .entries
        .iter()
        .map(|(key, object)| {
            debug_assert_eq!(key.key_type(), value.key_type);
            JsonValue::Array(vec![dict_key_to_value(key), object_to_value(object)])
        })
        .collect();

    JsonValue::Array(vec![
        JsonValue::String(DICT_VALUE_MARKER.to_string()),
        JsonValue::String(value.key_type.token().to_string()),
        JsonValue::Array(entries),
    ])
}

fn dict_key_to_value(key: &DictKey) -> JsonValue {
    match key {
        DictKey::String(value) => JsonValue::String(value.clone()),
        DictKey::Int(value) => JsonValue::Number((*value).into()),
    }
}

fn value_object_from_map(obj: &Map<String, JsonValue>) -> Result<Object, FormatError> {
    let mut fields = BTreeMap::new();
    for (key, value) in obj {
        fields.insert(key.clone(), object_from_value(value)?);
    }
    Ok(Object::ValueObject(fields))
}

fn divert_from_map(obj: &Map<String, JsonValue>) -> Result<Option<Object>, FormatError> {
    if let Some(target) = obj.get("->") {
        let target = json_value_to_string(target, "->")?.to_string();
        let variable = obj.contains_key("var");
        let conditional = obj.contains_key("c");
        return Ok(Some(if conditional {
            Object::ConditionalDivert { target }
        } else {
            Object::Divert { target, variable }
        }));
    }

    if let Some(target) = obj.get("f()") {
        return Ok(Some(Object::FunctionDivert {
            target: json_value_to_string(target, "f()")?.to_string(),
        }));
    }

    if let Some(target) = obj.get("->t->") {
        return Ok(Some(Object::TunnelDivert {
            target: json_value_to_string(target, "->t->")?.to_string(),
            variable: obj.contains_key("var"),
        }));
    }

    if let Some(target) = obj.get("x()") {
        return Ok(Some(Object::ExternalFunction {
            target: json_value_to_string(target, "x()")?.to_string(),
            args: match obj.get("exArgs") {
                Some(value) => json_value_to_usize(value, "exArgs")?,
                None => 0,
            },
        }));
    }

    Ok(None)
}

fn dynamic_interface_from_map(obj: &Map<String, JsonValue>) -> Result<Option<Object>, FormatError> {
    if let Some(member) = obj.get(DYNAMIC_INTERFACE_TARGET_KEY) {
        return Ok(Some(Object::DynamicInterfaceTarget {
            interface: required_json_string(obj, DYNAMIC_INTERFACE_NAME_KEY)?.to_string(),
            member: json_value_to_string(member, DYNAMIC_INTERFACE_TARGET_KEY)?.to_string(),
        }));
    }

    if let Some(member) = obj.get(DYNAMIC_INTERFACE_FUNCTION_KEY) {
        return Ok(Some(Object::DynamicInterfaceFunctionCall {
            interface: required_json_string(obj, DYNAMIC_INTERFACE_NAME_KEY)?.to_string(),
            member: json_value_to_string(member, DYNAMIC_INTERFACE_FUNCTION_KEY)?.to_string(),
            args: required_usize(obj, DYNAMIC_INTERFACE_ARGS_KEY)?,
        }));
    }

    Ok(None)
}

fn dynamic_interface_to_value(
    key: &str,
    interface: &str,
    member: &str,
    args: Option<usize>,
) -> JsonValue {
    let mut obj = Map::new();
    obj.insert(key.to_string(), JsonValue::String(member.to_string()));
    obj.insert(
        DYNAMIC_INTERFACE_NAME_KEY.to_string(),
        JsonValue::String(interface.to_string()),
    );
    if let Some(args) = args {
        obj.insert(
            DYNAMIC_INTERFACE_ARGS_KEY.to_string(),
            JsonValue::Number(args.into()),
        );
    }
    JsonValue::Object(obj)
}

fn divert_to_value(
    key: &str,
    target: &str,
    variable: bool,
    conditional: bool,
    external_args: Option<usize>,
) -> JsonValue {
    let mut obj = Map::new();
    obj.insert(key.to_string(), JsonValue::String(target.to_string()));
    if variable {
        obj.insert("var".to_string(), JsonValue::Bool(true));
    }
    if conditional {
        obj.insert("c".to_string(), JsonValue::Bool(true));
    }
    if let Some(external_args) = external_args {
        if external_args > 0 {
            obj.insert(
                "exArgs".to_string(),
                JsonValue::Number(external_args.into()),
            );
        }
    }
    JsonValue::Object(obj)
}

fn variable_assignment_from_map(
    obj: &Map<String, JsonValue>,
) -> Result<Option<Object>, FormatError> {
    if let Some(name) = obj.get("VAR=") {
        let name = json_value_to_string(name, "VAR=")?.to_string();
        return Ok(Some(if obj.contains_key("re") {
            Object::VariableReassignment(name)
        } else {
            Object::GlobalVariableAssignment(name)
        }));
    }

    if let Some(name) = obj.get("temp=") {
        let name = json_value_to_string(name, "temp=")?.to_string();
        return Ok(Some(if obj.contains_key("re") {
            Object::TempVariableReassignment(name)
        } else {
            Object::VariableAssignment(name)
        }));
    }

    Ok(None)
}

fn single_property_object(key: &str, value: &str) -> JsonValue {
    let mut obj = Map::new();
    obj.insert(key.to_string(), JsonValue::String(value.to_string()));
    JsonValue::Object(obj)
}

fn variable_assignment_to_value(key: &str, name: &str) -> JsonValue {
    let mut obj = Map::new();
    obj.insert(key.to_string(), JsonValue::String(name.to_string()));
    obj.insert("re".to_string(), JsonValue::Bool(true));
    JsonValue::Object(obj)
}

fn required_i32(obj: &Map<String, JsonValue>, key: &str) -> Result<i32, FormatError> {
    let value = obj
        .get(key)
        .ok_or_else(|| FormatError::new(format!("compiled story JSON is missing {key}")))?;
    json_value_to_i32(value, key)
}

fn required_json_string<'a>(
    obj: &'a Map<String, JsonValue>,
    key: &str,
) -> Result<&'a str, FormatError> {
    let value = obj
        .get(key)
        .ok_or_else(|| FormatError::new(format!("compiled story JSON is missing {key}")))?;
    json_value_to_string(value, key)
}

fn required_usize(obj: &Map<String, JsonValue>, key: &str) -> Result<usize, FormatError> {
    let value = obj
        .get(key)
        .ok_or_else(|| FormatError::new(format!("compiled story JSON is missing {key}")))?;
    json_value_to_usize(value, key)
}

fn json_value_to_string<'a>(value: &'a JsonValue, key: &str) -> Result<&'a str, FormatError> {
    value
        .as_str()
        .ok_or_else(|| FormatError::new(format!("{key} must be a string")))
}

fn json_value_to_i32(value: &JsonValue, key: &str) -> Result<i32, FormatError> {
    let value = value
        .as_i64()
        .ok_or_else(|| FormatError::new(format!("{key} must be an integer")))?;
    i32::try_from(value).map_err(|_| FormatError::new(format!("{key} is outside i32 range")))
}

fn json_value_to_usize(value: &JsonValue, key: &str) -> Result<usize, FormatError> {
    let value = value
        .as_u64()
        .ok_or_else(|| FormatError::new(format!("{key} must be a non-negative integer")))?;
    usize::try_from(value).map_err(|_| FormatError::new(format!("{key} is outside usize range")))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::*;
    use crate::ControlCommand;

    #[test]
    fn writes_plain_text_story_json() {
        let program = Program::new(Container::unnamed(vec![
            Object::Container(Container::unnamed(vec![
                Object::String("Line.".to_string()),
                Object::String("\n".to_string()),
                Object::Container(Container::named(
                    "g-0",
                    vec![Object::ControlCommand(ControlCommand::Done)],
                )),
            ])),
            Object::ControlCommand(ControlCommand::Done),
        ]));

        assert_eq!(
            program.to_json_value(),
            json!({
                "inkVersion": 1,
                "root": [["^Line.", "\n", ["done", {"#n": "g-0"}], null], "done", null]
            })
        );
    }

    #[test]
    fn roundtrips_named_content_and_command_tokens() {
        let input = json!({
            "inkVersion": 1,
            "root": [
                ["#", "^tag", "/#", {"->t->": "knot"}, {"#n": "g-0"}],
                "done",
                {
                    "knot": ["ev", "str", "^value", "/str", "/ev", "end", {"#f": 1}],
                    "global decl": ["ev", 2, {"VAR=": "x"}, "/ev", "end", null]
                }
            ]
        });

        let program = program_from_value(input.clone()).expect("format should parse");

        assert_eq!(program_to_value(&program), input);
    }

    #[test]
    fn roundtrips_module_shaped_named_content_without_schema_changes() {
        let input = json!({
            "inkVersion": 1,
            "root": [
                {"->": "game.main"},
                "done",
                {
                    "game": [
                        {
                            "main": ["^Start", "\n", {"->": "support.helper"}, null]
                        }
                    ],
                    "support": [
                        {
                            "helper": ["^Support", "\n", "end", null]
                        }
                    ],
                    "global decl": [
                        "ev",
                        1,
                        {"VAR=": "support::shown"},
                        "/ev",
                        "end",
                        null
                    ]
                }
            ]
        });

        let program = program_from_value(input.clone()).expect("format should parse modules");

        assert_eq!(program_to_value(&program), input);
    }

    #[test]
    fn roundtrips_internal_function_metadata() {
        let input = json!({
            "inkVersion": 1,
            "root": ["done", null],
            "internalFunctions": {
                "game::read_config": {
                    "path": "game.read_config",
                    "args": 1,
                    "argTypes": ["string"],
                    "returnType": "string"
                }
            }
        });

        let program = program_from_value(input.clone()).expect("format should parse metadata");

        assert_eq!(program_to_value(&program), input);
        assert_eq!(
            program.internal_functions["game::read_config"],
            InternalFunction::new("game.read_config", vec!["string".to_string()], "string")
        );
    }

    #[test]
    fn roundtrips_interface_metadata() {
        let input = json!({
            "inkVersion": 1,
            "root": ["done", null],
            "interfaces": {
                "IItem": {
                    "members": {
                        "score": "function",
                        "target": "knot"
                    },
                    "implementations": ["left", "right"]
                }
            }
        });

        let program = program_from_value(input.clone()).expect("format should parse interfaces");

        assert_eq!(program_to_value(&program), input);
        assert_eq!(
            program.interfaces["IItem"].members["target"],
            InterfaceMemberKind::Knot
        );
        assert_eq!(
            program.interfaces["IItem"].members["score"],
            InterfaceMemberKind::Function
        );
        assert_eq!(
            program.interfaces["IItem"].implementations,
            vec!["left".to_string(), "right".to_string()]
        );
    }

    #[test]
    fn writes_interface_metadata_from_typed_model() {
        let mut members = BTreeMap::new();
        members.insert("target".to_string(), InterfaceMemberKind::Knot);
        members.insert("score".to_string(), InterfaceMemberKind::Function);

        let mut program = Program::new(Container::unnamed(vec![Object::ControlCommand(
            ControlCommand::Done,
        )]));
        program.interfaces.insert(
            "IItem".to_string(),
            InterfaceDefinition::new(members, vec!["left".to_string(), "right".to_string()]),
        );

        assert_eq!(
            program.to_json_value(),
            json!({
                "inkVersion": 1,
                "root": ["done", null],
                "interfaces": {
                    "IItem": {
                        "members": {
                            "score": "function",
                            "target": "knot"
                        },
                        "implementations": ["left", "right"]
                    }
                }
            })
        );
    }

    #[test]
    fn rejects_unknown_interface_member_kind() {
        let input = json!({
            "inkVersion": 1,
            "root": ["done", null],
            "interfaces": {
                "IItem": {
                    "members": {
                        "target": "room"
                    },
                    "implementations": ["left"]
                }
            }
        });

        let error = program_from_value(input).expect_err("metadata should be rejected");

        assert!(error
            .to_string()
            .contains("interface 'IItem' member 'target' has unsupported kind: room"));
    }

    #[test]
    fn rejects_internal_function_metadata_with_mismatched_arg_count() {
        let input = json!({
            "inkVersion": 1,
            "root": ["done", null],
            "internalFunctions": {
                "game::bad": {
                    "path": "game.bad",
                    "args": 2,
                    "argTypes": ["int"],
                    "returnType": "int"
                }
            }
        });

        let error = program_from_value(input).expect_err("metadata should be rejected");

        assert!(error
            .to_string()
            .contains("internal function 'game::bad' args does not match argTypes length"));
    }

    #[test]
    fn native_function_tokens_roundtrip_as_typed_objects() {
        for function in NativeFunction::ALL {
            let token = function.token();
            let value = json!(token);

            assert_eq!(
                Object::from_json_value(value.clone()).unwrap(),
                Object::NativeFunction(function)
            );
            assert_eq!(Object::NativeFunction(function).to_json_value(), value);
        }
    }

    #[test]
    fn rejects_unknown_native_function_tokens() {
        let error = Object::from_json_value(json!("UNKNOWN_NATIVE")).unwrap_err();

        assert_eq!(
            error.message(),
            "unsupported native function token: UNKNOWN_NATIVE"
        );
    }

    #[test]
    fn roundtrips_dynamic_interface_instruction_objects() {
        let target = Object::DynamicInterfaceTarget {
            interface: "IItem".to_string(),
            member: "target".to_string(),
        };
        let target_value = target.to_json_value();

        assert_eq!(
            target_value,
            json!({
                "i->": "target",
                "interface": "IItem"
            })
        );
        assert_eq!(Object::from_json_value(target_value).unwrap(), target);

        let call = Object::DynamicInterfaceFunctionCall {
            interface: "IItem".to_string(),
            member: "score".to_string(),
            args: 2,
        };
        let call_value = call.to_json_value();

        assert_eq!(
            call_value,
            json!({
                "i()": "score",
                "interface": "IItem",
                "args": 2
            })
        );
        assert_eq!(Object::from_json_value(call_value).unwrap(), call);
    }

    #[test]
    fn rejects_dynamic_interface_function_without_arg_count() {
        let error = Object::from_json_value(json!({
            "i()": "score",
            "interface": "IItem"
        }))
        .unwrap_err();

        assert_eq!(error.message(), "compiled story JSON is missing args");
    }

    #[test]
    fn roundtrips_dynamic_array_values() {
        let object = Object::ValueArray(vec![
            Object::Int(1),
            Object::Float(2.5),
            Object::Bool(true),
            Object::String("text".to_string()),
            Object::ValueArray(vec![Object::Int(3)]),
        ]);

        let value = object.to_json_value();

        assert_eq!(value, json!([1, 2.5, true, "^text", [3]]));
        assert_eq!(Object::from_json_value(value).unwrap(), object);
    }

    #[test]
    fn roundtrips_string_key_dict_values() {
        let mut entries = BTreeMap::new();
        entries.insert(DictKey::String("ada".to_string()), Object::Int(10));
        entries.insert(
            DictKey::String("grace".to_string()),
            Object::String("compiler".to_string()),
        );
        let object = Object::ValueDict(
            DictValue::new(DictKeyType::String, entries).expect("keys should match"),
        );
        let value = object.to_json_value();

        assert_eq!(
            value,
            json!(["dict", "string", [["ada", 10], ["grace", "^compiler"]]])
        );
        assert_eq!(Object::from_json_value(value).unwrap(), object);
    }

    #[test]
    fn roundtrips_int_key_dict_values() {
        let mut entries = BTreeMap::new();
        entries.insert(DictKey::Int(1), Object::String("one".to_string()));
        entries.insert(DictKey::Int(2), Object::Bool(true));
        let object = Object::ValueDict(
            DictValue::new(DictKeyType::Int, entries).expect("keys should match"),
        );
        let value = object.to_json_value();

        assert_eq!(value, json!(["dict", "int", [[1, "^one"], [2, true]]]));
        assert_eq!(Object::from_json_value(value).unwrap(), object);
    }

    #[test]
    fn roundtrips_empty_dict_values_with_key_type() {
        let string_dict = Object::ValueDict(DictValue::empty(DictKeyType::String));
        let string_value = string_dict.to_json_value();

        assert_eq!(string_value, json!(["dict", "string", []]));
        assert_eq!(Object::from_json_value(string_value).unwrap(), string_dict);

        let int_dict = Object::ValueDict(DictValue::empty(DictKeyType::Int));
        let int_value = int_dict.to_json_value();

        assert_eq!(int_value, json!(["dict", "int", []]));
        assert_eq!(Object::from_json_value(int_value).unwrap(), int_dict);
    }

    #[test]
    fn roundtrips_nested_dict_values_with_arrays_and_objects() {
        let mut object_fields = BTreeMap::new();
        object_fields.insert("hp".to_string(), Object::Int(10));
        object_fields.insert(
            "tags".to_string(),
            Object::ValueArray(vec![
                Object::String("front".to_string()),
                Object::String("line".to_string()),
            ]),
        );

        let mut inner_entries = BTreeMap::new();
        inner_entries.insert(
            DictKey::String("player".to_string()),
            Object::ValueObject(object_fields),
        );
        let inner = Object::ValueDict(
            DictValue::new(DictKeyType::String, inner_entries).expect("keys should match"),
        );

        let mut outer_entries = BTreeMap::new();
        outer_entries.insert(
            DictKey::Int(7),
            Object::ValueArray(vec![Object::Int(1), inner]),
        );
        let object = Object::ValueDict(DictValue::new(DictKeyType::Int, outer_entries).unwrap());
        let value = object.to_json_value();

        assert_eq!(
            value,
            json!([
                "dict",
                "int",
                [[
                    7,
                    [
                        1,
                        [
                            "dict",
                            "string",
                            [[
                                "player",
                                {
                                    "hp": 10,
                                    "tags": ["^front", "^line"]
                                }
                            ]]
                        ]
                    ]
                ]]
            ])
        );
        assert_eq!(Object::from_json_value(value).unwrap(), object);
    }

    #[test]
    fn roundtrips_dynamic_object_values() {
        let mut fields = BTreeMap::new();
        fields.insert("hp".to_string(), Object::Int(10));
        fields.insert("name".to_string(), Object::String("Ada".to_string()));
        fields.insert(
            "flags".to_string(),
            Object::ValueArray(vec![Object::Bool(true), Object::Bool(false)]),
        );

        let object = Object::ValueObject(fields);
        let value = object.to_json_value();

        assert_eq!(
            value,
            json!({
                "flags": [true, false],
                "hp": 10,
                "name": "^Ada"
            })
        );
        assert_eq!(Object::from_json_value(value).unwrap(), object);
    }

    #[test]
    fn dict_marker_does_not_change_object_values() {
        let parsed = Object::from_json_value(json!({
            "dict": ["^not", "^marker"],
            "^dict": "^field"
        }))
        .unwrap();

        let mut fields = BTreeMap::new();
        fields.insert(
            "dict".to_string(),
            Object::ValueArray(vec![
                Object::String("not".to_string()),
                Object::String("marker".to_string()),
            ]),
        );
        fields.insert("^dict".to_string(), Object::String("field".to_string()));

        assert_eq!(parsed, Object::ValueObject(fields));
    }

    #[test]
    fn parses_json_arrays_without_container_terminators_as_values() {
        let parsed = Object::from_json_value(json!([1, 2, 3])).unwrap();

        assert_eq!(
            parsed,
            Object::ValueArray(vec![Object::Int(1), Object::Int(2), Object::Int(3)])
        );
    }

    #[test]
    fn still_parses_json_arrays_with_container_terminators_as_containers() {
        let parsed = Object::from_json_value(json!(["done", null])).unwrap();

        assert_eq!(
            parsed,
            Object::Container(Container::unnamed(vec![Object::ControlCommand(
                ControlCommand::Done
            )]))
        );
    }

    #[test]
    fn parses_arrays_of_objects_as_dynamic_values() {
        let parsed = Object::from_json_value(json!([{ "hp": 10 }])).unwrap();

        let mut fields = BTreeMap::new();
        fields.insert("hp".to_string(), Object::Int(10));
        assert_eq!(
            parsed,
            Object::ValueArray(vec![Object::ValueObject(fields)])
        );
    }

    #[test]
    fn parses_arrays_of_empty_objects_as_dynamic_values() {
        let parsed = Object::from_json_value(json!([{}, {}])).unwrap();

        assert_eq!(
            parsed,
            Object::ValueArray(vec![
                Object::ValueObject(BTreeMap::new()),
                Object::ValueObject(BTreeMap::new())
            ])
        );
    }
}
