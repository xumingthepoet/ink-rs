use std::collections::HashSet;

use crate::parsed::{ContentList, Expression, Flow, Object, Story, Weave};

use super::context::VariableTargetIndex;

pub(super) fn build_variable_target_index(story: &Story) -> VariableTargetIndex {
    let mut names = HashSet::new();
    collect_variable_targets_in_weave(story.root_weave(), &mut names);
    for flow in story.flows() {
        collect_variable_targets_in_flow(flow, &mut names);
    }
    names
}

fn collect_variable_targets_in_flow(flow: &Flow, names: &mut HashSet<String>) {
    for argument in flow.arguments() {
        names.insert(argument.name().to_string());
    }
    collect_variable_targets_in_weave(flow.weave(), names);
    for child in flow.child_flows() {
        collect_variable_targets_in_flow(child, names);
    }
}

fn collect_variable_targets_in_weave(weave: &Weave, names: &mut HashSet<String>) {
    for object in weave.content() {
        collect_variable_targets_in_object(object, names);
    }
}

fn collect_variable_targets_in_content_list(content: &ContentList, names: &mut HashSet<String>) {
    for object in content.objects() {
        collect_variable_targets_in_object(object, names);
    }
}

fn collect_variable_targets_in_object(object: &Object, names: &mut HashSet<String>) {
    match object {
        Object::VariableAssignment(assignment) => {
            if let Some(name) = assignment.target().variable_name() {
                names.insert(name.to_string());
            }
            if let Some(expression) = assignment.expression() {
                collect_variable_targets_in_expression(expression, names);
            }
        }
        Object::Choice(choice) => {
            if let Some(condition) = choice.condition() {
                collect_variable_targets_in_expression(condition, names);
            }
            if let Some(content) = choice.start_content() {
                collect_variable_targets_in_content_list(content, names);
            }
            if let Some(content) = choice.choice_only_content() {
                collect_variable_targets_in_content_list(content, names);
            }
            collect_variable_targets_in_content_list(choice.inner_content(), names);
        }
        Object::ContentList(content) => collect_variable_targets_in_content_list(content, names),
        Object::Conditional(conditional) => {
            if let Some(condition) = conditional.initial_condition() {
                collect_variable_targets_in_expression(condition, names);
            }
            for branch in conditional.branches() {
                if let Some(condition) = branch.own_condition() {
                    collect_variable_targets_in_expression(condition, names);
                }
                collect_variable_targets_in_weave(branch.content(), names);
            }
        }
        Object::Expression(expression) | Object::LogicLine(expression) => {
            collect_variable_targets_in_expression(expression, names);
        }
        Object::IncDec(inc_dec) => {
            collect_variable_targets_in_expression(inc_dec.expression(), names)
        }
        Object::Return(ret) => {
            if let Some(expression) = ret.returned_expression() {
                collect_variable_targets_in_expression(expression, names);
            }
        }
        Object::Sequence(sequence) => {
            for content in sequence.elements() {
                collect_variable_targets_in_content_list(content, names);
            }
        }
        Object::Weave(weave) => collect_variable_targets_in_weave(weave, names),
        Object::AuthorWarning(_)
        | Object::ConstantDeclaration(_)
        | Object::Divert(_)
        | Object::ExternalDeclaration(_)
        | Object::Gather(_)
        | Object::Glue(_)
        | Object::StructDeclaration(_)
        | Object::Tag(_)
        | Object::Text(_)
        | Object::TunnelOnwards(_) => {}
    }
}

fn collect_variable_targets_in_expression(expression: &Expression, names: &mut HashSet<String>) {
    match expression {
        Expression::StringContent(content) => {
            collect_variable_targets_in_content_list(content, names)
        }
        Expression::FunctionCall { args, .. } => {
            for arg in args {
                collect_variable_targets_in_expression(arg, names);
            }
        }
        Expression::ArrayLiteral(elements) => {
            for element in elements {
                collect_variable_targets_in_expression(element, names);
            }
        }
        Expression::StructLiteral(fields) => {
            for field in fields {
                collect_variable_targets_in_expression(field.expression(), names);
            }
        }
        Expression::FieldAccess { base, .. } => collect_variable_targets_in_expression(base, names),
        Expression::IndexAccess { base, index } => {
            collect_variable_targets_in_expression(base, names);
            collect_variable_targets_in_expression(index, names);
        }
        Expression::Binary { left, right, .. } => {
            collect_variable_targets_in_expression(left, names);
            collect_variable_targets_in_expression(right, names);
        }
        Expression::Unary { expression, .. } => {
            collect_variable_targets_in_expression(expression, names)
        }
        Expression::MultipleCondition(expressions) => {
            for expression in expressions {
                collect_variable_targets_in_expression(expression, names);
            }
        }
        Expression::String(_)
        | Expression::NumberInt(_)
        | Expression::NumberFloat(_)
        | Expression::NumberBool(_)
        | Expression::DivertTarget(_)
        | Expression::VariableReference(_) => {}
    }
}
