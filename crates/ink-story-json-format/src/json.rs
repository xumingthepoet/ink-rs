use std::collections::BTreeMap;

use serde_json::{Map, Number, Value as JsonValue};

use crate::{Container, FormatError, NamedContainer, Object, Program};

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

    Ok(Program { ink_version, root })
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
        Object::Void => JsonValue::String("void".to_string()),
        Object::NativeFunction(name) => JsonValue::String(name.clone()),
    }
}

fn array_object_from_value(value: &JsonValue) -> Result<Object, FormatError> {
    match container_from_value(value, None) {
        Ok(container) => Ok(Object::Container(container)),
        Err(container_error) => match value {
            JsonValue::Array(values) => {
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

    Ok(Object::NativeFunction(token.to_string()))
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
