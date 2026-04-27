use ink_story_json_format::{ControlCommand, Object as RuntimeObject};

use crate::parsed::{AssignmentTarget, Expression, IncDec, VariableAssignment};

use super::context::{ChoicePathMode, LoweringContext};
use super::expression::lower_expression_into;
use super::value::{lower_value_literal, runtime_default_for_type};

pub(super) enum AssignmentPathComponent<'a> {
    Field(&'a str),
    Index(&'a Expression),
    CachedIndex(String),
}

#[derive(Clone, Copy)]
pub(super) enum AssignmentUpdateValue<'a> {
    Expression(&'a Expression),
    Compound {
        expression: &'a Expression,
        operator: &'static str,
    },
    ArrayRemove {
        index: &'a Expression,
    },
}

pub(super) fn lower_assignment_initializer_with_context(
    content: &mut Vec<RuntimeObject>,
    assignment: &VariableAssignment,
    context: &LoweringContext<'_>,
) -> bool {
    lower_assignment_initializer_into(content, assignment, context)
}

fn lower_assignment_initializer_into(
    content: &mut Vec<RuntimeObject>,
    assignment: &VariableAssignment,
    context: &LoweringContext<'_>,
) -> bool {
    if let Some(expression) = assignment.expression() {
        if let Expression::ArrayLiteral(_) | Expression::StructLiteral(_) = expression {
            if let Some(value) = lower_value_literal(
                expression,
                assignment.declared_type(),
                context.struct_definitions(),
                context.choice_labels(),
                context.global_labels(),
                context.path_mode(),
            ) {
                content.push(value);
                return true;
            }
        }

        lower_expression_into(content, expression, context, false);
        return true;
    }

    assignment
        .declared_type()
        .and_then(|declared_type| {
            runtime_default_for_type(
                declared_type,
                context.struct_definitions(),
                context.path_mode().current_module_name(),
            )
        })
        .map(|default_value| content.push(default_value))
        .is_some()
}

pub(super) fn collect_assignment_path<'a>(
    target: &'a AssignmentTarget,
    components: &mut Vec<AssignmentPathComponent<'a>>,
) -> Option<&'a str> {
    match target {
        AssignmentTarget::Variable(name) => Some(name),
        AssignmentTarget::QualifiedVariable(name) => Some(name.as_str()),
        AssignmentTarget::FieldAccess { base, field } => {
            let root = collect_assignment_path(base, components)?;
            components.push(AssignmentPathComponent::Field(field));
            Some(root)
        }
        AssignmentTarget::IndexAccess { base, index } => {
            let root = collect_assignment_path(base, components)?;
            components.push(AssignmentPathComponent::Index(index));
            Some(root)
        }
    }
}

fn lower_assignment_path_component_key_into(
    content: &mut Vec<RuntimeObject>,
    component: &AssignmentPathComponent<'_>,
    context: &LoweringContext<'_>,
) {
    match component {
        AssignmentPathComponent::Field(field) => {
            content.push(RuntimeObject::String((*field).to_string()));
        }
        AssignmentPathComponent::CachedIndex(name) => {
            content.push(RuntimeObject::VariableReference(name.clone()));
        }
        AssignmentPathComponent::Index(index) => {
            lower_expression_into(content, index, context, false)
        }
    }
}

fn lower_assignment_path_read_into(
    content: &mut Vec<RuntimeObject>,
    root_name: &str,
    components: &[AssignmentPathComponent<'_>],
    context: &LoweringContext<'_>,
) {
    content.push(RuntimeObject::VariableReference(
        resolve_runtime_variable_name(root_name, context.path_mode(), context.global_variables()),
    ));
    for component in components {
        lower_assignment_path_component_key_into(content, component, context);
        let read_operation = match component {
            AssignmentPathComponent::Field(_) => "FIELD",
            AssignmentPathComponent::Index(_) | AssignmentPathComponent::CachedIndex(_) => "INDEX",
        };
        content.push(RuntimeObject::NativeFunction(read_operation.to_string()));
    }
}

fn lower_assignment_update_value_into(
    content: &mut Vec<RuntimeObject>,
    root_name: &str,
    components: &[AssignmentPathComponent<'_>],
    value: AssignmentUpdateValue<'_>,
    context: &LoweringContext<'_>,
) {
    match value {
        AssignmentUpdateValue::Expression(expression) => {
            lower_expression_into(content, expression, context, false)
        }
        AssignmentUpdateValue::Compound {
            expression,
            operator,
        } => {
            lower_assignment_path_read_into(content, root_name, components, context);
            lower_expression_into(content, expression, context, false);
            content.push(RuntimeObject::NativeFunction(operator.to_string()));
        }
        AssignmentUpdateValue::ArrayRemove { index } => {
            lower_assignment_path_read_into(content, root_name, components, context);
            lower_expression_into(content, index, context, false);
            content.push(RuntimeObject::NativeFunction("ARRAY_REMOVE".to_string()));
        }
    }
}

pub(super) fn lower_assignment_path_update_value_into(
    content: &mut Vec<RuntimeObject>,
    root_name: &str,
    components: &[AssignmentPathComponent<'_>],
    component_index: usize,
    value: AssignmentUpdateValue<'_>,
    context: &LoweringContext<'_>,
) {
    lower_assignment_path_read_into(content, root_name, &components[..component_index], context);
    lower_assignment_path_component_key_into(content, &components[component_index], context);

    if component_index + 1 == components.len() {
        lower_assignment_update_value_into(content, root_name, components, value, context);
    } else {
        lower_assignment_path_update_value_into(
            content,
            root_name,
            components,
            component_index + 1,
            value,
            context,
        );
    }

    let write_operation = match &components[component_index] {
        AssignmentPathComponent::Field(_) => "SET_FIELD",
        AssignmentPathComponent::Index(_) | AssignmentPathComponent::CachedIndex(_) => "SET_INDEX",
    };
    content.push(RuntimeObject::NativeFunction(write_operation.to_string()));
}

pub(super) fn lower_cached_assignment_indexes_into<'a>(
    content: &mut Vec<RuntimeObject>,
    components: &[AssignmentPathComponent<'a>],
    context: &LoweringContext<'_>,
) -> Vec<AssignmentPathComponent<'a>> {
    let mut cached_components = Vec::with_capacity(components.len());
    let mut next_index = 0;
    for component in components {
        match component {
            AssignmentPathComponent::Field(field) => {
                cached_components.push(AssignmentPathComponent::Field(field));
            }
            AssignmentPathComponent::CachedIndex(name) => {
                cached_components.push(AssignmentPathComponent::CachedIndex(name.clone()));
            }
            AssignmentPathComponent::Index(index) => {
                let temp_name = format!("$lvalue{next_index}");
                next_index += 1;
                lower_expression_into(content, index, context, false);
                content.push(RuntimeObject::VariableAssignment(temp_name.clone()));
                cached_components.push(AssignmentPathComponent::CachedIndex(temp_name));
            }
        }
    }
    cached_components
}

pub(super) fn push_reassignment_for_name(
    content: &mut Vec<RuntimeObject>,
    name: &str,
    path_mode: &ChoicePathMode,
) {
    if path_mode.is_local_variable(name) {
        content.push(RuntimeObject::TempVariableReassignment(name.to_string()));
    } else {
        content.push(RuntimeObject::VariableReassignment(name.to_string()));
    }
}

fn resolve_runtime_variable_name(
    name: &str,
    path_mode: &ChoicePathMode,
    global_variables: &std::collections::HashSet<String>,
) -> String {
    if name.contains("::") || path_mode.is_local_variable(name) {
        return name.to_string();
    }

    path_mode
        .current_module_name()
        .map(|module_name| format!("{module_name}::{name}"))
        .filter(|qualified_name| global_variables.contains(qualified_name))
        .unwrap_or_else(|| name.to_string())
}

pub(super) fn lower_variable_assignment_into(
    content: &mut Vec<RuntimeObject>,
    assignment: &VariableAssignment,
    context: &LoweringContext<'_>,
) {
    if assignment.is_global() {
        return;
    }

    let Some(name) = assignment.target().variable_name() else {
        let Some(expression) = assignment.expression() else {
            return;
        };
        let mut components = Vec::new();
        let Some(root_name) = collect_assignment_path(assignment.target(), &mut components) else {
            return;
        };
        if components.is_empty() {
            return;
        }

        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
        let resolved_root_name = resolve_runtime_variable_name(
            root_name,
            context.path_mode(),
            context.global_variables(),
        );
        lower_assignment_path_update_value_into(
            content,
            resolved_root_name.as_str(),
            &components,
            0,
            AssignmentUpdateValue::Expression(expression),
            context,
        );
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
        push_reassignment_for_name(content, resolved_root_name.as_str(), context.path_mode());
        return;
    };

    let mut initializer = Vec::new();
    if !lower_assignment_initializer_into(&mut initializer, assignment, context) {
        return;
    }

    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    content.extend(initializer);
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));

    if assignment.is_temporary() {
        content.push(RuntimeObject::VariableAssignment(name.to_string()));
    } else {
        let resolved_name =
            resolve_runtime_variable_name(name, context.path_mode(), context.global_variables());
        push_reassignment_for_name(content, resolved_name.as_str(), context.path_mode());
    }
}

pub(super) fn lower_inc_dec_into(
    content: &mut Vec<RuntimeObject>,
    inc_dec: &IncDec,
    context: &LoweringContext<'_>,
) {
    let Some(name) = inc_dec.target().variable_name() else {
        let mut components = Vec::new();
        let Some(root_name) = collect_assignment_path(inc_dec.target(), &mut components) else {
            return;
        };
        if components.is_empty() {
            return;
        }
        let operator = if inc_dec.is_increment() { "+" } else { "-" };

        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
        let resolved_root_name = resolve_runtime_variable_name(
            root_name,
            context.path_mode(),
            context.global_variables(),
        );
        let cached_components = lower_cached_assignment_indexes_into(content, &components, context);
        lower_assignment_path_update_value_into(
            content,
            resolved_root_name.as_str(),
            &cached_components,
            0,
            AssignmentUpdateValue::Compound {
                expression: inc_dec.expression(),
                operator,
            },
            context,
        );
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
        push_reassignment_for_name(content, resolved_root_name.as_str(), context.path_mode());
        return;
    };
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    let resolved_name =
        resolve_runtime_variable_name(name, context.path_mode(), context.global_variables());
    content.push(RuntimeObject::VariableReference(resolved_name.clone()));
    lower_expression_into(content, inc_dec.expression(), context, false);
    content.push(RuntimeObject::NativeFunction(
        if inc_dec.is_increment() { "+" } else { "-" }.to_string(),
    ));
    if context.path_mode().is_local_variable(name) {
        content.push(RuntimeObject::TempVariableReassignment(resolved_name));
    } else {
        content.push(RuntimeObject::VariableReassignment(resolved_name));
    }
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
}
