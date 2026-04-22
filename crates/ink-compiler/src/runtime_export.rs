use std::collections::BTreeMap;

use serde_json::{json, Map, Value};

use crate::{
    error::CompilerError,
    parsed::{ExpressionKind, ObjectKind, ObjectRef, Path, SequenceType, Story as ParsedStory},
};

pub fn export_story_json(story: &ParsedStory) -> Result<String, CompilerError> {
    let mut state = ExportState::default();
    let top_level_content = story.content();
    let top_level_has_weave_points = top_level_content.iter().any(|object| {
        matches!(
            object.borrow().kind(),
            ObjectKind::Choice { .. } | ObjectKind::Gather { .. }
        )
    });
    state.last_top_level_kind = top_level_content
        .last()
        .map(|object| object.borrow().kind().clone());
    let mut main_content = Vec::new();
    export_children(top_level_content, &mut state, &mut main_content, true, None)?;
    state.main_content = main_content;

    if !top_level_has_weave_points
        && !matches!(
            state.last_top_level_kind,
            Some(ObjectKind::Choice { .. } | ObjectKind::Gather { .. })
        )
        && !matches!(state.main_content.last(), Some(Value::String(value)) if value == "\n")
    {
        state.main_content.push(Value::String("\n".to_string()));
    }

    let mut inner_container = state.main_content;
    if !top_level_has_weave_points {
        inner_container.push(Value::String("end".to_string()));
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
    gather_index: usize,
    last_top_level_kind: Option<ObjectKind>,
}

fn export_object_into(
    object: &ObjectRef,
    state: &mut ExportState,
    tokens: &mut Vec<Value>,
    allow_named_flow: bool,
    inherited_gather_target: Option<String>,
) -> Result<(), CompilerError> {
    let (kind, content) = {
        let borrowed = object.borrow();
        (borrowed.kind().clone(), borrowed.content().to_vec())
    };

    match kind {
        ObjectKind::ContentList { .. } => {
            export_children(
                content,
                state,
                tokens,
                allow_named_flow,
                inherited_gather_target,
            )?;
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
                    let container = export_named_flow_container(
                        &content,
                        state,
                        inherited_gather_target.clone(),
                    )?;
                    state.named_containers.insert(name.clone(), container);
                } else {
                    export_children(content, state, tokens, false, inherited_gather_target)?;
                }
            } else {
                export_children(content, state, tokens, false, inherited_gather_target)?;
            }
        }
        ObjectKind::Choice {
            once_only,
            is_invisible_default,
            has_condition,
            has_start_content,
            has_choice_only_content,
            has_inline_inner_content,
            ..
        } => {
            let choice_named_content = export_choice_container(
                object,
                state,
                once_only,
                is_invisible_default,
                has_condition,
                has_start_content,
                has_choice_only_content,
                has_inline_inner_content,
                tokens.len(),
                None,
                inherited_gather_target.clone(),
                tokens,
            )?;
            tokens.push(Value::Object(
                choice_named_content.into_iter().collect::<Map<_, _>>(),
            ));
        }
        ObjectKind::Sequence { sequence_type } => {
            let sequence_container = export_sequence_container(&content, sequence_type, state)?;
            tokens.push(sequence_container);
        }
        ObjectKind::Gather { identifier, .. } => {
            let generated_index = state.gather_index;
            state.gather_index += 1;
            let gather_name = identifier
                .map(|identifier| identifier.name)
                .unwrap_or_else(|| format!("g-{}", generated_index));

            let container = export_gather_container(&content, state, inherited_gather_target)?;
            state.named_containers.insert(gather_name, container);
        }
        ObjectKind::Conditional
        | ObjectKind::ConditionalSingleBranch { .. }
        | ObjectKind::ConstantDeclaration { .. }
        | ObjectKind::ExternalDeclaration { .. }
        | ObjectKind::ListDefinition { .. }
        | ObjectKind::ListElementDefinition { .. }
        | ObjectKind::Generic => {
            return Err(CompilerError::Unsupported(
                "runtime export currently supports plain text stories and list definitions only",
            ));
        }
        ObjectKind::Expression { .. } => {
            tokens.extend(export_expression_tokens(object, state)?);
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
    inherited_gather_target: Option<String>,
) -> Result<Value, CompilerError> {
    let mut tokens = Vec::new();
    export_children(
        flow_content.to_vec(),
        state,
        &mut tokens,
        false,
        inherited_gather_target,
    )?;

    let should_wrap = flow_content.iter().any(|object| {
        matches!(
            object.borrow().kind(),
            ObjectKind::Choice { .. }
                | ObjectKind::Gather { .. }
                | ObjectKind::Sequence { .. }
                | ObjectKind::Conditional
                | ObjectKind::ConditionalSingleBranch { .. }
        )
    });

    if should_wrap {
        Ok(Value::Array(vec![Value::Array(tokens), Value::Null]))
    } else {
        tokens.push(Value::Null);
        Ok(Value::Array(tokens))
    }
}

fn export_gather_container(
    gather_content: &[ObjectRef],
    state: &mut ExportState,
    inherited_gather_target: Option<String>,
) -> Result<Value, CompilerError> {
    let mut tokens = Vec::new();
    export_children(
        gather_content.to_vec(),
        state,
        &mut tokens,
        false,
        inherited_gather_target,
    )?;

    let had_terminal_end = matches!(tokens.last(), Some(Value::String(value)) if value == "end");
    if had_terminal_end {
        tokens.pop();
    }

    if !matches!(tokens.last(), Some(Value::String(value)) if value == "\n") {
        tokens.push(Value::String("\n".to_string()));
    }
    tokens.push(Value::String("end".to_string()));
    tokens.push(json!([
        Value::String("done".to_string()),
        json!({"#n": format!("g-{}", state.gather_index)})
    ]));
    tokens.push(Value::Null);
    Ok(Value::Array(tokens))
}

fn export_sequence_container(
    sequence_content: &[ObjectRef],
    sequence_type: SequenceType,
    state: &mut ExportState,
) -> Result<Value, CompilerError> {
    let is_once = sequence_type.is_once();
    let is_cycle = sequence_type.is_cycle();
    let is_shuffle = sequence_type.is_shuffle();
    let is_stopping = sequence_type.is_stopping();
    let sequence_branch_count = if is_once {
        sequence_content.len() + 1
    } else {
        sequence_content.len()
    };

    let mut outer_tokens = vec![json!("ev"), json!("visit")];

    if is_stopping || is_once {
        outer_tokens.push(json!(sequence_branch_count - 1));
        outer_tokens.push(json!("MIN"));
    } else if is_cycle {
        outer_tokens.push(json!(sequence_content.len()));
        outer_tokens.push(json!("%"));
    }

    if is_shuffle {
        if is_stopping || is_once {
            outer_tokens.push(json!("du"));
            let last_idx = if is_stopping {
                sequence_content.len().saturating_sub(1)
            } else {
                sequence_content.len()
            };
            outer_tokens.push(json!(last_idx));
            outer_tokens.push(json!("=="));

            let post_shuffle_no_op_index = outer_tokens.len() + 3;
            outer_tokens.push(json!({
                "->": format!(".^.{}", post_shuffle_no_op_index),
                "c": true,
            }));
        }

        let element_count_to_shuffle = if is_stopping {
            sequence_content.len().saturating_sub(1)
        } else {
            sequence_content.len()
        };
        outer_tokens.push(json!(element_count_to_shuffle));
        outer_tokens.push(json!("seq"));

        if is_stopping || is_once {
            outer_tokens.push(json!("nop"));
        }
    }

    outer_tokens.push(json!("/ev"));
    outer_tokens.push(json!("ev"));

    let post_sequence_no_op_index =
        outer_tokens.len() + (sequence_branch_count * 6).saturating_sub(1);
    let mut branch_named_content = BTreeMap::new();

    for branch_index in 0..sequence_branch_count {
        if branch_index > 0 {
            outer_tokens.push(json!("ev"));
        }

        outer_tokens.push(json!("du"));
        outer_tokens.push(json!(branch_index));
        outer_tokens.push(json!("=="));
        outer_tokens.push(json!("/ev"));
        outer_tokens.push(json!({
            "->": format!(".^.s{branch_index}"),
            "c": true,
        }));

        let mut branch_tokens = vec![json!("pop")];
        if let Some(branch_content) = sequence_content.get(branch_index) {
            branch_tokens.extend(export_content_list_tokens(branch_content, state, None)?);
        }
        branch_tokens.push(json!({
            "->": format!(".^.^.{post_sequence_no_op_index}"),
        }));
        branch_tokens.push(Value::Null);
        branch_named_content.insert(format!("s{branch_index}"), Value::Array(branch_tokens));
    }

    outer_tokens.push(json!("nop"));
    branch_named_content.insert("#f".to_string(), json!(5));
    outer_tokens.push(Value::Object(
        branch_named_content.into_iter().collect::<Map<_, _>>(),
    ));

    Ok(Value::Array(outer_tokens))
}

fn export_choice_container(
    object: &ObjectRef,
    state: &mut ExportState,
    once_only: bool,
    is_invisible_default: bool,
    has_condition: bool,
    has_start_content: bool,
    has_choice_only_content: bool,
    has_inline_inner_content: bool,
    outer_index: usize,
    branch_gather_target: Option<String>,
    inherited_gather_target: Option<String>,
    tokens: &mut Vec<Value>,
) -> Result<BTreeMap<String, Value>, CompilerError> {
    let choice_index = state.choice_index;
    state.choice_index += 1;

    let (start_content, choice_only_content, inner_content) = {
        let mut children = object.borrow().content().to_vec();
        if has_condition {
            children.pop();
        }
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
    let condition = if has_condition {
        object.borrow().content().last().cloned()
    } else {
        None
    };
    let needs_eval = has_start_content || has_choice_only_content || has_condition;

    if needs_eval {
        outer_tokens.push(json!("ev"));
    }

    if let Some(_start_content) = start_content.as_ref() {
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
        outer_tokens.push(json!("str"));
        outer_tokens.extend(export_content_list_tokens(
            choice_only_content,
            state,
            branch_gather_target
                .clone()
                .or(inherited_gather_target.clone()),
        )?);
        outer_tokens.push(json!("/str"));
    }

    if let Some(condition) = condition.as_ref() {
        outer_tokens.extend(export_expression_tokens(condition, state)?);
    }

    if needs_eval {
        outer_tokens.push(json!("/ev"));
    }

    let flags = {
        let mut flags = 0;
        if has_condition {
            flags |= 1;
        }
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
    let choice_path = if has_start_content {
        format!("0.c-{choice_index}")
    } else {
        format!(".^.c-{choice_index}")
    };
    outer_tokens.push(json!({"*": choice_path, "flg": flags}));

    if let Some(start_content) = start_content {
        let mut s_tokens = Vec::new();
        s_tokens.extend(export_content_list_tokens(
            &start_content,
            state,
            branch_gather_target
                .clone()
                .or(inherited_gather_target.clone()),
        )?);
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
        export_content_list_tokens(
            &inner_content,
            state,
            branch_gather_target
                .clone()
                .or(inherited_gather_target.clone()),
        )?
    } else {
        Vec::new()
    };

    if has_start_content
        && !has_inline_inner_content
        && !matches!(body_tokens.first(), Some(Value::String(value)) if value == "\n")
    {
        body_tokens.insert(0, Value::String("\n".to_string()));
    }

    inner_tokens.extend(body_tokens);

    if has_start_content {
        if branch_gather_target.is_some()
            && !matches!(
                inner_tokens.last(),
                Some(Value::String(value)) if value == "\n"
            )
        {
            inner_tokens.push(Value::String("\n".to_string()));
        }
        if branch_gather_target.is_none()
            && !matches!(
                inner_tokens.last(),
                Some(Value::String(value)) if value == "done" || value == "end"
            )
        {
            inner_tokens.push(Value::String("end".to_string()));
        }

        let branch_target = branch_gather_target
            .clone()
            .unwrap_or_else(|| format!("0.g-{choice_index}"));
        inner_tokens.push(json!({"->": branch_target}));
        inner_tokens.push(json!({"#f":5}));

        choice_named_content.insert(format!("c-{choice_index}"), Value::Array(inner_tokens));
        if branch_gather_target.is_none() {
            choice_named_content.insert(
                format!("g-{choice_index}"),
                Value::Array(vec![Value::String("done".to_string()), Value::Null]),
            );
        }
    } else {
        if let Some(branch_gather_target) = branch_gather_target {
            inner_tokens.push(json!({"->": branch_gather_target}));
        }
        if !matches!(
            inner_tokens.last(),
            Some(Value::String(value)) if value == "\n"
        ) {
            inner_tokens.push(Value::String("\n".to_string()));
        }
        inner_tokens.push(json!({"#f":5}));
        choice_named_content.insert(format!("c-{choice_index}"), Value::Array(inner_tokens));
    }
    if has_start_content {
        tokens.push(Value::Array(outer_tokens));
    } else {
        tokens.extend(outer_tokens);
    }
    Ok(choice_named_content)
}

fn export_content_list_tokens(
    content_list: &ObjectRef,
    state: &mut ExportState,
    inherited_gather_target: Option<String>,
) -> Result<Vec<Value>, CompilerError> {
    let children = {
        let borrowed = content_list.borrow();
        borrowed.content().to_vec()
    };

    let mut tokens = Vec::new();
    export_children(children, state, &mut tokens, false, inherited_gather_target)?;
    normalize_text_token_sequences(&mut tokens);

    Ok(tokens)
}

fn normalize_text_token_sequences(tokens: &mut Vec<Value>) {
    let mut normalized = Vec::with_capacity(tokens.len());
    let mut index = 0usize;

    while index < tokens.len() {
        let current = tokens.get(index).cloned();
        let next = tokens.get(index + 1).cloned();
        let next_next = tokens.get(index + 2).cloned();

        if let (
            Some(Value::String(current_text)),
            Some(Value::String(next_text)),
            Some(Value::String(next_next_text)),
        ) = (current.as_ref(), next.as_ref(), next_next.as_ref())
        {
            let current_body = current_text.strip_prefix('^').unwrap_or(current_text);
            let next_body = next_next_text.strip_prefix('^').unwrap_or(next_next_text);
            if current_body.chars().all(|c| c == ' ' || c == '\t')
                && next_text == "\n"
                && !next_body.is_empty()
            {
                normalized.push(Value::String(format!("^{}{}", current_body, next_body)));
                normalized.push(Value::String("\n".to_string()));
                index += 3;
                continue;
            }
        }

        if let Some(value) = current {
            normalized.push(value);
        }
        index += 1;
    }

    *tokens = normalized;
}

fn export_children(
    children: Vec<ObjectRef>,
    state: &mut ExportState,
    tokens: &mut Vec<Value>,
    allow_named_flow: bool,
    inherited_gather_target: Option<String>,
) -> Result<(), CompilerError> {
    let mut pending_choice_named_contents: Vec<BTreeMap<String, Value>> = Vec::new();

    for (index, child) in children.iter().enumerate() {
        let child_kind = child.borrow().kind().clone();
        match child_kind {
            ObjectKind::Choice {
                once_only,
                is_invisible_default,
                has_condition,
                has_start_content,
                has_choice_only_content,
                has_inline_inner_content,
                ..
            } => {
                let choice_named_content = export_choice_container(
                    &child,
                    state,
                    once_only,
                    is_invisible_default,
                    has_condition,
                    has_start_content,
                    has_choice_only_content,
                    has_inline_inner_content,
                    tokens.len(),
                    gather_target_for_choice(
                        &children,
                        index,
                        state,
                        inherited_gather_target.clone(),
                    ),
                    inherited_gather_target.clone(),
                    tokens,
                )?;
                pending_choice_named_contents.push(choice_named_content);
            }
            ObjectKind::Gather { .. } if !pending_choice_named_contents.is_empty() => {
                let gather_named_content =
                    export_gather_named_content(child, state, inherited_gather_target.clone())?;
                pending_choice_named_contents.push(gather_named_content);
            }
            _ => {
                if !pending_choice_named_contents.is_empty() {
                    flush_pending_choice_named_contents(&mut pending_choice_named_contents, tokens);
                }
                export_object_into(
                    child,
                    state,
                    tokens,
                    allow_named_flow,
                    inherited_gather_target.clone(),
                )?;
            }
        }
    }

    if !pending_choice_named_contents.is_empty() {
        flush_pending_choice_named_contents(&mut pending_choice_named_contents, tokens);
    }

    Ok(())
}

fn gather_target_for_choice(
    children: &[ObjectRef],
    current_index: usize,
    state: &ExportState,
    inherited_gather_target: Option<String>,
) -> Option<String> {
    for child in children.iter().skip(current_index + 1) {
        match child.borrow().kind() {
            ObjectKind::Choice { .. } => continue,
            ObjectKind::Gather { .. } => return Some(format!("0.g-{}", state.gather_index)),
            _ => return None,
        }
    }

    inherited_gather_target
}

fn export_gather_named_content(
    object: &ObjectRef,
    state: &mut ExportState,
    _inherited_gather_target: Option<String>,
) -> Result<BTreeMap<String, Value>, CompilerError> {
    let (identifier, content) = {
        let borrowed = object.borrow();
        let identifier = match borrowed.kind() {
            ObjectKind::Gather { identifier, .. } => identifier.clone(),
            _ => None,
        };
        (identifier, borrowed.content().to_vec())
    };

    let generated_index = state.gather_index;
    state.gather_index += 1;
    let gather_name = identifier
        .map(|identifier| identifier.name)
        .unwrap_or_else(|| format!("g-{}", generated_index));

    let container = export_gather_container(&content, state, _inherited_gather_target)?;
    Ok(vec![(gather_name, container)]
        .into_iter()
        .collect::<BTreeMap<_, _>>())
}

fn flush_pending_choice_named_contents(
    pending_choice_named_contents: &mut Vec<BTreeMap<String, Value>>,
    tokens: &mut Vec<Value>,
) {
    if pending_choice_named_contents.is_empty() {
        return;
    }

    let mut merged = BTreeMap::new();
    for entry in pending_choice_named_contents.drain(..) {
        merged.extend(entry);
    }

    tokens.push(Value::Object(merged.into_iter().collect::<Map<_, _>>()));
}

fn export_expression_tokens(
    expression: &ObjectRef,
    state: &mut ExportState,
) -> Result<Vec<Value>, CompilerError> {
    let (kind, content) = {
        let borrowed = expression.borrow();
        (borrowed.kind().clone(), borrowed.content().to_vec())
    };

    let mut tokens = Vec::new();
    match kind {
        ObjectKind::Expression {
            kind: ExpressionKind::Number(value),
        } => match value {
            crate::parsed::NumberValue::Int(value) => tokens.push(json!(value)),
            crate::parsed::NumberValue::Float(value) => tokens.push(json!(value)),
            crate::parsed::NumberValue::Bool(value) => tokens.push(json!(value)),
        },
        ObjectKind::Expression {
            kind: ExpressionKind::StringExpression,
        } => {
            tokens.push(json!("str"));
            for child in content {
                tokens.extend(export_object_tokens(&child, state)?);
            }
            tokens.push(json!("/str"));
        }
        ObjectKind::Expression {
            kind: ExpressionKind::VariableReference { path, .. },
        } => {
            let path = path
                .into_iter()
                .map(|identifier| identifier.name)
                .collect::<Vec<_>>()
                .join(".");
            tokens.push(json!({"VAR?": path}));
        }
        ObjectKind::Expression {
            kind: ExpressionKind::FunctionCall { function_name, .. },
        } => {
            for argument in content {
                tokens.extend(export_expression_tokens(&argument, state)?);
            }
            tokens.push(json!({"f()": function_name.name}));
        }
        ObjectKind::Expression {
            kind: ExpressionKind::DivertTarget,
        } => {
            let target = content
                .first()
                .and_then(|child| match child.borrow().kind() {
                    ObjectKind::Divert { target, .. } => target.clone(),
                    _ => None,
                })
                .and_then(|path| path.dot_separated_components())
                .unwrap_or_default();
            tokens.push(json!({"^->": target}));
        }
        ObjectKind::Expression {
            kind: ExpressionKind::Binary { op_name },
        } => {
            if let Some(left) = content.first().cloned() {
                tokens.extend(export_expression_tokens(&left, state)?);
            }
            if let Some(right) = content.get(1).cloned() {
                tokens.extend(export_expression_tokens(&right, state)?);
            }
            tokens.push(json!(runtime_operator_name(&op_name)));
        }
        ObjectKind::Expression {
            kind: ExpressionKind::Unary { op },
        } => {
            if let Some(inner) = content.first().cloned() {
                tokens.extend(export_expression_tokens(&inner, state)?);
            }
            tokens.push(json!(runtime_unary_operator_name(&op)));
        }
        ObjectKind::Expression {
            kind: ExpressionKind::IncDec { identifier, is_inc },
        } => {
            tokens.push(json!({"VAR?": identifier.name}));
            tokens.push(json!(if is_inc { "++" } else { "--" }));
        }
        ObjectKind::Expression {
            kind: ExpressionKind::MultipleCondition,
        } => {
            let mut iter = content.into_iter();
            if let Some(first) = iter.next() {
                tokens.extend(export_expression_tokens(&first, state)?);
            }
            for child in iter {
                tokens.extend(export_expression_tokens(&child, state)?);
                tokens.push(json!("&&"));
            }
        }
        ObjectKind::Expression {
            kind: ExpressionKind::List { .. },
        } => {
            return Err(CompilerError::Unsupported(
                "runtime export currently supports plain text stories and basic expressions only",
            ));
        }
        _ => {
            return Err(CompilerError::Unsupported(
                "runtime export currently supports plain text stories and basic expressions only",
            ));
        }
    }

    Ok(tokens)
}

fn export_object_tokens(
    object: &ObjectRef,
    state: &mut ExportState,
) -> Result<Vec<Value>, CompilerError> {
    let mut tokens = Vec::new();
    export_object_into(object, state, &mut tokens, false, None)?;
    Ok(tokens)
}

fn runtime_operator_name(op_name: &str) -> &str {
    match op_name {
        "and" => "&&",
        "or" => "||",
        "mod" => "%",
        "has" => "?",
        "hasnt" => "!?",
        _ => op_name,
    }
}

fn runtime_unary_operator_name(op: &str) -> &str {
    match op {
        "-" => "_",
        "not" => "!",
        _ => op,
    }
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
    use ink_runtime::story::Story as RuntimeStory;

    use crate::parsed::{
        ContentList, Divert, Identifier, Knot, ListDefinition, ListElementDefinition, Path,
        Sequence, SequenceType, Story, Text, VariableAssignment,
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
        let line = ContentList::new();
        line.add_content(Text::new("Hello ").object());

        let first = ContentList::new();
        first.add_content(Text::new("One").object());

        let second = ContentList::new();
        second.add_content(Text::new("Two").object());

        line.add_content(Sequence::new(vec![first, second], SequenceType::Stopping).object());
        line.add_content(Text::new("\n").object());

        let story = Story::new(vec![line.object()], false);

        let json = export_story_json(&story).expect("expected sequence story export");
        let _runtime_story = RuntimeStory::new(&json).expect("expected runtime story to load");
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
                Divert::new(Some(Path::new(vec![Identifier::new("END")]))).object(),
            ],
            Vec::new(),
            false,
        );
        let story = Story::new(vec![line.object(), knot.object()], false);

        let json = export_story_json(&story).expect("expected divert story export");
        assert!(json.contains("\"->\":\"hurry_home\""));
        assert!(json.contains("\"hurry_home\""));

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
