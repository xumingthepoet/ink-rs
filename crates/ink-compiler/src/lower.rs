use std::collections::{HashMap, HashSet};

mod conditional;
mod context;
mod expression;
mod flow;
mod indexes;
pub(crate) mod ir;
mod path;
mod sequence;
mod weave;

use crate::{
    analysis::CheckedStory,
    compiler::StageOutput,
    parsed::{Choice, Divert, DivertTarget, Expression, Object},
};

use conditional::lower_conditional_into;
use context::ChoicePathMode;
use expression::{lower_expression_into, lower_logic_line_into, lower_output_expression_into};
use flow::{lower_flow, lower_root_weave};
use indexes::{ExternalSignatures, LoweringIndexes, RuntimeLenEstimator};
use ir::{Container, ControlCommand, RuntimeObject, RuntimeProgram};
use path::{compact_path_strings_in_container, LabelIndex};
use sequence::lower_sequence;
use weave::{choice_container_prefix, lower_choice_weave, lower_content_list_into_context};

pub(crate) fn lower(story: &CheckedStory, count_all_visits: bool) -> StageOutput<RuntimeProgram> {
    let indexes = LoweringIndexes::build(
        &story.parsed,
        RuntimeLenEstimator {
            choice_content_len: estimated_choice_content_len,
            object_len: estimated_runtime_len_for_label_collection,
        },
    );
    let root_weave = story.parsed.root_weave();
    let main_content = lower_root_weave(root_weave, &indexes, count_all_visits);

    let main_container = RuntimeObject::Container(Container {
        content: main_content,
        name: None,
        flags: None,
        merge_tail_metadata: true,
    });

    let mut root_content = vec![
        main_container,
        RuntimeObject::ControlCommand(ControlCommand::Done),
    ];

    let mut named_containers = story
        .parsed
        .flows()
        .iter()
        .map(|flow| lower_flow(flow, &indexes, count_all_visits))
        .collect::<Vec<_>>();
    if let Some(global_declarations) = lower_global_declarations(&indexes) {
        named_containers.push(global_declarations);
    }
    if !named_containers.is_empty() {
        root_content.push(RuntimeObject::NamedContent(named_containers));
    }

    let mut root = Container {
        content: root_content,
        name: None,
        flags: count_all_visits.then_some(1),
        merge_tail_metadata: true,
    };
    compact_path_strings_in_container(&mut root);

    StageOutput {
        artifact: Some(RuntimeProgram { root }),
        diagnostics: Vec::new(),
    }
}

fn lower_global_declarations(indexes: &LoweringIndexes<'_>) -> Option<Container> {
    let declarations = indexes
        .variable_declarations
        .iter()
        .copied()
        .filter(|assignment| assignment.is_global())
        .collect::<Vec<_>>();

    if indexes.variable_declarations.is_empty() {
        return None;
    }

    let choice_labels = LabelIndex::new();
    let mut content = vec![RuntimeObject::ControlCommand(ControlCommand::EvalStart)];
    for declaration in declarations {
        lower_expression_into(
            &mut content,
            declaration.expression(),
            &choice_labels,
            &indexes.global_labels,
            &indexes.external_signatures,
            &indexes.constants,
            &ChoicePathMode::Root,
            false,
        );
        content.push(RuntimeObject::GlobalVariableAssignment(
            declaration.name().to_string(),
        ));
    }
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    content.push(RuntimeObject::ControlCommand(ControlCommand::End));

    Some(Container {
        content,
        name: Some("global decl".to_string()),
        flags: None,
        merge_tail_metadata: true,
    })
}

fn estimated_choice_content_len(choice: &Choice, constants: &HashMap<String, Expression>) -> usize {
    let mut content = Vec::new();
    if choice.has_start_content() {
        content.extend(choice_container_prefix(&ChoicePathMode::Root, "c-0", 0, 2));
    }
    lower_content_list_into_context(
        &mut content,
        choice.inner_content(),
        &ChoicePathMode::Root,
        &LabelIndex::new(),
        &LabelIndex::new(),
        &HashSet::new(),
        &HashMap::new(),
        constants,
    );
    content.len()
}

fn estimated_runtime_len_for_label_collection(
    object: &Object,
    constants: &HashMap<String, Expression>,
) -> usize {
    if matches!(object, Object::Weave(_)) {
        return 1;
    }

    let mut content = Vec::new();
    lower_object_into_with_context_count(
        &mut content,
        object,
        &ChoicePathMode::Root,
        &LabelIndex::new(),
        &LabelIndex::new(),
        &HashSet::new(),
        &HashMap::new(),
        constants,
        false,
    );
    content.len()
}

fn lower_object_into_with_context(
    content: &mut Vec<RuntimeObject>,
    object: &Object,
    path_mode: &ChoicePathMode,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
) {
    lower_object_into_with_context_count(
        content,
        object,
        path_mode,
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        false,
    );
}

fn lower_object_into_with_context_count(
    content: &mut Vec<RuntimeObject>,
    object: &Object,
    path_mode: &ChoicePathMode,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    count_all_visits: bool,
) {
    match object {
        Object::Text(text) => content.push(RuntimeObject::String(text.text().to_string())),
        Object::AuthorWarning(_) => {}
        Object::ContentList(content_list) => {
            lower_content_list_into_context(
                content,
                content_list,
                path_mode,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
            );
        }
        Object::Expression(expression) => lower_output_expression_into(
            content,
            expression,
            choice_labels,
            global_labels,
            external_signatures,
            constants,
            path_mode,
        ),
        Object::Conditional(conditional) => lower_conditional_into(
            content,
            conditional,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            path_mode,
        ),
        Object::LogicLine(expression) => {
            lower_logic_line_into(
                content,
                expression,
                choice_labels,
                global_labels,
                external_signatures,
                constants,
                path_mode,
            );
        }
        Object::Glue(_) => content.push(RuntimeObject::Glue),
        Object::Divert(divert) => push_divert_with_context(
            content,
            divert,
            path_mode,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
        ),
        Object::TunnelOnwards(tunnel_onwards) => {
            lower_tunnel_onwards_into(
                content,
                tunnel_onwards,
                path_mode,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
            );
        }
        Object::Choice(_) => {}
        Object::ConstantDeclaration(_) => {}
        Object::Gather(_) => {} // Handled in lower_choice_weave
        Object::VariableAssignment(assignment) => {
            lower_variable_assignment_into(
                content,
                assignment,
                path_mode,
                choice_labels,
                global_labels,
                external_signatures,
                constants,
            );
        }
        Object::IncDec(inc_dec) => {
            lower_inc_dec_into(
                content,
                inc_dec,
                path_mode,
                choice_labels,
                global_labels,
                external_signatures,
                constants,
            );
        }
        Object::Return(ret) => {
            content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
            if let Some(expr) = ret.returned_expression() {
                lower_expression_into(
                    content,
                    expr,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    false,
                );
            } else {
                content.push(RuntimeObject::Void);
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
            content.push(RuntimeObject::ControlCommand(ControlCommand::PopFunction));
        }
        Object::Tag(tag) => content.push(RuntimeObject::Tag {
            is_start: tag.is_start(),
        }),
        Object::Sequence(sequence) => content.push(RuntimeObject::Container(lower_sequence(
            sequence,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            path_mode,
            &path_mode.sequence_container_path(content.len()),
        ))),
        Object::Weave(weave) => {
            let nested_path_mode = path_mode.for_nested_weave(content.len());
            content.push(RuntimeObject::Container(Container {
                content: lower_choice_weave(
                    weave,
                    nested_path_mode,
                    global_labels,
                    global_variables,
                    external_signatures,
                    constants,
                    count_all_visits,
                ),
                name: None,
                flags: None,
                merge_tail_metadata: true,
            }));
        }
        Object::ExternalDeclaration(_) => {}
    }
}

fn lower_variable_assignment_into(
    content: &mut Vec<RuntimeObject>,
    assignment: &crate::parsed::VariableAssignment,
    path_mode: &ChoicePathMode,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
) {
    if assignment.is_global() {
        return;
    }

    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    lower_expression_into(
        content,
        assignment.expression(),
        choice_labels,
        global_labels,
        external_signatures,
        constants,
        path_mode,
        false,
    );
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));

    if assignment.is_temporary() {
        content.push(RuntimeObject::VariableAssignment(
            assignment.name().to_string(),
        ));
    } else if path_mode.is_local_variable(assignment.name()) {
        content.push(RuntimeObject::TempVariableReassignment(
            assignment.name().to_string(),
        ));
    } else {
        content.push(RuntimeObject::VariableReassignment(
            assignment.name().to_string(),
        ));
    }
}

fn lower_inc_dec_into(
    content: &mut Vec<RuntimeObject>,
    inc_dec: &crate::parsed::IncDec,
    path_mode: &ChoicePathMode,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
) {
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    content.push(RuntimeObject::VariableReference(inc_dec.name().to_string()));
    lower_expression_into(
        content,
        inc_dec.expression(),
        choice_labels,
        global_labels,
        external_signatures,
        constants,
        path_mode,
        false,
    );
    content.push(RuntimeObject::NativeFunction(
        if inc_dec.is_increment() { "+" } else { "-" }.to_string(),
    ));
    if path_mode.is_local_variable(inc_dec.name()) {
        content.push(RuntimeObject::TempVariableReassignment(
            inc_dec.name().to_string(),
        ));
    } else {
        content.push(RuntimeObject::VariableReassignment(
            inc_dec.name().to_string(),
        ));
    }
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
}

fn push_divert_with_context(
    content: &mut Vec<RuntimeObject>,
    divert: &Divert,
    path_mode: &ChoicePathMode,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
) {
    if !divert.arguments().is_empty() {
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
        for argument in divert.arguments() {
            lower_expression_into(
                content,
                argument,
                choice_labels,
                global_labels,
                external_signatures,
                constants,
                path_mode,
                false,
            );
        }
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    }

    if divert.is_thread() {
        content.push(RuntimeObject::ControlCommand(ControlCommand::StartThread));
    }

    match divert.target() {
        DivertTarget::Done => content.push(RuntimeObject::ControlCommand(ControlCommand::Done)),
        DivertTarget::End => content.push(RuntimeObject::ControlCommand(ControlCommand::End)),
        DivertTarget::Path(target) => {
            let resolved_target = if let Some(choice_target) = choice_labels.get(target) {
                runtime_divert(choice_target.to_string(), false, divert.is_tunnel())
            } else if let Some(label_target) = path_mode
                .scoped_label_target(target, global_labels)
                .filter(|label_target| *label_target != target)
            {
                runtime_divert(
                    path_mode.resolve_label_target(label_target),
                    false,
                    divert.is_tunnel(),
                )
            } else if path_mode.is_local_variable(target) || global_variables.contains(target) {
                runtime_divert(target.clone(), true, divert.is_tunnel())
            } else {
                runtime_divert(
                    path_mode.resolve_divert_target(target),
                    false,
                    divert.is_tunnel(),
                )
            };
            content.push(resolved_target);
        }
        DivertTarget::Empty => {
            content.push(runtime_divert(String::new(), false, divert.is_tunnel()))
        }
    }
}

fn runtime_divert(target: String, variable: bool, is_tunnel: bool) -> RuntimeObject {
    if is_tunnel {
        RuntimeObject::TunnelDivert { target, variable }
    } else {
        RuntimeObject::Divert { target, variable }
    }
}

fn lower_tunnel_onwards_into(
    content: &mut Vec<RuntimeObject>,
    tunnel_onwards: &crate::parsed::TunnelOnwards,
    path_mode: &ChoicePathMode,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
) {
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    for argument in tunnel_onwards.arguments() {
        lower_expression_into(
            content,
            argument,
            choice_labels,
            global_labels,
            external_signatures,
            constants,
            path_mode,
            false,
        );
    }
    if let Some(target) = tunnel_onwards.override_target() {
        match target {
            DivertTarget::Path(target) => {
                if let Some(choice_target) = choice_labels.get(target) {
                    content.push(RuntimeObject::DivertTarget(choice_target.to_string()));
                } else if let Some(label_target) = path_mode
                    .scoped_label_target(target, global_labels)
                    .filter(|label_target| *label_target != target)
                {
                    content.push(RuntimeObject::DivertTarget(
                        path_mode.resolve_label_target(label_target),
                    ));
                } else if path_mode.is_local_variable(target) || global_variables.contains(target) {
                    content.push(RuntimeObject::VariableReference(target.clone()));
                } else {
                    content.push(RuntimeObject::DivertTarget(
                        path_mode.resolve_divert_target(target),
                    ));
                }
            }
            DivertTarget::Done => content.push(RuntimeObject::DivertTarget("DONE".to_string())),
            DivertTarget::End => content.push(RuntimeObject::DivertTarget("END".to_string())),
            DivertTarget::Empty => content.push(RuntimeObject::Void),
        }
    } else {
        content.push(RuntimeObject::Void);
    }
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    content.push(RuntimeObject::ControlCommand(ControlCommand::PopTunnel));
}

fn ends_with_flow_terminator(content: &[RuntimeObject]) -> bool {
    content
        .iter()
        .rev()
        .find(|object| !matches!(object, RuntimeObject::String(text) if text == "\n"))
        .is_some_and(|object| {
            matches!(
                object,
                RuntimeObject::Divert { .. }
                    | RuntimeObject::ControlCommand(ControlCommand::End | ControlCommand::Done)
            )
        })
}

fn ends_with_end_or_done(content: &[RuntimeObject]) -> bool {
    content
        .iter()
        .rev()
        .find(|object| !matches!(object, RuntimeObject::String(text) if text == "\n"))
        .is_some_and(|object| {
            matches!(
                object,
                RuntimeObject::ControlCommand(ControlCommand::End | ControlCommand::Done)
            )
        })
}

fn done_container(name: &str, count_all_visits: bool) -> Container {
    Container {
        content: vec![RuntimeObject::ControlCommand(ControlCommand::Done)],
        name: Some(name.to_string()),
        flags: count_all_visits.then_some(5),
        merge_tail_metadata: true,
    }
}
