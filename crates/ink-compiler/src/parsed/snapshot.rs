use super::{ExpressionKind, ObjectKind, ObjectRef, Story as ParsedStory};

#[doc(hidden)]
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
            lines.push(format!("{padding}Text({text:?})"));
        }
        ObjectKind::Divert {
            target,
            is_empty,
            is_tunnel,
            is_thread,
        } => {
            let target = target
                .as_ref()
                .map(|path| path.to_string())
                .unwrap_or_else(|| "->".to_string());
            lines.push(format!(
                "{padding}Divert(target={target:?}, empty={is_empty}, tunnel={is_tunnel}, thread={is_thread})"
            ));
        }
        ObjectKind::Flow {
            flow_level,
            name,
            is_function,
        } => {
            lines.push(format!(
                "{padding}Flow(level={flow_level:?}, name={:?}, function={is_function})",
                name.as_deref().unwrap_or("<unnamed>")
            ));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::AuthorWarning { warning_message } => {
            lines.push(format!("{padding}AuthorWarning({warning_message:?})"));
        }
        ObjectKind::Tag {
            is_start,
            in_choice,
        } => {
            lines.push(format!(
                "{padding}Tag(start={is_start}, in_choice={in_choice})"
            ));
        }
        ObjectKind::VariableAssignment {
            identifier,
            is_global_declaration,
            is_new_temporary_declaration,
        } => {
            lines.push(format!(
                "{padding}VariableAssignment(name={:?}, global={is_global_declaration}, temp={is_new_temporary_declaration})",
                identifier.name
            ));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::ConstantDeclaration { identifier } => {
            lines.push(format!(
                "{padding}ConstantDeclaration(name={:?})",
                identifier.name
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
                "{padding}ExternalDeclaration(name={:?}, args={argument_names:?})",
                identifier.name
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
                "{padding}ListDefinition(name={:?})",
                identifier.name
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
                "{padding}ListElementDefinition(name={:?}, explicit={explicit_value:?}, series={series_value}, initial={in_initial_list})",
                identifier.name
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
                "{padding}ConditionalBranch(true={is_true_branch}, else={is_else}, inline={is_inline})"
            ));
            for child in borrowed.content() {
                render_object(child, indent + 1, lines);
            }
        }
        ObjectKind::Expression { .. } => {
            lines.push(format!("{padding}{}", render_expression(object)));
        }
        other => lines.push(format!("{padding}{other:?}")),
    }
}

fn render_expression(object: &ObjectRef) -> String {
    let borrowed = object.borrow();
    match borrowed.kind() {
        ObjectKind::Expression {
            kind: ExpressionKind::Number(value),
        } => format!("Number({value})"),
        ObjectKind::Expression {
            kind: ExpressionKind::StringExpression,
        } => format!("String({:?})", borrowed.content()),
        ObjectKind::Expression {
            kind: ExpressionKind::VariableReference { path, .. },
        } => format!(
            "VariableReference({})",
            path.iter()
                .map(|identifier| identifier.name.clone())
                .collect::<Vec<_>>()
                .join(".")
        ),
        ObjectKind::Expression {
            kind: ExpressionKind::FunctionCall { function_name, .. },
        } => format!(
            "FunctionCall({}, args={})",
            function_name.name,
            borrowed.content().len()
        ),
        ObjectKind::Expression {
            kind: ExpressionKind::DivertTarget,
        } => {
            let target = borrowed
                .content()
                .first()
                .and_then(|child| match child.borrow().kind() {
                    super::ObjectKind::Divert { target, .. } => Some(
                        target
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "->".to_string()),
                    ),
                    _ => None,
                })
                .unwrap_or_else(|| "<missing>".to_string());
            format!("DivertTarget({target})")
        }
        ObjectKind::Expression {
            kind: ExpressionKind::List { item_identifiers },
        } => format!(
            "List({})",
            item_identifiers
                .iter()
                .map(|identifier| identifier.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ObjectKind::Expression {
            kind: ExpressionKind::Binary { op_name },
        } => format!(
            "Binary({op_name}, {}, {})",
            borrowed
                .content()
                .first()
                .map(render_expression)
                .unwrap_or_else(|| "<missing>".to_string()),
            borrowed
                .content()
                .get(1)
                .map(render_expression)
                .unwrap_or_else(|| "<missing>".to_string())
        ),
        ObjectKind::Expression {
            kind: ExpressionKind::Unary { op },
        } => format!(
            "Unary({op}, {})",
            borrowed
                .content()
                .first()
                .map(render_expression)
                .unwrap_or_else(|| "<missing>".to_string())
        ),
        ObjectKind::Expression {
            kind: ExpressionKind::IncDec { identifier, is_inc },
        } => format!("IncDec({}, inc={is_inc})", identifier.name),
        ObjectKind::Expression {
            kind: ExpressionKind::MultipleCondition,
        } => format!(
            "MultipleCondition({})",
            borrowed
                .content()
                .iter()
                .map(render_expression)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => "<Expression>".to_string(),
    }
}
