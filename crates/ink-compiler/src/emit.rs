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
    container_to_value_with_name(container, true)
}

fn container_to_value_without_name(container: &Container) -> Value {
    container_to_value_with_name(container, false)
}

fn container_to_value_with_name(container: &Container, include_name: bool) -> Value {
    let named_content_tail = container
        .content
        .last()
        .is_some_and(|object| matches!(object, RuntimeObject::NamedContent(_)));
    let merge_named_content_tail = named_content_tail && container.merge_tail_metadata;
    let value_content = if merge_named_content_tail {
        &container.content[..container.content.len() - 1]
    } else {
        &container.content
    };
    let mut values = value_content
        .iter()
        .map(runtime_object_to_value)
        .collect::<Vec<_>>();

    if merge_named_content_tail {
        if let Some(RuntimeObject::NamedContent(containers)) = container.content.last() {
            values.push(named_content_to_value_with_metadata(
                containers,
                container,
                include_name,
            ));
        }
    } else {
        values.push(container_terminator_to_value(container, include_name));
    }
    Value::Array(values)
}

fn container_terminator_to_value(container: &Container, include_name: bool) -> Value {
    let mut obj = Map::new();

    if let Some(flags) = container.flags {
        obj.insert("#f".to_string(), Value::Number(flags.into()));
    }

    if include_name {
        if let Some(name) = &container.name {
            obj.insert("#n".to_string(), Value::String(name.clone()));
        }
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
        RuntimeObject::NamedContent(containers) => named_content_to_value(containers),
        RuntimeObject::String(text) if text == "\n" => Value::String("\n".to_string()),
        RuntimeObject::String(text) => Value::String(format!("^{text}")),
        RuntimeObject::Glue => Value::String("<>".to_string()),
        RuntimeObject::ControlCommand(command) => Value::String(
            match command {
                ControlCommand::Done => "done",
                ControlCommand::End => "end",
                ControlCommand::EvalStart => "ev",
                ControlCommand::EvalOutput => "out",
                ControlCommand::EvalEnd => "/ev",
                ControlCommand::BeginString => "str",
                ControlCommand::EndString => "/str",
                ControlCommand::VisitIndex => "visit",
                ControlCommand::SequenceShuffleIndex => "seq",
                ControlCommand::Duplicate => "du",
                ControlCommand::NoOp => "nop",
                ControlCommand::Pop => "pop",
            }
            .to_string(),
        ),
        RuntimeObject::Tag { is_start } => {
            Value::String(if *is_start { "#" } else { "/#" }.to_string())
        }
        RuntimeObject::Bool(value) => Value::Bool(*value),
        RuntimeObject::Int(value) => Value::Number((*value).into()),
        RuntimeObject::Float(value) => Value::Number(
            serde_json::Number::from_f64(value.value()).expect("finite ink float literal"),
        ),
        RuntimeObject::NativeFunction(name) => Value::String(name.clone()),
        RuntimeObject::ConditionalDivert { target } => {
            let mut obj = Map::new();
            obj.insert("->".to_string(), Value::String(target.clone()));
            obj.insert("c".to_string(), Value::Bool(true));
            Value::Object(obj)
        }
        RuntimeObject::Divert { target, variable } => {
            let mut obj = Map::new();
            obj.insert("->".to_string(), Value::String(target.clone()));
            if *variable {
                obj.insert("var".to_string(), Value::Bool(true));
            }
            Value::Object(obj)
        }
        RuntimeObject::DivertTarget(target) => json!({ "^->": target }),
        RuntimeObject::ReadCount(target) => json!({ "CNT?": target }),
        RuntimeObject::VariableAssignment(name) => json!({ "temp=": name }),
        RuntimeObject::GlobalVariableAssignment(name) => json!({ "VAR=": name }),
        RuntimeObject::VariableReassignment(name) => json!({ "VAR=": name, "re": true }),
        RuntimeObject::VariableReference(name) => json!({ "VAR?": name }),
        RuntimeObject::ChoicePoint { target, flags } => json!({ "*": target, "flg": flags }),
    }
}

fn named_content_to_value(containers: &[Container]) -> Value {
    named_content_to_value_with_extra(containers, Map::new())
}

fn named_content_to_value_with_metadata(
    containers: &[Container],
    container: &Container,
    include_name: bool,
) -> Value {
    let extra = match container_terminator_to_value(container, include_name) {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    named_content_to_value_with_extra(containers, extra)
}

fn named_content_to_value_with_extra(
    containers: &[Container],
    mut obj: Map<String, Value>,
) -> Value {
    for container in containers {
        let Some(name) = &container.name else {
            continue;
        };
        obj.insert(name.clone(), container_to_value_without_name(container));
    }
    Value::Object(obj)
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
            merge_tail_metadata: true,
        };
        assert_eq!(container_to_value(&container), json!(["^Line.", null]));
    }
}
