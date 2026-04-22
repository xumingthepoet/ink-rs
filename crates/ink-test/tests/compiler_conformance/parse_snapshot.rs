use ink_compiler::parsed::{ExpressionKind, ObjectKind, ObjectRef, Story as ParsedStory};

pub fn render_story(story: &ParsedStory) -> String {
    let mut lines = vec!["Story".to_string()];

    for child in story.content() {
        render_object(&child, 1, &mut lines);
    }

    lines.join("\n")
}

fn render_object(object: &ObjectRef, indent: usize, lines: &mut Vec<String>) {
    let padding = "  ".repeat(indent);
    let borrowed = object.borrow();

    match borrowed.kind() {
        ObjectKind::ContentList { .. } => {
            lines.push(format!("{padding}ContentList"));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::Text { text } => {
            lines.push(format!(
                "{padding}Text({})",
                serde_json::to_string(text).unwrap()
            ));
        }
        ObjectKind::Divert {
            target,
            is_empty,
            is_tunnel,
            is_thread,
        } => {
            let target = target
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "->".to_string());
            lines.push(format!(
                "{padding}Divert(target={}, empty={}, tunnel={}, thread={})",
                serde_json::to_string(&target).unwrap(),
                bool_str(*is_empty),
                bool_str(*is_tunnel),
                bool_str(*is_thread)
            ));
        }
        ObjectKind::Flow {
            flow_level,
            name,
            is_function,
        } => {
            lines.push(format!(
                "{padding}Flow(level={flow_level:?}, name={}, function={})",
                serde_json::to_string(name).unwrap(),
                bool_str(*is_function)
            ));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::AuthorWarning { warning_message } => {
            lines.push(format!(
                "{padding}AuthorWarning({})",
                serde_json::to_string(warning_message).unwrap()
            ));
        }
        ObjectKind::Tag {
            is_start,
            in_choice,
        } => {
            lines.push(format!(
                "{padding}Tag(start={}, inChoice={})",
                bool_str(*is_start),
                bool_str(*in_choice)
            ));
        }
        ObjectKind::VariableAssignment {
            identifier,
            is_global_declaration,
            is_new_temporary_declaration,
        } => {
            lines.push(format!(
                "{padding}VariableAssignment(name={}, global={}, temp={})",
                serde_json::to_string(&identifier.name).unwrap(),
                bool_str(*is_global_declaration),
                bool_str(*is_new_temporary_declaration)
            ));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::ConstantDeclaration { identifier } => {
            lines.push(format!(
                "{padding}ConstantDeclaration(name={})",
                serde_json::to_string(&identifier.name).unwrap()
            ));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::ExternalDeclaration {
            identifier,
            argument_names,
        } => {
            lines.push(format!(
                "{padding}ExternalDeclaration(name={}, args={})",
                serde_json::to_string(&identifier.name).unwrap(),
                serde_json::to_string(argument_names).unwrap()
            ));
        }
        ObjectKind::Return => {
            lines.push(format!("{padding}Return"));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::ListDefinition { identifier } => {
            lines.push(format!(
                "{padding}ListDefinition(name={})",
                serde_json::to_string(&identifier.name).unwrap()
            ));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::ListElementDefinition {
            identifier,
            explicit_value,
            series_value,
            in_initial_list,
        } => {
            lines.push(format!(
                "{padding}ListElementDefinition(name={}, explicit={}, series={}, initial={})",
                serde_json::to_string(&identifier.name).unwrap(),
                format_option_int(*explicit_value),
                series_value,
                bool_str(*in_initial_list)
            ));
        }
        ObjectKind::Sequence { sequence_type } => {
            lines.push(format!("{padding}Sequence(type={sequence_type})"));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::Conditional => {
            lines.push(format!("{padding}Conditional"));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::ConditionalSingleBranch {
            is_true_branch,
            is_else,
            is_inline,
        } => {
            lines.push(format!(
                "{padding}ConditionalBranch(true={}, else={}, inline={})",
                bool_str(*is_true_branch),
                bool_str(*is_else),
                bool_str(*is_inline)
            ));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::Choice {
            identifier,
            indentation_depth,
            once_only,
            is_invisible_default,
            has_weave_style_inline_brackets,
            ..
        } => {
            lines.push(format!(
                "{padding}Choice(name={}, once={}, invisible={}, depth={}, inline={})",
                serde_json::to_string(
                    &identifier
                        .as_ref()
                        .map(|identifier| identifier.name.as_str())
                )
                .unwrap(),
                bool_str(*once_only),
                bool_str(*is_invisible_default),
                indentation_depth,
                bool_str(*has_weave_style_inline_brackets)
            ));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::Gather {
            identifier,
            indentation_depth,
        } => {
            lines.push(format!(
                "{padding}Gather(name={}, depth={})",
                serde_json::to_string(
                    &identifier
                        .as_ref()
                        .map(|identifier| identifier.name.as_str())
                )
                .unwrap(),
                indentation_depth
            ));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::Weave { base_indent_index } => {
            lines.push(format!("{padding}Weave(baseIndent={base_indent_index})"));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::Expression { kind } => {
            lines.push(format!("{padding}{}", render_expression(object, kind)));
        }
        ObjectKind::Generic => {
            lines.push(format!("{padding}Generic"));
        }
    }
}

fn render_expression(object: &ObjectRef, kind: &ExpressionKind) -> String {
    let borrowed = object.borrow();

    match kind {
        ExpressionKind::Number(value) => format!("Number({value})"),
        ExpressionKind::StringExpression => {
            let text = borrowed
                .content()
                .iter()
                .filter_map(|child| match child.borrow().kind() {
                    ObjectKind::Text { text } => Some(text.clone()),
                    _ => None,
                })
                .collect::<String>();
            format!("String({})", serde_json::to_string(&text).unwrap())
        }
        ExpressionKind::VariableReference { path, .. } => format!(
            "VariableReference({})",
            path.iter()
                .map(|identifier| identifier.name.clone())
                .collect::<Vec<_>>()
                .join(".")
        ),
        ExpressionKind::FunctionCall { function_name, .. } => format!(
            "FunctionCall({}, args={})",
            function_name.name,
            borrowed.content().len()
        ),
        ExpressionKind::DivertTarget => {
            let target = borrowed
                .content()
                .first()
                .and_then(|child| match child.borrow().kind() {
                    ObjectKind::Divert { target, .. } => Some(
                        target
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "->".to_string()),
                    ),
                    _ => None,
                })
                .unwrap_or_else(|| "->".to_string());
            format!("DivertTarget({target})")
        }
        ExpressionKind::List { item_identifiers } => format!(
            "List({})",
            item_identifiers
                .iter()
                .map(|identifier| identifier.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ExpressionKind::Binary { op_name } => format!(
            "Binary({op_name}, {}, {})",
            borrowed
                .content()
                .first()
                .map(render_expression_ref)
                .unwrap_or_else(|| "<missing>".to_string()),
            borrowed
                .content()
                .get(1)
                .map(render_expression_ref)
                .unwrap_or_else(|| "<missing>".to_string())
        ),
        ExpressionKind::Unary { op } => format!(
            "Unary({op}, {})",
            borrowed
                .content()
                .first()
                .map(render_expression_ref)
                .unwrap_or_else(|| "<missing>".to_string())
        ),
        ExpressionKind::IncDec { identifier, is_inc } => {
            format!("IncDec({}, inc={})", identifier.name, bool_str(*is_inc))
        }
        ExpressionKind::MultipleCondition => format!(
            "MultipleCondition({})",
            borrowed
                .content()
                .iter()
                .map(render_expression_ref)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn render_expression_ref(object: &ObjectRef) -> String {
    let borrowed = object.borrow();
    match borrowed.kind() {
        ObjectKind::Expression { kind } => render_expression(object, kind),
        ObjectKind::Text { text } => format!("Text({})", serde_json::to_string(text).unwrap()),
        other => format!("{other:?}"),
    }
}

fn bool_str(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn format_option_int(value: Option<i64>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".to_string(),
    }
}
