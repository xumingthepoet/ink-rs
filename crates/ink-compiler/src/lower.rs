use std::collections::{HashMap, HashSet};

mod context;
mod expression;
mod flow;
mod indexes;
pub(crate) mod ir;
mod path;
mod weave;

use crate::{
    analysis::CheckedStory,
    compiler::StageOutput,
    parsed::{
        Choice, Conditional, Divert, DivertTarget, Expression, Object, Sequence, SequenceType,
        Weave,
    },
};

use context::ChoicePathMode;
use expression::{lower_expression_into, lower_logic_line_into, lower_output_expression_into};
use flow::{lower_flow, lower_root_weave};
use indexes::{ExternalSignatures, LoweringIndexes, RuntimeLenEstimator};
use ir::{Container, ControlCommand, RuntimeObject, RuntimeProgram};
use path::{
    canonical_runtime_path, child_path, compact_path_string, compact_relative_path,
    is_absolute_runtime_path, is_user_named_path_component, semantic_path_key,
};
use weave::{
    choice_container_prefix, content_list_has_choice, lower_choice_weave,
    lower_choice_weave_with_initial_content, lower_content_list_into_context, weave_has_choice,
};

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

    let choice_labels = HashMap::new();
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
        &HashMap::new(),
        &HashMap::new(),
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
        &HashMap::new(),
        &HashMap::new(),
        &HashSet::new(),
        &HashMap::new(),
        constants,
        false,
    );
    content.len()
}

fn compact_path_strings_in_container(container: &mut Container) {
    let semantic_paths = build_semantic_path_index(container);
    compact_path_strings_in_container_at(container, "", &semantic_paths);
}

fn compact_path_strings_in_container_at(
    container: &mut Container,
    container_path: &str,
    semantic_paths: &HashMap<String, Option<String>>,
) {
    let mut content_index = 0;
    for object in &mut container.content {
        if matches!(object, RuntimeObject::NamedContent(_)) {
            compact_path_strings_in_named_content(object, container_path, semantic_paths);
            continue;
        }

        let object_path = match object {
            RuntimeObject::Container(container) => container
                .name
                .as_deref()
                .map(|name| child_path(container_path, name))
                .unwrap_or_else(|| child_path(container_path, &content_index.to_string())),
            _ => child_path(container_path, &content_index.to_string()),
        };
        compact_path_strings_in_object(object, &object_path, semantic_paths);
        content_index += 1;
    }
}

fn compact_path_strings_in_named_content(
    object: &mut RuntimeObject,
    container_path: &str,
    semantic_paths: &HashMap<String, Option<String>>,
) {
    let RuntimeObject::NamedContent(containers) = object else {
        return;
    };

    for container in containers {
        let Some(name) = container.name.clone() else {
            continue;
        };
        let path = child_path(container_path, &name);
        compact_path_strings_in_container_at(container, &path, semantic_paths);
    }
}

fn compact_path_strings_in_object(
    object: &mut RuntimeObject,
    object_path: &str,
    semantic_paths: &HashMap<String, Option<String>>,
) {
    match object {
        RuntimeObject::Container(container) => {
            compact_path_strings_in_container_at(container, object_path, semantic_paths);
        }
        RuntimeObject::NamedContent(_) => {
            compact_path_strings_in_named_content(object, object_path, semantic_paths)
        }
        RuntimeObject::Divert { target, variable }
        | RuntimeObject::TunnelDivert { target, variable } => {
            if !*variable {
                compact_target_path_string(target, object_path, semantic_paths);
            }
        }
        RuntimeObject::ConditionalDivert { target }
        | RuntimeObject::ReadCount(target)
        | RuntimeObject::ChoicePoint { target, .. } => {
            compact_target_path_string(target, object_path, semantic_paths);
        }
        _ => {}
    }
}

fn compact_target_path_string(
    target: &mut String,
    object_path: &str,
    semantic_paths: &HashMap<String, Option<String>>,
) {
    if !is_absolute_runtime_path(target) {
        return;
    }
    if let Some(canonical) = canonical_runtime_path(target, semantic_paths) {
        *target = canonical;
    }
    *target = compact_path_string(object_path, target);
}

fn build_semantic_path_index(container: &Container) -> HashMap<String, Option<String>> {
    let mut paths = HashMap::new();
    collect_semantic_paths(container, "", &mut paths);
    paths
}

fn collect_semantic_paths(
    container: &Container,
    container_path: &str,
    paths: &mut HashMap<String, Option<String>>,
) {
    let mut content_index = 0;
    for object in &container.content {
        if let RuntimeObject::NamedContent(containers) = object {
            for container in containers {
                let Some(name) = container.name.as_deref() else {
                    continue;
                };
                let path = child_path(container_path, name);
                collect_semantic_path_for_container(container, &path, paths);
            }
            continue;
        }

        let object_path = match object {
            RuntimeObject::Container(container) => container
                .name
                .as_deref()
                .map(|name| child_path(container_path, name))
                .unwrap_or_else(|| child_path(container_path, &content_index.to_string())),
            _ => child_path(container_path, &content_index.to_string()),
        };

        if let RuntimeObject::Container(container) = object {
            collect_semantic_path_for_container(container, &object_path, paths);
        }
        content_index += 1;
    }
}

fn collect_semantic_path_for_container(
    container: &Container,
    path: &str,
    paths: &mut HashMap<String, Option<String>>,
) {
    if container
        .name
        .as_deref()
        .is_some_and(is_user_named_path_component)
    {
        if let Some(key) = semantic_path_key(path) {
            paths
                .entry(key)
                .and_modify(|entry| {
                    if entry.as_deref() != Some(path) {
                        *entry = None;
                    }
                })
                .or_insert_with(|| Some(path.to_string()));
        }
    }
    collect_semantic_paths(container, path, paths);
}

fn lower_sequence(
    sequence: &Sequence,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
    sequence_container_path: &str,
) -> Container {
    let mut content = vec![
        RuntimeObject::ControlCommand(ControlCommand::EvalStart),
        RuntimeObject::ControlCommand(ControlCommand::VisitIndex),
    ];

    let sequence_type = sequence.sequence_type();
    let once = sequence_type.contains(SequenceType::ONCE);
    let cycle = sequence_type.contains(SequenceType::CYCLE);
    let stopping = sequence_type.contains(SequenceType::STOPPING);
    let shuffle = sequence_type.contains(SequenceType::SHUFFLE);
    let branch_count = sequence.elements().len() + usize::from(once);

    if stopping || once {
        content.push(RuntimeObject::Int(branch_count.saturating_sub(1) as i32));
        content.push(RuntimeObject::NativeFunction("MIN".to_string()));
    } else if cycle {
        content.push(RuntimeObject::Int(sequence.elements().len() as i32));
        content.push(RuntimeObject::NativeFunction("%".to_string()));
    }

    if shuffle {
        if once || stopping {
            let last_index = if stopping {
                sequence.elements().len().saturating_sub(1)
            } else {
                sequence.elements().len()
            };
            let post_shuffle_noop_index = content.len() + 6;
            content.extend([
                RuntimeObject::ControlCommand(ControlCommand::Duplicate),
                RuntimeObject::Int(last_index as i32),
                RuntimeObject::NativeFunction("==".to_string()),
                RuntimeObject::ConditionalDivert {
                    target: format!(".^.{post_shuffle_noop_index}"),
                },
            ]);
        }

        let element_count_to_shuffle = sequence.elements().len() - usize::from(stopping);
        content.push(RuntimeObject::Int(element_count_to_shuffle as i32));
        content.push(RuntimeObject::ControlCommand(
            ControlCommand::SequenceShuffleIndex,
        ));
        if once || stopping {
            content.push(RuntimeObject::ControlCommand(ControlCommand::NoOp));
        }
    }

    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));

    for index in 0..branch_count {
        content.extend([
            RuntimeObject::ControlCommand(ControlCommand::EvalStart),
            RuntimeObject::ControlCommand(ControlCommand::Duplicate),
            RuntimeObject::Int(index as i32),
            RuntimeObject::NativeFunction("==".to_string()),
            RuntimeObject::ControlCommand(ControlCommand::EvalEnd),
            RuntimeObject::ConditionalDivert {
                target: format!(".^.s{index}"),
            },
        ]);
    }

    let post_sequence_index = content.len();
    content.push(RuntimeObject::ControlCommand(ControlCommand::NoOp));

    let branch_containers = sequence
        .elements()
        .iter()
        .map(Some)
        .chain((branch_count > sequence.elements().len()).then_some(None))
        .enumerate()
        .map(|(index, element)| {
            let branch_name = format!("s{index}");
            let branch_path_mode =
                path_mode.for_sequence_branch(sequence_container_path, &branch_name);
            let mut branch_content = vec![RuntimeObject::ControlCommand(ControlCommand::Pop)];
            if let Some(element) = element {
                if content_list_has_choice(element) {
                    let element_weave = Weave::new(element.objects().to_vec(), 0);
                    branch_content = lower_choice_weave_with_initial_content(
                        &element_weave,
                        branch_path_mode.clone(),
                        global_labels,
                        global_variables,
                        external_signatures,
                        constants,
                        false,
                        branch_content,
                    );
                } else {
                    lower_content_list_into_context(
                        &mut branch_content,
                        element,
                        &branch_path_mode,
                        choice_labels,
                        global_labels,
                        global_variables,
                        external_signatures,
                        constants,
                    );
                }
            }
            let relative_return_target = format!(".^.^.{post_sequence_index}");
            let global_return_target = format!("{sequence_container_path}.{post_sequence_index}");
            let trailing_named_content =
                if matches!(branch_content.last(), Some(RuntimeObject::NamedContent(_))) {
                    branch_content.pop()
                } else {
                    None
                };
            branch_content.push(RuntimeObject::Divert {
                target: compact_relative_path(&relative_return_target, &global_return_target),
                variable: false,
            });
            if let Some(named_content) = trailing_named_content {
                branch_content.push(named_content);
            }
            Container {
                content: branch_content,
                name: Some(branch_name),
                flags: None,
                merge_tail_metadata: true,
            }
        })
        .collect::<Vec<_>>();
    content.push(RuntimeObject::NamedContent(branch_containers));

    Container {
        content,
        name: None,
        flags: Some(5),
        merge_tail_metadata: true,
    }
}

fn lower_conditional_into(
    content: &mut Vec<RuntimeObject>,
    conditional: &Conditional,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
) {
    if let Some(condition) = conditional.initial_condition() {
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
        lower_expression_into(
            content,
            condition,
            choice_labels,
            global_labels,
            external_signatures,
            constants,
            path_mode,
            false,
        );
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    }

    let has_initial_condition = conditional.initial_condition().is_some();
    let switch_like = has_initial_condition
        && conditional
            .branches()
            .iter()
            .any(|branch| branch.own_condition().is_some());
    let needs_fallthrough_pop = switch_like
        && !conditional
            .branches()
            .last()
            .is_some_and(|branch| branch.is_else());
    let rejoin_index =
        content.len() + conditional.branches().len() + usize::from(needs_fallthrough_pop);
    let rejoin_target = path_mode.runtime_index_path(rejoin_index);
    let branch_rejoin_target = rejoin_target;

    for branch in conditional.branches() {
        let branch_path_mode = path_mode.for_conditional_branch(content.len());
        let mut branch_content = Vec::new();
        let duplicates_stack_value = switch_like && !branch.is_else();
        if duplicates_stack_value {
            branch_content.push(RuntimeObject::ControlCommand(ControlCommand::Duplicate));
        }

        if !branch.is_true_branch() && !branch.is_else() {
            let needs_eval = branch.own_condition().is_some();
            if needs_eval {
                branch_content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
            }
            if let Some(condition) = branch.own_condition() {
                lower_expression_into(
                    &mut branch_content,
                    condition,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    false,
                );
            }
            if switch_like {
                branch_content.push(RuntimeObject::NativeFunction("==".to_string()));
            }
            if needs_eval {
                branch_content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
            }
        }

        if branch.is_else() {
            branch_content.push(RuntimeObject::Divert {
                target: ".^.b".to_string(),
                variable: false,
            });
        } else {
            branch_content.push(RuntimeObject::ConditionalDivert {
                target: ".^.b".to_string(),
            });
        }

        if weave_has_choice(branch.content()) {
            let initial_content = if branch.is_inline() {
                if duplicates_stack_value || (branch.is_else() && switch_like) {
                    vec![RuntimeObject::ControlCommand(ControlCommand::Pop)]
                } else {
                    Vec::new()
                }
            } else {
                let mut initial_content = Vec::new();
                if duplicates_stack_value || (branch.is_else() && switch_like) {
                    initial_content.push(RuntimeObject::ControlCommand(ControlCommand::Pop));
                }
                initial_content.push(RuntimeObject::String("\n".to_string()));
                initial_content
            };
            let mut content_container = Vec::new();
            let mut lowered_branch = lower_choice_weave_with_initial_content(
                branch.content(),
                branch_path_mode,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                false,
                initial_content,
            );
            let trailing_named_content =
                if matches!(lowered_branch.last(), Some(RuntimeObject::NamedContent(_))) {
                    lowered_branch.pop()
                } else {
                    None
                };
            content_container.extend(lowered_branch);
            content_container.push(RuntimeObject::Divert {
                target: branch_rejoin_target.clone(),
                variable: false,
            });
            if let Some(named_content) = trailing_named_content {
                content_container.push(named_content);
            }
            branch_content.push(RuntimeObject::NamedContent(vec![Container {
                content: content_container,
                name: Some("b".to_string()),
                flags: None,
                merge_tail_metadata: true,
            }]));
        } else {
            let mut content_container = Vec::new();
            if duplicates_stack_value || (branch.is_else() && switch_like) {
                content_container.push(RuntimeObject::ControlCommand(ControlCommand::Pop));
            }
            if !branch.is_inline() {
                content_container.push(RuntimeObject::String("\n".to_string()));
            }
            for object in branch.content().content() {
                lower_object_into_with_context(
                    &mut content_container,
                    object,
                    &branch_path_mode,
                    choice_labels,
                    global_labels,
                    global_variables,
                    external_signatures,
                    constants,
                );
            }
            content_container.push(RuntimeObject::Divert {
                target: branch_rejoin_target.clone(),
                variable: false,
            });
            branch_content.push(RuntimeObject::NamedContent(vec![Container {
                content: content_container,
                name: Some("b".to_string()),
                flags: None,
                merge_tail_metadata: true,
            }]));
        }

        content.push(RuntimeObject::Container(Container {
            content: branch_content,
            name: None,
            flags: None,
            merge_tail_metadata: true,
        }));
    }

    if needs_fallthrough_pop {
        content.push(RuntimeObject::ControlCommand(ControlCommand::Pop));
    }
    content.push(RuntimeObject::ControlCommand(ControlCommand::NoOp));
}

fn lower_object_into_with_context(
    content: &mut Vec<RuntimeObject>,
    object: &Object,
    path_mode: &ChoicePathMode,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
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
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
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
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
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
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
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
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
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
                runtime_divert(choice_target.clone(), false, divert.is_tunnel())
            } else if let Some(label_target) = path_mode
                .scoped_label_target(target, global_labels)
                .filter(|label_target| label_target.as_str() != target)
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
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
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
                    content.push(RuntimeObject::DivertTarget(choice_target.clone()));
                } else if let Some(label_target) = path_mode
                    .scoped_label_target(target, global_labels)
                    .filter(|label_target| label_target.as_str() != target)
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
