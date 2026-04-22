use std::collections::BTreeMap;

use serde_json::{json, Map, Value};

use crate::{
    error::CompilerError,
    parsed::{ObjectKind, ObjectRef, Path, Story as ParsedStory},
};

pub fn export_story_json(story: &ParsedStory) -> Result<String, CompilerError> {
    let mut state = ExportState::default();
    for object in story.content() {
        let top_level_kind = object.borrow().kind().clone();
        export_object(&object, &mut state)?;
        state.last_top_level_kind = Some(top_level_kind);
    }

    if !matches!(
        state.last_top_level_kind,
        Some(ObjectKind::Choice { .. } | ObjectKind::Gather { .. })
    ) && !matches!(state.main_content.last(), Some(Value::String(value)) if value == "\n")
    {
        state.main_content.push(Value::String("\n".to_string()));
    }

    let mut inner_container = state.main_content;
    if !matches!(
        state.last_top_level_kind,
        Some(ObjectKind::Choice { .. } | ObjectKind::Gather { .. })
    ) {
        inner_container.push(Value::Null);
    }

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
    choice_index: usize,
    last_top_level_kind: Option<ObjectKind>,
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
        ObjectKind::Choice {
            once_only,
            is_invisible_default,
            has_start_content,
            has_choice_only_content,
            has_inline_inner_content,
            ..
        } => {
            export_choice_container(
                object,
                state,
                once_only,
                is_invisible_default,
                has_start_content,
                has_choice_only_content,
                has_inline_inner_content,
                tokens.len(),
                tokens,
            )?;
        }
        ObjectKind::Gather { .. }
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
        ObjectKind::AuthorWarning { .. } | ObjectKind::Tag { .. } | ObjectKind::Weave { .. } => {
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

fn export_choice_container(
    object: &ObjectRef,
    state: &mut ExportState,
    once_only: bool,
    is_invisible_default: bool,
    has_start_content: bool,
    has_choice_only_content: bool,
    has_inline_inner_content: bool,
    outer_index: usize,
    tokens: &mut Vec<Value>,
) -> Result<(), CompilerError> {
    let choice_index = state.choice_index;
    state.choice_index += 1;

    let (start_content, choice_only_content, inner_content) = {
        let children = object.borrow().content().to_vec();
        let mut iter = children.into_iter();

        let start_content = if has_start_content { iter.next() } else { None };
        let choice_only_content = if has_choice_only_content {
            iter.next()
        } else {
            None
        };
        let inner_content = iter.next();
        (start_content, choice_only_content, inner_content)
    };

    let mut outer_tokens = Vec::new();
    let mut choice_named_content = BTreeMap::new();

    if let Some(_start_content) = start_content.as_ref() {
        outer_tokens.push(json!("ev"));
        outer_tokens.push(json!({
            "^->": format!("0.{outer_index}.$r1")
        }));
        outer_tokens.push(json!({"temp=":"$r"}));
        outer_tokens.push(json!("str"));
        outer_tokens.push(json!({"->":".^.s"}));
        outer_tokens.push(Value::Array(vec![json!({"#n":"$r1"})]));
        outer_tokens.push(json!("/str"));
    }

    if let Some(choice_only_content) = choice_only_content.as_ref() {
        if !has_start_content {
            outer_tokens.push(json!("ev"));
        }
        outer_tokens.push(json!("str"));
        outer_tokens.extend(export_content_list_tokens(choice_only_content, state)?);
        outer_tokens.push(json!("/str"));
    }

    if has_start_content || has_choice_only_content {
        outer_tokens.push(json!("/ev"));
    }

    let flags = {
        let mut flags = 0;
        if has_start_content {
            flags |= 2;
        }
        if has_choice_only_content {
            flags |= 4;
        }
        if is_invisible_default {
            flags |= 8;
        }
        if once_only {
            flags |= 16;
        }
        flags
    };
    outer_tokens.push(json!({"*": format!("0.c-{choice_index}"), "flg": flags}));

    if let Some(start_content) = start_content {
        let mut s_tokens = Vec::new();
        s_tokens.extend(export_content_list_tokens(&start_content, state)?);
        s_tokens.push(json!({"->":"$r", "var":true}));
        s_tokens.push(Value::Null);
        outer_tokens.push(Value::Object(
            vec![("s".to_string(), Value::Array(s_tokens))]
                .into_iter()
                .collect::<Map<_, _>>(),
        ));
    }

    let mut inner_tokens = Vec::new();
    if has_start_content {
        inner_tokens.push(json!("ev"));
        inner_tokens.push(json!({
            "^->": format!("0.c-{choice_index}.$r2")
        }));
        inner_tokens.push(json!("/ev"));
        inner_tokens.push(json!({"temp=":"$r"}));
        inner_tokens.push(json!({
            "->": format!("0.{outer_index}.s")
        }));
        inner_tokens.push(Value::Array(vec![json!({"#n":"$r2"})]));
    }

    let mut body_tokens = if let Some(inner_content) = inner_content {
        export_content_list_tokens(&inner_content, state)?
    } else {
        Vec::new()
    };

    if !has_inline_inner_content
        && !matches!(body_tokens.first(), Some(Value::String(value)) if value == "\n")
    {
        body_tokens.insert(0, Value::String("\n".to_string()));
    }

    inner_tokens.extend(body_tokens);

    if !matches!(
        inner_tokens.last(),
        Some(Value::String(value)) if value == "done" || value == "end"
    ) {
        inner_tokens.push(Value::String("end".to_string()));
    }

    inner_tokens.push(json!({"->": format!("0.g-{choice_index}")}));
    inner_tokens.push(json!({"#f":5}));

    choice_named_content.insert(format!("c-{choice_index}"), Value::Array(inner_tokens));
    choice_named_content.insert(
        format!("g-{choice_index}"),
        Value::Array(vec![Value::String("done".to_string()), Value::Null]),
    );
    if has_start_content {
        tokens.push(Value::Array(outer_tokens));
    } else {
        tokens.extend(outer_tokens);
    }
    tokens.push(Value::Object(
        choice_named_content.into_iter().collect::<Map<_, _>>(),
    ));
    Ok(())
}

fn export_content_list_tokens(
    content_list: &ObjectRef,
    state: &mut ExportState,
) -> Result<Vec<Value>, CompilerError> {
    let children = {
        let borrowed = content_list.borrow();
        borrowed.content().to_vec()
    };

    let mut tokens = Vec::new();
    for child in children {
        export_object_into(&child, state, &mut tokens, false)?;
    }

    Ok(tokens)
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
