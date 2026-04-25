use ink_story_json_format as format;

use crate::{
    compiler::StageOutput,
    diagnostic::Diagnostic,
    lower::ir::{Container, ControlCommand, RuntimeObject, RuntimeProgram},
    source::SourceSpan,
};

pub(crate) const INK_VERSION_CURRENT: i32 = format::INK_VERSION_CURRENT;

pub(crate) fn emit_json(program: RuntimeProgram) -> StageOutput<String> {
    match program_to_format(&program).and_then(|program| {
        program
            .to_json_string()
            .map_err(|error| format!("failed to serialize compiled story JSON: {error}"))
    }) {
        Ok(json) => StageOutput {
            artifact: Some(json),
            diagnostics: Vec::new(),
        },
        Err(error) => StageOutput {
            artifact: None,
            diagnostics: vec![Diagnostic::error(SourceSpan::new(None, 1, 1), error)],
        },
    }
}

fn program_to_format(program: &RuntimeProgram) -> Result<format::Program, String> {
    Ok(format::Program {
        ink_version: INK_VERSION_CURRENT,
        root: container_to_format(&program.root)?,
        list_defs: serde_json::Map::new(),
    })
}

fn container_to_format(container: &Container) -> Result<format::Container, String> {
    let trailing_named_content = container.content.last().and_then(|object| match object {
        RuntimeObject::NamedContent(containers) if container.merge_tail_metadata => {
            Some(containers)
        }
        _ => None,
    });

    let content_objects = if trailing_named_content.is_some() {
        &container.content[..container.content.len() - 1]
    } else {
        &container.content
    };

    let mut content = Vec::with_capacity(content_objects.len());
    for object in content_objects {
        if matches!(object, RuntimeObject::NamedContent(_)) {
            return Err(
                "compiler lowering produced named content outside a container terminator"
                    .to_string(),
            );
        }
        content.push(object_to_format(object)?);
    }

    let mut named_content = Vec::new();
    if let Some(containers) = trailing_named_content {
        for container in containers {
            let Some(name) = &container.name else {
                continue;
            };
            named_content.push(format::NamedContainer::new(
                name.clone(),
                container_to_format(container)?,
            ));
        }
    }

    Ok(format::Container {
        content,
        named_content,
        name: container.name.clone(),
        flags: container.flags,
    })
}

fn object_to_format(object: &RuntimeObject) -> Result<format::Object, String> {
    Ok(match object {
        RuntimeObject::Container(container) => {
            format::Object::Container(container_to_format(container)?)
        }
        RuntimeObject::NamedContent(_) => {
            return Err("named content must be part of a container terminator".to_string())
        }
        RuntimeObject::String(text) => format::Object::Value(format::Value::String(text.clone())),
        RuntimeObject::ControlCommand(command) => {
            format::Object::ControlCommand(control_command_to_format(*command))
        }
        RuntimeObject::Divert { target, variable } => format::Object::Divert(format::Divert {
            kind: format::DivertKind::Direct,
            target: target.clone(),
            variable: *variable,
            conditional: false,
            external_args: None,
        }),
        RuntimeObject::TunnelDivert { target, variable } => {
            format::Object::Divert(format::Divert {
                kind: format::DivertKind::Tunnel,
                target: target.clone(),
                variable: *variable,
                conditional: false,
                external_args: None,
            })
        }
        RuntimeObject::FunctionDivert { target } => format::Object::Divert(format::Divert {
            kind: format::DivertKind::Function,
            target: target.clone(),
            variable: false,
            conditional: false,
            external_args: None,
        }),
        RuntimeObject::ExternalFunction { target, args } => {
            format::Object::Divert(format::Divert {
                kind: format::DivertKind::External,
                target: target.clone(),
                variable: false,
                conditional: false,
                external_args: Some(*args),
            })
        }
        RuntimeObject::ConditionalDivert { target } => format::Object::Divert(format::Divert {
            kind: format::DivertKind::Direct,
            target: target.clone(),
            variable: false,
            conditional: true,
            external_args: None,
        }),
        RuntimeObject::DivertTarget(target) => {
            format::Object::Value(format::Value::DivertTarget(target.clone()))
        }
        RuntimeObject::ReadCount(target) => {
            format::Object::VariableReference(format::VariableReference {
                kind: format::VariableReferenceKind::ReadCount,
                name: target.clone(),
            })
        }
        RuntimeObject::VariableAssignment(name) => {
            variable_assignment_to_format(format::VariableAssignmentKind::Temporary, name, true)
        }
        RuntimeObject::GlobalVariableAssignment(name) => {
            variable_assignment_to_format(format::VariableAssignmentKind::Global, name, true)
        }
        RuntimeObject::TempVariableReassignment(name) => {
            variable_assignment_to_format(format::VariableAssignmentKind::Temporary, name, false)
        }
        RuntimeObject::VariableReassignment(name) => {
            variable_assignment_to_format(format::VariableAssignmentKind::Global, name, false)
        }
        RuntimeObject::VariableReference(name) => {
            format::Object::VariableReference(format::VariableReference {
                kind: format::VariableReferenceKind::Variable,
                name: name.clone(),
            })
        }
        RuntimeObject::VariablePointer {
            name,
            context_index,
        } => format::Object::Value(format::Value::VariablePointer(format::VariablePointer {
            name: name.clone(),
            context_index: *context_index,
        })),
        RuntimeObject::ChoicePoint { target, flags } => {
            format::Object::ChoicePoint(format::ChoicePoint {
                target: target.clone(),
                flags: *flags,
            })
        }
        RuntimeObject::Glue => format::Object::Glue,
        RuntimeObject::Tag { is_start } => {
            let command = if *is_start {
                format::ControlCommand::BeginTag
            } else {
                format::ControlCommand::EndTag
            };
            format::Object::ControlCommand(command)
        }
        RuntimeObject::Bool(value) => format::Object::Value(format::Value::Bool(*value)),
        RuntimeObject::Int(value) => format::Object::Value(format::Value::Int(*value)),
        RuntimeObject::Float(value) => format::Object::Value(format::Value::Float(value.value())),
        RuntimeObject::Void => format::Object::Void,
        RuntimeObject::NativeFunction(name) => {
            format::Object::NativeFunction(format::NativeFunction::new(name.clone()))
        }
    })
}

fn control_command_to_format(command: ControlCommand) -> format::ControlCommand {
    match command {
        ControlCommand::Done => format::ControlCommand::Done,
        ControlCommand::End => format::ControlCommand::End,
        ControlCommand::EvalStart => format::ControlCommand::EvalStart,
        ControlCommand::EvalOutput => format::ControlCommand::EvalOutput,
        ControlCommand::EvalEnd => format::ControlCommand::EvalEnd,
        ControlCommand::BeginString => format::ControlCommand::BeginString,
        ControlCommand::EndString => format::ControlCommand::EndString,
        ControlCommand::VisitIndex => format::ControlCommand::VisitIndex,
        ControlCommand::SequenceShuffleIndex => format::ControlCommand::SequenceShuffleIndex,
        ControlCommand::Duplicate => format::ControlCommand::Duplicate,
        ControlCommand::NoOp => format::ControlCommand::NoOp,
        ControlCommand::Pop => format::ControlCommand::Pop,
        ControlCommand::PopFunction => format::ControlCommand::PopFunction,
        ControlCommand::PopTunnel => format::ControlCommand::PopTunnel,
        ControlCommand::StartThread => format::ControlCommand::StartThread,
        ControlCommand::ChoiceCount => format::ControlCommand::ChoiceCount,
        ControlCommand::Turns => format::ControlCommand::Turns,
        ControlCommand::TurnsSince => format::ControlCommand::TurnsSince,
        ControlCommand::ReadCount => format::ControlCommand::ReadCount,
        ControlCommand::Random => format::ControlCommand::Random,
        ControlCommand::SeedRandom => format::ControlCommand::SeedRandom,
        ControlCommand::ListRange => format::ControlCommand::ListRange,
        ControlCommand::ListRandom => format::ControlCommand::ListRandom,
    }
}

fn variable_assignment_to_format(
    kind: format::VariableAssignmentKind,
    name: &str,
    is_new_declaration: bool,
) -> format::Object {
    format::Object::VariableAssignment(format::VariableAssignment {
        kind,
        name: name.to_string(),
        is_new_declaration,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::lower::ir::{Container, RuntimeObject};

    #[test]
    fn emits_text_string_tokens() {
        let program = RuntimeProgram {
            root: Container {
                content: vec![RuntimeObject::String("Line.".to_string())],
                name: None,
                flags: None,
                merge_tail_metadata: true,
            },
        };

        let json = emit_json(program).artifact.expect("expected emitted json");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(
            value,
            json!({
                "inkVersion": 21,
                "root": ["^Line.", null],
                "listDefs": {}
            })
        );
    }

    #[test]
    fn emits_void_token() {
        let program = RuntimeProgram {
            root: Container {
                content: vec![RuntimeObject::Void],
                name: None,
                flags: None,
                merge_tail_metadata: true,
            },
        };

        let json = emit_json(program).artifact.expect("expected emitted json");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["root"][0], json!("void"));
    }
}
