use serde_json::{Map, Number, Value as JsonValue};

use crate::{
    ChoicePoint, Container, Divert, DivertKind, FormatError, ListItemValue, ListValue,
    NamedContainer, NativeFunction, Object, Program, Value, VariableAssignment,
    VariableAssignmentKind, VariablePointer, VariableReference, VariableReferenceKind,
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
    let list_defs = match obj.get("listDefs") {
        Some(JsonValue::Object(list_defs)) => list_defs.clone(),
        Some(_) => return Err(FormatError::new("listDefs must be an object")),
        None => Map::new(),
    };

    Ok(Program {
        ink_version,
        root,
        list_defs,
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
    obj.insert(
        "listDefs".to_string(),
        JsonValue::Object(program.list_defs.clone()),
    );
    JsonValue::Object(obj)
}

fn container_from_value(
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

fn container_to_value(container: &Container, include_name: bool) -> JsonValue {
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

fn object_from_value(value: &JsonValue) -> Result<Object, FormatError> {
    match value {
        JsonValue::Null => Err(FormatError::new(
            "null is only valid as a container terminator",
        )),
        JsonValue::Bool(value) => Ok(Object::Value(Value::Bool(*value))),
        JsonValue::Number(number) => {
            if let Some(value) = number.as_i64() {
                let value = i32::try_from(value)
                    .map_err(|_| FormatError::new("integer value is outside i32 range"))?;
                Ok(Object::Value(Value::Int(value)))
            } else {
                let value = number
                    .as_f64()
                    .ok_or_else(|| FormatError::new("number value is not representable as f64"))?;
                Ok(Object::Value(Value::Float(value)))
            }
        }
        JsonValue::String(value) => string_object_from_token(value),
        JsonValue::Array(_) => Ok(Object::Container(container_from_value(value, None)?)),
        JsonValue::Object(obj) => object_from_map(obj),
    }
}

fn object_to_value(object: &Object) -> JsonValue {
    match object {
        Object::Container(container) => container_to_value(container, true),
        Object::Value(value) => value_to_json(value),
        Object::ControlCommand(command) => JsonValue::String(command.token().to_string()),
        Object::NativeFunction(function) => JsonValue::String(function.token().to_string()),
        Object::Divert(divert) => divert_to_value(divert),
        Object::ChoicePoint(choice) => {
            let mut obj = Map::new();
            obj.insert("*".to_string(), JsonValue::String(choice.target.clone()));
            obj.insert("flg".to_string(), JsonValue::Number(choice.flags.into()));
            JsonValue::Object(obj)
        }
        Object::VariableAssignment(assignment) => variable_assignment_to_value(assignment),
        Object::VariableReference(reference) => variable_reference_to_value(reference),
        Object::Glue => JsonValue::String("<>".to_string()),
        Object::Void => JsonValue::String("void".to_string()),
    }
}

fn string_object_from_token(token: &str) -> Result<Object, FormatError> {
    if let Some(text) = token.strip_prefix('^') {
        return Ok(Object::Value(Value::String(text.to_string())));
    }

    if token == "\n" {
        return Ok(Object::Value(Value::String("\n".to_string())));
    }

    if token == "<>" {
        return Ok(Object::Glue);
    }

    if token == "void" {
        return Ok(Object::Void);
    }

    if let Some(command) = crate::ControlCommand::from_token(token) {
        return Ok(Object::ControlCommand(command));
    }

    Ok(Object::NativeFunction(NativeFunction::from_token(token)))
}

fn value_to_json(value: &Value) -> JsonValue {
    match value {
        Value::String(text) if text == "\n" => JsonValue::String("\n".to_string()),
        Value::String(text) => JsonValue::String(format!("^{text}")),
        Value::Bool(value) => JsonValue::Bool(*value),
        Value::Int(value) => JsonValue::Number((*value).into()),
        Value::Float(value) => JsonValue::Number(
            Number::from_f64(*value).expect("compiled story float values must be finite"),
        ),
        Value::DivertTarget(target) => {
            let mut obj = Map::new();
            obj.insert("^->".to_string(), JsonValue::String(target.clone()));
            JsonValue::Object(obj)
        }
        Value::VariablePointer(pointer) => {
            let mut obj = Map::new();
            obj.insert("^var".to_string(), JsonValue::String(pointer.name.clone()));
            obj.insert(
                "ci".to_string(),
                JsonValue::Number(pointer.context_index.into()),
            );
            JsonValue::Object(obj)
        }
        Value::List(value) => list_value_to_json(value),
    }
}

fn object_from_map(obj: &Map<String, JsonValue>) -> Result<Object, FormatError> {
    if let Some(target) = obj.get("^->") {
        return Ok(Object::Value(Value::DivertTarget(
            json_value_to_string(target, "^->")?.to_string(),
        )));
    }

    if let Some(name) = obj.get("^var") {
        let context_index = match obj.get("ci") {
            Some(value) => json_value_to_i32(value, "ci")?,
            None => -1,
        };
        return Ok(Object::Value(Value::VariablePointer(VariablePointer {
            name: json_value_to_string(name, "^var")?.to_string(),
            context_index,
        })));
    }

    if let Some(divert) = divert_from_map(obj)? {
        return Ok(Object::Divert(divert));
    }

    if let Some(target) = obj.get("*") {
        let flags = match obj.get("flg") {
            Some(value) => json_value_to_i32(value, "flg")?,
            None => 0,
        };
        return Ok(Object::ChoicePoint(ChoicePoint {
            target: json_value_to_string(target, "*")?.to_string(),
            flags,
        }));
    }

    if let Some(name) = obj.get("VAR?") {
        return Ok(Object::VariableReference(VariableReference {
            kind: VariableReferenceKind::Variable,
            name: json_value_to_string(name, "VAR?")?.to_string(),
        }));
    }

    if let Some(name) = obj.get("CNT?") {
        return Ok(Object::VariableReference(VariableReference {
            kind: VariableReferenceKind::ReadCount,
            name: json_value_to_string(name, "CNT?")?.to_string(),
        }));
    }

    if let Some(assignment) = variable_assignment_from_map(obj)? {
        return Ok(Object::VariableAssignment(assignment));
    }

    if obj.contains_key("list") {
        return Ok(Object::Value(Value::List(list_value_from_map(obj)?)));
    }

    Err(FormatError::new(format!(
        "unrecognized compiled story object: {}",
        JsonValue::Object(obj.clone())
    )))
}

fn divert_from_map(obj: &Map<String, JsonValue>) -> Result<Option<Divert>, FormatError> {
    let keys = [
        (DivertKind::Direct, "->"),
        (DivertKind::Function, "f()"),
        (DivertKind::Tunnel, "->t->"),
        (DivertKind::External, "x()"),
    ];

    for (kind, key) in keys {
        if let Some(target) = obj.get(key) {
            let external_args = match obj.get("exArgs") {
                Some(value) => Some(json_value_to_usize(value, "exArgs")?),
                None => None,
            };
            return Ok(Some(Divert {
                kind,
                target: json_value_to_string(target, key)?.to_string(),
                variable: obj.contains_key("var"),
                conditional: obj.contains_key("c"),
                external_args,
            }));
        }
    }

    Ok(None)
}

fn divert_to_value(divert: &Divert) -> JsonValue {
    let mut obj = Map::new();
    obj.insert(
        divert.kind.key().to_string(),
        JsonValue::String(divert.target.clone()),
    );
    if divert.variable {
        obj.insert("var".to_string(), JsonValue::Bool(true));
    }
    if divert.conditional {
        obj.insert("c".to_string(), JsonValue::Bool(true));
    }
    if let Some(external_args) = divert.external_args {
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
) -> Result<Option<VariableAssignment>, FormatError> {
    let Some((kind, name)) = obj
        .get("VAR=")
        .map(|name| (VariableAssignmentKind::Global, name))
        .or_else(|| {
            obj.get("temp=")
                .map(|name| (VariableAssignmentKind::Temporary, name))
        })
    else {
        return Ok(None);
    };

    Ok(Some(VariableAssignment {
        kind,
        name: json_value_to_string(name, "variable assignment")?.to_string(),
        is_new_declaration: !obj.contains_key("re"),
    }))
}

fn variable_assignment_to_value(assignment: &VariableAssignment) -> JsonValue {
    let mut obj = Map::new();
    let key = match assignment.kind {
        VariableAssignmentKind::Global => "VAR=",
        VariableAssignmentKind::Temporary => "temp=",
    };
    obj.insert(key.to_string(), JsonValue::String(assignment.name.clone()));
    if !assignment.is_new_declaration {
        obj.insert("re".to_string(), JsonValue::Bool(true));
    }
    JsonValue::Object(obj)
}

fn variable_reference_to_value(reference: &VariableReference) -> JsonValue {
    let mut obj = Map::new();
    let key = match reference.kind {
        VariableReferenceKind::Variable => "VAR?",
        VariableReferenceKind::ReadCount => "CNT?",
    };
    obj.insert(key.to_string(), JsonValue::String(reference.name.clone()));
    JsonValue::Object(obj)
}

fn list_value_from_map(obj: &Map<String, JsonValue>) -> Result<ListValue, FormatError> {
    let list = obj
        .get("list")
        .ok_or_else(|| FormatError::new("list value must include list"))?
        .as_object()
        .ok_or_else(|| FormatError::new("list must be an object"))?;

    let mut items = Vec::with_capacity(list.len());
    for (name, value) in list {
        items.push(ListItemValue {
            name: name.clone(),
            value: json_value_to_i32(value, "list item value")?,
        });
    }

    let origins = match obj.get("origins") {
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(|value| json_value_to_string(value, "list origin").map(str::to_string))
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err(FormatError::new("origins must be an array")),
        None => Vec::new(),
    };

    Ok(ListValue { items, origins })
}

fn list_value_to_json(value: &ListValue) -> JsonValue {
    let mut obj = Map::new();
    let mut items = Map::new();
    for item in &value.items {
        items.insert(item.name.clone(), JsonValue::Number(item.value.into()));
    }
    obj.insert("list".to_string(), JsonValue::Object(items));
    if !value.origins.is_empty() {
        obj.insert(
            "origins".to_string(),
            JsonValue::Array(
                value
                    .origins
                    .iter()
                    .map(|origin| JsonValue::String(origin.clone()))
                    .collect(),
            ),
        );
    }
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
    use serde_json::json;

    use super::*;
    use crate::{ControlCommand, NativeFunction};

    #[test]
    fn writes_plain_text_story_json() {
        let program = Program::new(Container::new(vec![
            Object::Container(Container::new(vec![
                Object::Value(Value::String("Line.".to_string())),
                Object::Value(Value::String("\n".to_string())),
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
                "inkVersion": 21,
                "root": [["^Line.", "\n", ["done", {"#n": "g-0"}], null], "done", null],
                "listDefs": {}
            })
        );
    }

    #[test]
    fn roundtrips_named_content_and_command_tokens() {
        let input = json!({
            "inkVersion": 21,
            "root": [
                ["#", "^tag", "/#", {"->t->": "knot"}, {"#n": "g-0"}],
                "done",
                {
                    "knot": ["ev", "str", "^value", "/str", "/ev", "end", {"#f": 1}],
                    "global decl": ["ev", 2, {"VAR=": "x"}, "/ev", "end", null]
                }
            ],
            "listDefs": {}
        });

        let program = program_from_value(input.clone()).expect("format should parse");

        assert_eq!(program_to_value(&program), input);
    }

    #[test]
    fn roundtrips_list_values_and_native_tokens() {
        let input = json!({
            "inkVersion": 21,
            "root": [
                "ev",
                {"list": {"list.a": 1, "list.c": 3}, "origins": ["list"]},
                "LIST_ALL",
                "L^",
                "/ev",
                "done",
                null
            ],
            "listDefs": {
                "list": {"a": 1, "b": 2, "c": 3}
            }
        });

        let program = program_from_value(input.clone()).expect("format should parse");
        assert_eq!(program_to_value(&program), input);

        let native_tokens = program
            .root
            .content
            .iter()
            .filter_map(|object| match object {
                Object::NativeFunction(function) => Some(function.name()),
                _ => None,
            });
        assert_eq!(native_tokens.collect::<Vec<_>>(), vec!["LIST_ALL", "^"]);
    }

    #[test]
    fn native_function_escapes_caret_token() {
        let function = NativeFunction::new("^");
        assert_eq!(function.token(), "L^");
        assert_eq!(NativeFunction::from_token("L^").name(), "^");
    }
}
