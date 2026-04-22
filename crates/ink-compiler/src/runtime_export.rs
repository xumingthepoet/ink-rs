use std::collections::BTreeMap;

use serde_json::{json, Map, Value};

use crate::{
    error::CompilerError,
    parsed::{ObjectKind, ObjectRef, Story as ParsedStory},
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

    let root = json!([Value::Array(inner_container), "done", null]);
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
    list_defs: BTreeMap<String, BTreeMap<String, i64>>,
}

fn export_object(object: &ObjectRef, state: &mut ExportState) -> Result<(), CompilerError> {
    let borrowed = object.borrow();

    match borrowed.kind() {
        ObjectKind::ContentList { .. } => {
            for child in borrowed.content() {
                export_object(&child, state)?;
            }
        }
        ObjectKind::Text { text } => state.main_content.push(export_text_token(text)),
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
        | ObjectKind::Divert { .. }
        | ObjectKind::Weave { .. }
        | ObjectKind::Choice { .. }
        | ObjectKind::Gather { .. }
        | ObjectKind::Sequence { .. }
        | ObjectKind::Conditional
        | ObjectKind::ConditionalSingleBranch { .. }
        | ObjectKind::ConstantDeclaration { .. }
        | ObjectKind::ExternalDeclaration { .. }
        | ObjectKind::Expression { .. }
        | ObjectKind::Flow { .. }
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

fn export_text_token(text: &str) -> Value {
    if text == "\n" {
        json!("\n")
    } else {
        json!(format!("^{}", text))
    }
}

#[cfg(test)]
mod tests {
    use bladeink::story::Story as RuntimeStory;

    use crate::parsed::{
        ContentList, Divert, Identifier, ListDefinition, ListElementDefinition, Story, Text,
        VariableAssignment,
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
        let story = Story::new(vec![Divert::empty().object()], false);

        assert!(export_story_json(&story).is_err());
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
