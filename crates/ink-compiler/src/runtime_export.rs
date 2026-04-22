use std::collections::BTreeMap;

use serde_json::{json, Map, Value};

use crate::{
    error::CompilerError,
    parsed::{ObjectKind, ObjectRef, Path, Story as ParsedStory},
};

pub fn export_story_json(story: &ParsedStory) -> Result<String, CompilerError> {
    let mut state = ExportState::default();
    for object in story.content() {
        export_object(&object, &mut state)?;
    }

    if !matches!(state.main_content.last(), Some(Value::String(value)) if value == "\n") {
        state.main_content.push(Value::String("\n".to_string()));
    }

    let mut inner_container = state.main_content;
    inner_container.push(Value::Null);

    let named_containers = if state.named_containers.is_empty() {
        Value::Null
    } else {
        Value::Object(
            state
                .named_containers
                .into_iter()
                .collect::<serde_json::Map<_, _>>(),
        )
    };

    let root = json!([Value::Array(inner_container), "done", named_containers]);
    let list_defs = build_list_defs_json(state.list_defs);
    let story_json = json!({
        "inkVersion": 21,
        "root": root,
        "listDefs": list_defs,
    });

    Ok(story_json.to_string())
}

#[derive(Default)]
struct ExportState {
    main_content: Vec<Value>,
    named_containers: BTreeMap<String, Value>,
    list_defs: BTreeMap<String, BTreeMap<String, i64>>,
}

fn export_object(object: &ObjectRef, state: &mut ExportState) -> Result<(), CompilerError> {
    let mut main_content = std::mem::take(&mut state.main_content);
    let result = export_object_into(object, state, &mut main_content, true);
    state.main_content = main_content;
    result
}

fn export_object_into(
    object: &ObjectRef,
    state: &mut ExportState,
    tokens: &mut Vec<Value>,
    allow_named_flow: bool,
) -> Result<(), CompilerError> {
    let (kind, content) = {
        let borrowed = object.borrow();
        (borrowed.kind().clone(), borrowed.content().to_vec())
    };

    match kind {
        ObjectKind::ContentList { .. } => {
            for child in content {
                export_object_into(&child, state, tokens, allow_named_flow)?;
            }
        }
        ObjectKind::Text { text } => tokens.extend(export_text_tokens(&text)),
        ObjectKind::Divert {
            target, is_tunnel, ..
        } => {
            if target.is_none() {
                return Err(CompilerError::Unsupported(
                    "runtime export currently supports plain text stories, terminal diverts, and list definitions only",
                ));
            }
            tokens.push(export_divert_token(target.as_ref(), is_tunnel));
        }
        ObjectKind::Flow { name, .. } => {
            if allow_named_flow {
                if let Some(name) = name {
                    let container = export_named_flow_container(&content, state)?;
                    state.named_containers.insert(name.clone(), container);
                } else {
                    for child in content {
                        export_object_into(&child, state, tokens, false)?;
                    }
                }
            } else {
                for child in content {
                    export_object_into(&child, state, tokens, false)?;
                }
            }
        }
        ObjectKind::VariableAssignment { .. } => {
            let assignment = crate::parsed::VariableAssignment::from_object(object.clone());
            if let Some(list_definition) = assignment.list_definition() {
                if let Some(identifier) = list_definition.identifier() {
                    state.list_defs.insert(
                        identifier.name,
                        list_definition
                            .runtime_items()
                            .into_iter()
                            .collect::<BTreeMap<_, _>>(),
                    );
                }
            } else {
                return Err(CompilerError::Unsupported(
                    "runtime export currently supports plain text and list definitions only",
                ));
            }
        }
        ObjectKind::Return => {
            return Err(CompilerError::Unsupported(
                "runtime export currently supports plain text and list definitions only",
            ));
        }
        ObjectKind::AuthorWarning { .. }
        | ObjectKind::Tag { .. }
        | ObjectKind::Weave { .. }
        | ObjectKind::Choice { .. }
        | ObjectKind::Gather { .. }
        | ObjectKind::Sequence { .. }
        | ObjectKind::Conditional
        | ObjectKind::ConditionalSingleBranch { .. }
        | ObjectKind::ConstantDeclaration { .. }
        | ObjectKind::ExternalDeclaration { .. }
        | ObjectKind::Expression { .. }
        | ObjectKind::ListDefinition { .. }
        | ObjectKind::ListElementDefinition { .. }
        | ObjectKind::Generic => {
            return Err(CompilerError::Unsupported(
                "runtime export currently supports plain text stories and list definitions only",
            ));
        }
    }

    Ok(())
}

fn export_named_flow_container(
    flow_content: &[ObjectRef],
    state: &mut ExportState,
) -> Result<Value, CompilerError> {
    let mut tokens = Vec::new();
    for child in flow_content {
        export_object_into(child, state, &mut tokens, false)?;
    }

    if !matches!(tokens.last(), Some(Value::String(value)) if value == "\n") {
        tokens.push(Value::String("\n".to_string()));
    }

    tokens.push(Value::String("end".to_string()));
    tokens.push(Value::Null);

    Ok(Value::Array(tokens))
}

fn build_list_defs_json(list_defs: BTreeMap<String, BTreeMap<String, i64>>) -> Value {
    let mut defs = Map::new();
    for (list_name, items) in list_defs {
        let mut item_defs = Map::new();
        for (item_name, value) in items {
            item_defs.insert(item_name, json!(value));
        }
        defs.insert(list_name, Value::Object(item_defs));
    }

    Value::Object(defs)
}

fn export_divert_token(target: Option<&Path>, is_tunnel: bool) -> Value {
    if let Some(target) = target {
        if target.dot_separated_components().as_deref() == Some("END") {
            return json!("end");
        }

        if target.dot_separated_components().as_deref() == Some("DONE") {
            return json!("done");
        }
    }

    let divert_key = if is_tunnel { "->t->" } else { "->" };
    let target = target
        .and_then(|path| path.dot_separated_components())
        .unwrap_or_default();
    json!({ divert_key: target })
}

fn export_text_token(text: &str) -> Value {
    if text == "\n" {
        json!("\n")
    } else {
        json!(format!("^{}", text))
    }
}

fn export_text_tokens(text: &str) -> Vec<Value> {
    if !text.contains("<>") {
        return vec![export_text_token(text)];
    }

    let mut tokens = Vec::new();
    let mut remainder = text;

    while let Some(index) = remainder.find("<>") {
        let prefix = &remainder[..index];
        if !prefix.is_empty() {
            tokens.push(export_text_token(prefix));
        }
        tokens.push(json!("<>"));
        remainder = &remainder[index + 2..];
    }

    if !remainder.is_empty() {
        tokens.push(export_text_token(remainder));
    }

    tokens
}

#[cfg(test)]
mod tests {
    use bladeink::story::Story as RuntimeStory;

    use crate::parsed::{
        Choice, ContentList, Divert, Identifier, Knot, ListDefinition, ListElementDefinition, Path,
        Story, Text, VariableAssignment,
    };

    use super::export_story_json;

    #[test]
    fn exports_minimal_plain_text_story_json() {
        let line = ContentList::new();
        line.add_content(Text::new("Hello").object());
        line.add_content(Text::new("\n").object());
        let story = Story::new(vec![line.object()], false);

        let json = export_story_json(&story).expect("expected plain text story export");
        assert!(json.contains("\"inkVersion\":21"));
        assert!(json.contains("\"root\""));
        assert!(json.contains("\"listDefs\":{}"));
        assert!(json.contains("^Hello"));
        assert!(json.contains("\"done\""));
    }

    #[test]
    fn exports_plain_text_story_with_terminal_newline() {
        let line = ContentList::new();
        line.add_content(Text::new("Hello").object());
        let story = Story::new(vec![line.object()], false);

        let json = export_story_json(&story).expect("expected plain text story export");
        let runtime_story = RuntimeStory::new(&json).expect("expected runtime story to load");

        let mut output = String::new();
        let mut story = runtime_story;
        while story.can_continue() {
            output.push_str(&story.cont().expect("continue story"));
        }

        assert_eq!(output, "Hello\n");
    }

    #[test]
    fn rejects_non_text_nodes() {
        let story = Story::new(vec![Choice::new(None, 1).object()], false);

        assert!(export_story_json(&story).is_err());
    }

    #[test]
    fn exports_diverts_and_named_flows() {
        let line = ContentList::new();
        line.add_content(Text::new("We arrived into London at 9.45pm exactly.").object());
        line.add_content(Text::new("\n").object());
        line.add_content(
            Divert::new(Some(Path::new(vec![Identifier::new("hurry_home")]))).object(),
        );
        line.add_content(Text::new("\n").object());

        let knot = Knot::new(
            Identifier::new("hurry_home"),
            vec![
                Text::new("We hurried home to Savile Row as fast as we could.").object(),
                Text::new("\n").object(),
            ],
            Vec::new(),
            false,
        );
        let story = Story::new(vec![line.object(), knot.object()], false);

        let json = export_story_json(&story).expect("expected divert story export");
        assert!(json.contains("\"->\":\"hurry_home\""));
        assert!(json.contains("\"hurry_home\""));
        assert!(json.contains("\"end\""));

        let mut runtime_story = RuntimeStory::new(&json).expect("expected runtime story to load");
        let mut output = String::new();
        while runtime_story.can_continue() {
            output.push_str(&runtime_story.cont().expect("continue story"));
        }

        assert!(output.contains("We arrived into London at 9.45pm exactly."));
        assert!(output.contains("We hurried home to Savile Row as fast as we could."));
    }

    #[test]
    fn exports_glue_tokens_and_loads_runtime_story() {
        let line1 = ContentList::new();
        line1.add_content(Text::new("Some <>").object());
        line1.add_content(Text::new("\n").object());

        let line2 = ContentList::new();
        line2.add_content(Text::new("content <>").object());
        line2.add_content(Text::new("\n").object());

        let line3 = ContentList::new();
        line3.add_content(Text::new("with glue.").object());
        line3.add_content(Text::new("\n").object());

        let story = Story::new(vec![line1.object(), line2.object(), line3.object()], false);

        let json = export_story_json(&story).expect("expected glue story export");
        assert!(json.contains("\"<>\""));

        let mut runtime_story = RuntimeStory::new(&json).expect("expected runtime story to load");
        let mut output = String::new();
        while runtime_story.can_continue() {
            output.push_str(&runtime_story.cont().expect("continue story"));
        }

        assert_eq!(output, "Some content with glue.\n");
    }

    #[test]
    fn exports_lists_definitions_and_loads_runtime_story() {
        let list_definition = ListDefinition::new(
            Identifier::new("terrain"),
            vec![
                ListElementDefinition::new(Identifier::new("forest"), true, None, 1),
                ListElementDefinition::new(Identifier::new("hill"), false, Some(4), 4),
            ],
        );
        let declaration = VariableAssignment::new_with_list_definition(
            Identifier::new("terrain"),
            list_definition,
            true,
        );

        let line = ContentList::new();
        line.add_content(Text::new("Hello").object());
        line.add_content(Text::new("\n").object());
        let story = Story::new(vec![declaration.object(), line.object()], false);

        let json = export_story_json(&story).expect("expected list definition export");
        assert!(json.contains("\"listDefs\""));
        assert!(json.contains("\"terrain\""));
        assert!(json.contains("\"forest\""));
        assert!(json.contains("\"hill\""));

        let _runtime_story = RuntimeStory::new(&json).expect("expected runtime story to load");
    }
}
