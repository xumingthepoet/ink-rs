use serde_json::{json, Map, Value};

use crate::{
    compiler::StageOutput,
    diagnostic::Diagnostic,
    lower::{Container, ControlCommand, RuntimeObject, RuntimeProgram},
    source::SourceSpan,
};

pub(crate) const INK_VERSION_CURRENT: i32 = 21;

pub(crate) fn emit_json(program: RuntimeProgram) -> StageOutput<String> {
    let value = json!({
        "inkVersion": INK_VERSION_CURRENT,
        "root": container_to_value(&program.root),
        "listDefs": {},
    });

    match serde_json::to_string(&value) {
        Ok(json) => StageOutput {
            artifact: Some(json),
            diagnostics: Vec::new(),
        },
        Err(error) => StageOutput {
            artifact: None,
            diagnostics: vec![Diagnostic::error(
                SourceSpan::new(None, 1, 1),
                format!("failed to serialize compiled story JSON: {error}"),
            )],
        },
    }
}

fn container_to_value(container: &Container) -> Value {
    let mut values = container
        .content
        .iter()
        .map(runtime_object_to_value)
        .collect::<Vec<_>>();

    values.push(container_terminator_to_value(container));
    Value::Array(values)
}

fn container_terminator_to_value(container: &Container) -> Value {
    let mut obj = Map::new();

    if let Some(flags) = container.flags {
        obj.insert("#f".to_string(), Value::Number(flags.into()));
    }

    if let Some(name) = &container.name {
        obj.insert("#n".to_string(), Value::String(name.clone()));
    }

    if obj.is_empty() {
        Value::Null
    } else {
        Value::Object(obj)
    }
}

fn runtime_object_to_value(object: &RuntimeObject) -> Value {
    match object {
        RuntimeObject::Container(container) => container_to_value(container),
        RuntimeObject::String(text) if text == "\n" => Value::String("\n".to_string()),
        RuntimeObject::String(text) => Value::String(format!("^{text}")),
        RuntimeObject::ControlCommand(command) => Value::String(
            match command {
                ControlCommand::Done => "done",
            }
            .to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lower::{Container, RuntimeObject};

    #[test]
    fn emits_text_string_tokens() {
        let container = Container {
            content: vec![RuntimeObject::String("Line.".to_string())],
            name: None,
            flags: None,
        };
        assert_eq!(container_to_value(&container), json!(["^Line.", null]));
    }
}
