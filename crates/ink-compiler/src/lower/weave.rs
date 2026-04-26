use std::collections::HashSet;

use ink_story_json_format::{Container, ControlCommand, NamedContainer, Object as RuntimeObject};

use crate::parsed::{Choice, ContentList, Object, Weave};

use super::context::ChoicePathMode;
use super::expression::lower_expression_into;
use super::indexes::{
    collect_counted_paths_in_weave, ConstantValues, CountedFlowPaths, ExternalSignatures,
    StructDefinitions,
};
use super::path::{child_path, LabelIndex};
use super::{
    done_container, ends_with_end_or_done, lower_object_into_with_context,
    lower_object_into_with_context_count, named_container, named_content,
};

enum ChoiceOuter {
    Inline(Vec<RuntimeObject>),
    Nested(Container),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GatherLocation {
    Main(Vec<usize>),
    Named(Vec<usize>),
}

impl GatherLocation {
    fn child(&self, child_index: usize) -> Self {
        match self {
            GatherLocation::Main(path) => {
                let mut path = path.clone();
                path.push(child_index);
                GatherLocation::Main(path)
            }
            GatherLocation::Named(path) => {
                let mut path = path.clone();
                path.push(child_index);
                GatherLocation::Named(path)
            }
        }
    }
}

fn named_container_flags(
    count_visits: bool,
    count_turns: bool,
    force_named_flag: bool,
) -> Option<i32> {
    let mut flags = 0;
    if count_visits {
        flags |= 1;
    }
    if count_turns {
        flags |= 2;
    }
    if force_named_flag || count_visits || count_turns {
        flags |= 4;
    }

    (flags != 0).then_some(flags)
}

pub(super) fn weave_has_choice(weave: &Weave) -> bool {
    weave
        .content()
        .iter()
        .any(|object| matches!(object, Object::Choice(_)))
}

pub(super) fn weave_has_weave_points(weave: &Weave) -> bool {
    weave
        .content()
        .iter()
        .any(|object| matches!(object, Object::Choice(_) | Object::Gather(_)))
}

pub(super) fn content_list_has_choice(content_list: &ContentList) -> bool {
    content_list
        .objects()
        .iter()
        .any(|object| matches!(object, Object::Choice(_)))
}

pub(super) fn lower_linear_weave(
    weave: &Weave,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
) -> Vec<RuntimeObject> {
    lower_linear_weave_with_context(
        weave,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
        &ChoicePathMode::Root,
    )
}

fn lower_linear_weave_with_context(
    weave: &Weave,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    path_mode: &ChoicePathMode,
) -> Vec<RuntimeObject> {
    let mut content = Vec::new();
    lower_linear_weave_into_context(
        &mut content,
        weave,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
        path_mode,
    );
    content
}

pub(super) fn lower_linear_weave_into_context(
    content: &mut Vec<RuntimeObject>,
    weave: &Weave,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    path_mode: &ChoicePathMode,
) {
    let choice_labels = LabelIndex::new();
    for object in weave.content() {
        lower_object_into_with_context(
            content,
            object,
            path_mode,
            &choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            struct_definitions,
        );
    }
}

pub(super) fn lower_choice_weave(
    weave: &Weave,
    path_mode: ChoicePathMode,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    count_all_visits: bool,
) -> Container {
    lower_choice_weave_with_initial_content(
        weave,
        path_mode,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
        count_all_visits,
        Vec::new(),
    )
}

pub(super) fn lower_choice_weave_with_initial_content(
    weave: &Weave,
    path_mode: ChoicePathMode,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    count_all_visits: bool,
    initial_content: Vec<RuntimeObject>,
) -> Container {
    let mut main_content = initial_content;
    let mut named_content = Vec::new();
    let mut index = 0;
    let mut gather_count = 0;
    let mut choice_count = 0;
    let mut needs_terminal_gather = false;
    let mut last_gather_location = None;
    let mut last_section_had_choice = false;
    let mut current_path_mode = path_mode.clone();
    let objects = weave.content();
    let mut choice_labels = collect_local_weave_labels(objects, &path_mode);
    let mut counted_paths = CountedFlowPaths::default();
    collect_counted_paths_in_weave(
        weave,
        global_labels,
        constants,
        global_variables,
        &path_mode,
        &mut counted_paths,
    );

    let has_explicit_gather = objects.iter().any(|o| matches!(o, Object::Gather(_)));

    while index < objects.len() {
        match &objects[index] {
            Object::AuthorWarning(_)
            | Object::Text(_)
            | Object::ContentList(_)
            | Object::Expression(_)
            | Object::Conditional(_)
            | Object::ConstantDeclaration(_)
            | Object::LogicLine(_)
            | Object::Glue(_)
            | Object::Divert(_)
            | Object::TunnelOnwards(_)
            | Object::Tag(_)
            | Object::Sequence(_)
            | Object::IncDec(_)
            | Object::VariableAssignment(_)
            | Object::ExternalDeclaration(_)
            | Object::StructDeclaration(_)
            | Object::Return(_)
            | Object::Weave(_) => {
                last_section_had_choice = lower_weave_section(
                    objects,
                    &mut index,
                    &mut main_content,
                    &mut named_content,
                    &mut choice_count,
                    &mut needs_terminal_gather,
                    &mut choice_labels,
                    global_labels,
                    global_variables,
                    external_signatures,
                    constants,
                    struct_definitions,
                    gather_count,
                    &current_path_mode,
                    &path_mode,
                    has_explicit_gather,
                    count_all_visits,
                    &counted_paths,
                );
            }
            Object::Gather(_gather) => {
                let gather = match &objects[index] {
                    Object::Gather(gather) => gather,
                    _ => unreachable!(),
                };
                let gather_name = if let Some(identifier) = gather.identifier() {
                    identifier.to_string()
                } else {
                    let gather_name = format!("g-{gather_count}");
                    gather_count += 1;
                    gather_name
                };
                let auto_enter_gather = !last_section_had_choice;

                let mut gather_content = Vec::new();
                let mut gather_named_content = Vec::new();
                index += 1;
                let section_start = index;

                let gather_path_mode = if auto_enter_gather {
                    current_path_mode.for_gather(&gather_name)
                } else {
                    path_mode.for_gather(&gather_name)
                };
                if let Some(identifier) = gather.identifier() {
                    choice_labels.insert(identifier.to_string(), gather_path_mode.container_path());
                }
                let gather_has_choice = lower_weave_section(
                    objects,
                    &mut index,
                    &mut gather_content,
                    &mut gather_named_content,
                    &mut choice_count,
                    &mut needs_terminal_gather,
                    &mut choice_labels,
                    global_labels,
                    global_variables,
                    external_signatures,
                    constants,
                    struct_definitions,
                    gather_count,
                    &gather_path_mode,
                    &path_mode,
                    has_explicit_gather,
                    count_all_visits,
                    &counted_paths,
                );
                if !gather_has_choice && !ends_with_end_or_done(&gather_content) {
                    if let Some(target) = gather_path_mode.fallback_gather_target() {
                        gather_content.push(RuntimeObject::Divert {
                            target,
                            variable: false,
                        });
                    }
                }

                let gather_container = Container {
                    content: gather_content,
                    named_content: gather_named_content,
                    name: Some(gather_name),
                    flags: named_container_flags(
                        count_all_visits
                            || gather.identifier().is_some()
                            || counted_paths
                                .visits
                                .contains(&gather_path_mode.container_path()),
                        counted_paths
                            .turns
                            .contains(&gather_path_mode.container_path()),
                        gather.identifier().is_some(),
                    ),
                };
                if auto_enter_gather {
                    if let Some(location) = last_gather_location.clone() {
                        if let Some(parent) = gather_container_at_location_mut(
                            &mut main_content,
                            &mut named_content,
                            &location,
                        ) {
                            let child_index = parent.content.len();
                            parent
                                .content
                                .push(RuntimeObject::Container(gather_container));
                            last_gather_location = Some(location.child(child_index));
                        }
                    } else {
                        let child_index = main_content.len();
                        main_content.push(RuntimeObject::Container(gather_container));
                        last_gather_location = Some(GatherLocation::Main(vec![child_index]));
                    }
                } else {
                    named_content.push(named_container(gather_container));
                    last_gather_location =
                        Some(GatherLocation::Named(vec![named_content.len() - 1]));
                }
                current_path_mode = gather_path_mode;
                last_section_had_choice =
                    gather_has_choice || section_contains_choice(objects, section_start, index);
            }
            Object::Choice(_) => {
                last_section_had_choice = lower_weave_section(
                    objects,
                    &mut index,
                    &mut main_content,
                    &mut named_content,
                    &mut choice_count,
                    &mut needs_terminal_gather,
                    &mut choice_labels,
                    global_labels,
                    global_variables,
                    external_signatures,
                    constants,
                    struct_definitions,
                    gather_count,
                    &current_path_mode,
                    &path_mode,
                    has_explicit_gather,
                    count_all_visits,
                    &counted_paths,
                );
            }
        }
    }

    if path_mode.is_root() && has_explicit_gather {
        if let Some(location) = last_gather_location {
            if let Some(container) =
                gather_container_at_location_mut(&mut main_content, &mut named_content, &location)
            {
                container
                    .content
                    .push(RuntimeObject::Container(done_container(
                        &format!("g-{gather_count}"),
                        count_all_visits,
                    )));
            }
        }
    } else if !path_mode.is_nested_root()
        && !has_explicit_gather
        && needs_terminal_gather
        && path_mode.fallback_gather_target().is_none()
    {
        named_content.push(named_container(done_container(
            &format!("g-{gather_count}"),
            count_all_visits,
        )));
    }

    Container {
        content: main_content,
        named_content,
        name: None,
        flags: None,
    }
}

fn lower_weave_section(
    objects: &[Object],
    index: &mut usize,
    content: &mut Vec<RuntimeObject>,
    named_content: &mut Vec<NamedContainer>,
    choice_count: &mut usize,
    needs_terminal_gather: &mut bool,
    choice_labels: &mut LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    gather_count: usize,
    path_mode: &ChoicePathMode,
    weave_path_mode: &ChoicePathMode,
    has_explicit_gather: bool,
    count_all_visits: bool,
    counted_paths: &CountedFlowPaths,
) -> bool {
    let mut section_has_choice = false;
    while *index < objects.len() {
        match &objects[*index] {
            Object::Gather(_) => break,
            Object::AuthorWarning(_)
            | Object::Text(_)
            | Object::ContentList(_)
            | Object::Expression(_)
            | Object::Conditional(_)
            | Object::ConstantDeclaration(_)
            | Object::LogicLine(_)
            | Object::Glue(_)
            | Object::Divert(_)
            | Object::TunnelOnwards(_)
            | Object::Tag(_)
            | Object::Sequence(_)
            | Object::IncDec(_)
            | Object::VariableAssignment(_)
            | Object::ExternalDeclaration(_)
            | Object::StructDeclaration(_)
            | Object::Return(_)
            | Object::Weave(_) => {
                lower_object_into_with_context_count(
                    content,
                    &objects[*index],
                    path_mode,
                    choice_labels,
                    global_labels,
                    global_variables,
                    external_signatures,
                    constants,
                    struct_definitions,
                    count_all_visits,
                );
                *index += 1;
            }
            Object::Choice(_) => {
                section_has_choice = true;
                lower_choice_in_section(
                    objects,
                    index,
                    content,
                    named_content,
                    choice_count,
                    needs_terminal_gather,
                    choice_labels,
                    global_labels,
                    global_variables,
                    external_signatures,
                    constants,
                    struct_definitions,
                    gather_count,
                    path_mode,
                    weave_path_mode,
                    has_explicit_gather,
                    count_all_visits,
                    counted_paths,
                );
            }
        }
    }
    section_has_choice
}

fn section_contains_choice(objects: &[Object], start: usize, end: usize) -> bool {
    objects[start..end]
        .iter()
        .any(|object| matches!(object, Object::Choice(_)))
}

fn gather_container_at_location_mut<'a>(
    main_content: &'a mut Vec<RuntimeObject>,
    named_content: &'a mut [NamedContainer],
    location: &GatherLocation,
) -> Option<&'a mut Container> {
    match location {
        GatherLocation::Main(path) => nested_container_in_runtime_content_mut(main_content, path),
        GatherLocation::Named(path) => nested_container_in_named_content_mut(named_content, path),
    }
}

fn nested_container_in_named_content_mut<'a>(
    named_content: &'a mut [NamedContainer],
    path: &[usize],
) -> Option<&'a mut Container> {
    let (&first, rest) = path.split_first()?;
    let named = named_content.get_mut(first)?;
    nested_container_in_container_mut(&mut named.container, rest)
}

fn nested_container_in_runtime_content_mut<'a>(
    content: &'a mut Vec<RuntimeObject>,
    path: &[usize],
) -> Option<&'a mut Container> {
    let (&first, rest) = path.split_first()?;
    let RuntimeObject::Container(container) = content.get_mut(first)? else {
        return None;
    };
    nested_container_in_container_mut(container, rest)
}

fn nested_container_in_container_mut<'a>(
    container: &'a mut Container,
    path: &[usize],
) -> Option<&'a mut Container> {
    let Some((&first, rest)) = path.split_first() else {
        return Some(container);
    };
    let RuntimeObject::Container(child) = container.content.get_mut(first)? else {
        return None;
    };
    nested_container_in_container_mut(child, rest)
}

fn lower_choice_in_section(
    objects: &[Object],
    index: &mut usize,
    content: &mut Vec<RuntimeObject>,
    named_content: &mut Vec<NamedContainer>,
    choice_count: &mut usize,
    needs_terminal_gather: &mut bool,
    choice_labels: &mut LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    gather_count: usize,
    path_mode: &ChoicePathMode,
    weave_path_mode: &ChoicePathMode,
    has_explicit_gather: bool,
    count_all_visits: bool,
    counted_paths: &CountedFlowPaths,
) {
    let Object::Choice(choice) = &objects[*index] else {
        return;
    };

    let choice_index = *choice_count;
    let has_following_gather = objects[*index + 1..]
        .iter()
        .any(|object| matches!(object, Object::Gather(_)));
    *choice_count += 1;
    let choice_container_name = format!("c-{choice_index}");
    let gather_container_name = next_gather_name(objects, *index + 1, gather_count);
    let choice_container_path = path_mode.choice_point_target(choice_index);

    match choice_outer(
        choice,
        &choice_container_path,
        content.len(),
        path_mode,
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
    ) {
        ChoiceOuter::Inline(objects) => content.extend(objects),
        ChoiceOuter::Nested(container) => content.push(RuntimeObject::Container(container)),
    }

    let mut choice_content = Vec::new();
    let mut nested_choice_content_path_mode = path_mode.for_choice_nested_content(
        &choice_container_name,
        &gather_container_name,
        has_following_gather,
    );
    if has_following_gather {
        nested_choice_content_path_mode.set_flow_fallback_gather_target(
            weave_path_mode.absolute_child_path(&gather_container_name),
        );
    }
    if choice.has_start_content() {
        choice_content =
            choice_container_prefix(path_mode, &choice_container_name, content.len() - 1, 2);
    }
    lower_content_list_into_context(
        &mut choice_content,
        choice.inner_content(),
        &nested_choice_content_path_mode,
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
    );
    let mut has_nested_weave_content = false;

    *index += 1;
    while *index < objects.len() {
        if matches!(objects[*index], Object::Choice(_) | Object::Gather(_)) {
            break;
        }
        if matches!(objects[*index], Object::Weave(_)) {
            has_nested_weave_content = true;
        }
        lower_object_into_with_context_count(
            &mut choice_content,
            &objects[*index],
            &nested_choice_content_path_mode,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            struct_definitions,
            count_all_visits,
        );
        *index += 1;
    }

    let include_gather = !has_nested_weave_content
        && (has_following_gather || path_mode.should_include_choice_gather());
    if include_gather
        && !(has_explicit_gather && !has_following_gather && ends_with_end_or_done(&choice_content))
    {
        choice_content.push(RuntimeObject::Divert {
            target: weave_path_mode.gather_target(&gather_container_name, has_following_gather),
            variable: false,
        });
        *needs_terminal_gather = true;
    }

    named_content.push(named_container(Container::named_with_flags(
        choice_container_name,
        choice_content,
        named_container_flags(
            count_all_visits
                || choice.once_only()
                || counted_paths.visits.contains(&choice_container_path),
            counted_paths.turns.contains(&choice_container_path),
            false,
        ),
    )));
    if let Some(identifier) = choice.identifier() {
        choice_labels.insert(
            identifier.to_string(),
            path_mode.absolute_child_path(&format!("c-{choice_index}")),
        );
    }
}

fn collect_local_weave_labels(objects: &[Object], path_mode: &ChoicePathMode) -> LabelIndex {
    let mut labels = LabelIndex::new();
    let mut choice_count = 0;
    let mut gather_count = 0;
    let base_container_path = path_mode.container_path();
    let mut current_container_path = base_container_path.clone();
    let mut last_section_had_choice = false;
    for object in objects {
        match object {
            Object::Choice(choice) => {
                if let Some(identifier) = choice.identifier() {
                    labels.insert(
                        identifier.to_string(),
                        child_path(&current_container_path, &format!("c-{choice_count}")),
                    );
                }
                choice_count += 1;
                last_section_had_choice = true;
            }
            Object::Gather(gather) => {
                let gather_name = gather.identifier().map(str::to_string).unwrap_or_else(|| {
                    let name = format!("g-{gather_count}");
                    gather_count += 1;
                    name
                });
                let gather_path = if last_section_had_choice {
                    child_path(&base_container_path, &gather_name)
                } else {
                    child_path(&current_container_path, &gather_name)
                };
                if let Some(identifier) = gather.identifier() {
                    labels.insert(identifier.to_string(), gather_path.clone());
                }
                current_container_path = gather_path;
                last_section_had_choice = false;
            }
            _ => {}
        }
    }
    labels
}

fn next_gather_name(objects: &[Object], start_index: usize, unnamed_gather_count: usize) -> String {
    objects[start_index..]
        .iter()
        .find_map(|object| match object {
            Object::Gather(gather) => Some(
                gather
                    .identifier()
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("g-{unnamed_gather_count}")),
            ),
            _ => None,
        })
        .unwrap_or_else(|| format!("g-{unnamed_gather_count}"))
}

fn choice_outer(
    choice: &Choice,
    choice_container_path: &str,
    choice_point_index: usize,
    path_mode: &ChoicePathMode,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
) -> ChoiceOuter {
    let mut outer_content = Vec::new();
    let has_eval_content = choice.has_start_content()
        || choice.has_choice_only_content()
        || choice.condition().is_some();

    if has_eval_content {
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    }

    if choice.has_start_content() {
        outer_content.push(RuntimeObject::DivertTarget(format!(
            "{}.$r1",
            path_mode.outer_return_target(choice_point_index)
        )));
        outer_content.push(RuntimeObject::VariableAssignment("$r".to_string()));
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::BeginString));
        outer_content.push(RuntimeObject::Divert {
            target: ".^.s".to_string(),
            variable: false,
        });
        outer_content.push(RuntimeObject::named_container("$r1", Vec::new()));
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::EndString));
    }

    if let Some(choice_only_content) = choice.choice_only_content() {
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::BeginString));
        lower_content_list_into_context(
            &mut outer_content,
            choice_only_content,
            path_mode,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            struct_definitions,
        );
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::EndString));
    }

    if let Some(condition) = choice.condition() {
        lower_expression_into(
            &mut outer_content,
            condition,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            struct_definitions,
            path_mode,
            choice.has_start_content(),
        );
    }

    if has_eval_content {
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    }

    outer_content.push(RuntimeObject::ChoicePoint {
        target: choice_container_path.to_string(),
        flags: choice.choice_flags(),
    });

    if !choice.has_start_content() {
        return ChoiceOuter::Inline(outer_content);
    }

    let mut start_content = choice
        .start_content()
        .map(|cl| {
            lower_content_list_with_context(
                cl,
                path_mode,
                choice_labels,
                global_labels,
                global_variables,
                external_signatures,
                constants,
                struct_definitions,
            )
        })
        .unwrap_or_default();
    start_content.push(RuntimeObject::Divert {
        target: "$r".to_string(),
        variable: true,
    });

    let mut outer_container = Container::unnamed(outer_content);
    outer_container
        .named_content
        .push(named_content("s", start_content));

    ChoiceOuter::Nested(outer_container)
}

pub(super) fn choice_container_prefix(
    path_mode: &ChoicePathMode,
    choice_container_name: &str,
    choice_point_index: usize,
    return_index: usize,
) -> Vec<RuntimeObject> {
    vec![
        RuntimeObject::ControlCommand(ControlCommand::EvalStart),
        RuntimeObject::DivertTarget(format!(
            "{}.$r{return_index}",
            path_mode.choice_content_return_target(choice_container_name)
        )),
        RuntimeObject::ControlCommand(ControlCommand::EvalEnd),
        RuntimeObject::VariableAssignment("$r".to_string()),
        RuntimeObject::Divert {
            target: path_mode.start_content_target(choice_point_index),
            variable: false,
        },
        RuntimeObject::named_container(format!("$r{return_index}"), Vec::new()),
    ]
}

fn lower_content_list_with_context(
    content_list: &ContentList,
    path_mode: &ChoicePathMode,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
) -> Vec<RuntimeObject> {
    let mut content = Vec::new();
    lower_content_list_into_context(
        &mut content,
        content_list,
        path_mode,
        choice_labels,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        struct_definitions,
    );
    content
}

pub(super) fn lower_content_list_into_context(
    content: &mut Vec<RuntimeObject>,
    content_list: &ContentList,
    path_mode: &ChoicePathMode,
    choice_labels: &LabelIndex,
    global_labels: &LabelIndex,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
) {
    for object in content_list.objects() {
        lower_object_into_with_context(
            content,
            object,
            path_mode,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            struct_definitions,
        );
    }
}
