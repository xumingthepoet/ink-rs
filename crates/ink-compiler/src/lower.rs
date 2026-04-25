use std::collections::{HashMap, HashSet};

mod context;
mod indexes;
pub(crate) mod ir;
mod path;

use crate::{
    analysis::CheckedStory,
    compiler::StageOutput,
    parsed::{
        BinaryOperator, Choice, Conditional, ContentList, Divert, DivertTarget, Expression, Flow,
        FlowArgument, Object, Sequence, SequenceType, Weave,
    },
};

use context::ChoicePathMode;
use indexes::{
    collect_counted_paths_in_weave, CallSignature, CountedFlowPaths, ExternalSignatures,
    LoweringIndexes, RuntimeLenEstimator,
};
use ir::{Container, ControlCommand, RuntimeObject, RuntimeProgram};
use path::{
    canonical_runtime_path, child_path, compact_path_string, compact_relative_path,
    is_absolute_runtime_path, is_user_named_path_component, semantic_path_key,
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

fn lower_root_weave(
    weave: &Weave,
    indexes: &LoweringIndexes<'_>,
    count_all_visits: bool,
) -> Vec<RuntimeObject> {
    if weave_has_weave_points(weave) {
        lower_choice_weave(
            weave,
            ChoicePathMode::Root,
            &indexes.global_labels,
            &indexes.global_variables,
            &indexes.external_signatures,
            &indexes.constants,
            count_all_visits,
        )
    } else {
        let mut content = lower_linear_weave(
            weave,
            &indexes.global_labels,
            &indexes.global_variables,
            &indexes.external_signatures,
            &indexes.constants,
        );
        content.push(RuntimeObject::Container(done_container(
            "g-0",
            count_all_visits,
        )));
        content
    }
}

fn lower_flow(flow: &Flow, indexes: &LoweringIndexes<'_>, count_all_visits: bool) -> Container {
    // Collect child stitch names upfront so knot-level choices can reference them
    let child_stitch_names: Vec<String> = flow
        .child_flows()
        .iter()
        .map(|f| f.name().to_string())
        .collect();
    lower_flow_with_context(
        flow,
        None,
        &child_stitch_names,
        &indexes.global_labels,
        &indexes.global_variables,
        &indexes.external_signatures,
        &indexes.constants,
        &indexes.counted_flow_paths,
        count_all_visits,
    )
}

fn lower_flow_with_context(
    flow: &Flow,
    parent_knot_name: Option<&str>,
    sibling_stitch_names: &[String],
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    counted_flow_paths: &CountedFlowPaths,
    count_all_visits: bool,
) -> Container {
    let mut content = Vec::new();
    let flow_path = parent_knot_name
        .map(|parent| format!("{parent}.{}", flow.name()))
        .unwrap_or_else(|| flow.name().to_string());
    let local_variables = collect_flow_local_variables(flow);

    lower_flow_arguments_into(&mut content, flow);

    // Lower any content in the flow's own weave
    if weave_has_weave_points(flow.weave()) {
        // For stitches inside a knot, pass the parent knot name and sibling stitch names
        let flow_container_path = parent_knot_name
            .map(|parent| format!("{parent}.{}.{}", flow.name(), content.len()))
            .unwrap_or_else(|| format!("{}.{}", flow.name(), content.len()));
        let path_mode = ChoicePathMode::Flow {
            flow_name: flow.name().to_string(),
            container_path: flow_container_path,
            parent_flow_name: parent_knot_name.map(|s| s.to_string()),
            sibling_stitch_names: sibling_stitch_names.to_vec(),
            local_variables: local_variables.clone(),
            self_target_relative: false,
            fallback_gather_target: None,
        };
        content.push(RuntimeObject::Container(Container {
            content: lower_choice_weave(
                flow.weave(),
                path_mode,
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
    } else if !flow.weave().content().is_empty() {
        let path_mode = ChoicePathMode::Flow {
            flow_name: flow.name().to_string(),
            container_path: flow_path.clone(),
            parent_flow_name: parent_knot_name.map(str::to_string),
            sibling_stitch_names: sibling_stitch_names.to_vec(),
            local_variables,
            self_target_relative: false,
            fallback_gather_target: None,
        };
        lower_linear_weave_into_context(
            &mut content,
            flow.weave(),
            global_labels,
            global_variables,
            external_signatures,
            constants,
            &path_mode,
        );
    }

    // Lower child flows (stitches)
    if !flow.child_flows().is_empty() {
        // Only add auto-divert to first child flow if the knot's weave doesn't have choices
        // When choices are present, they explicitly divert to stitches
        let weave_has_choices = weave_has_choice(flow.weave());
        if !weave_has_choices && !ends_with_flow_terminator(&content) {
            let first_child_name = flow.child_flows()[0].name();
            content.push(RuntimeObject::Divert {
                target: format!(".^.{}", first_child_name),
                variable: false,
            });
        }

        // Collect all child stitch names for sibling reference
        let child_stitch_names: Vec<String> = flow
            .child_flows()
            .iter()
            .map(|f| f.name().to_string())
            .collect();

        // Lower each child flow as named content, passing sibling stitch names
        let child_containers: Vec<Container> = flow
            .child_flows()
            .iter()
            .map(|child| {
                lower_flow_with_context(
                    child,
                    Some(flow.name()),
                    &child_stitch_names,
                    global_labels,
                    global_variables,
                    external_signatures,
                    constants,
                    counted_flow_paths,
                    count_all_visits,
                )
            })
            .collect();
        content.push(RuntimeObject::NamedContent(child_containers));
    }

    Container {
        content,
        name: Some(flow.name().to_string()),
        flags: flow_container_flags(
            counted_flow_paths.turns.contains(&flow_path),
            counted_flow_paths.visits.contains(&flow_path),
            count_all_visits,
        ),
        merge_tail_metadata: true,
    }
}

fn flow_container_flags(
    count_turns: bool,
    count_visits: bool,
    count_all_visits: bool,
) -> Option<i32> {
    match (count_turns, count_visits || count_all_visits) {
        (true, _) => Some(3),
        (false, true) => Some(1),
        (false, false) => None,
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

fn lower_flow_arguments_into(content: &mut Vec<RuntimeObject>, flow: &Flow) {
    for argument in flow.arguments().iter().rev() {
        content.push(RuntimeObject::VariableAssignment(
            argument.name().to_string(),
        ));
    }
}

fn collect_flow_local_variables(flow: &Flow) -> HashSet<String> {
    let mut local_variables = flow
        .arguments()
        .iter()
        .map(|argument| argument.name().to_string())
        .collect::<HashSet<_>>();
    collect_local_variables_in_weave(flow.weave(), &mut local_variables);
    local_variables
}

fn collect_local_variables_in_weave(weave: &Weave, local_variables: &mut HashSet<String>) {
    for object in weave.content() {
        collect_local_variables_in_object(object, local_variables);
    }
}

fn collect_local_variables_in_content_list(
    content_list: &ContentList,
    local_variables: &mut HashSet<String>,
) {
    for object in content_list.objects() {
        collect_local_variables_in_object(object, local_variables);
    }
}

fn collect_local_variables_in_object(object: &Object, local_variables: &mut HashSet<String>) {
    match object {
        Object::VariableAssignment(assignment) if assignment.is_temporary() => {
            local_variables.insert(assignment.name().to_string());
        }
        Object::ContentList(content_list) => {
            collect_local_variables_in_content_list(content_list, local_variables);
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_local_variables_in_weave(branch.content(), local_variables);
            }
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_local_variables_in_content_list(content, local_variables);
            }
            if let Some(content) = choice.choice_only_content() {
                collect_local_variables_in_content_list(content, local_variables);
            }
            collect_local_variables_in_content_list(choice.inner_content(), local_variables);
        }
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                collect_local_variables_in_content_list(element, local_variables);
            }
        }
        Object::Weave(weave) => collect_local_variables_in_weave(weave, local_variables),
        _ => {}
    }
}

fn weave_has_choice(weave: &Weave) -> bool {
    weave
        .content()
        .iter()
        .any(|object| matches!(object, Object::Choice(_)))
}

fn weave_has_weave_points(weave: &Weave) -> bool {
    weave
        .content()
        .iter()
        .any(|object| matches!(object, Object::Choice(_) | Object::Gather(_)))
}

fn content_list_has_choice(content_list: &ContentList) -> bool {
    content_list
        .objects()
        .iter()
        .any(|object| matches!(object, Object::Choice(_)))
}

fn lower_linear_weave(
    weave: &Weave,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
) -> Vec<RuntimeObject> {
    lower_linear_weave_with_context(
        weave,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        &ChoicePathMode::Root,
    )
}

fn lower_linear_weave_with_context(
    weave: &Weave,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
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
        path_mode,
    );
    content
}

fn lower_linear_weave_into_context(
    content: &mut Vec<RuntimeObject>,
    weave: &Weave,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
) {
    let choice_labels = HashMap::new();
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
        );
    }
}

fn lower_choice_weave(
    weave: &Weave,
    path_mode: ChoicePathMode,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    count_all_visits: bool,
) -> Vec<RuntimeObject> {
    lower_choice_weave_with_initial_content(
        weave,
        path_mode,
        global_labels,
        global_variables,
        external_signatures,
        constants,
        count_all_visits,
        Vec::new(),
    )
}

fn lower_choice_weave_with_initial_content(
    weave: &Weave,
    path_mode: ChoicePathMode,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    count_all_visits: bool,
    initial_content: Vec<RuntimeObject>,
) -> Vec<RuntimeObject> {
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
        &path_mode,
        &mut counted_paths,
    );

    // Check if there's an explicit gather anywhere in the weave
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
                    gather_count,
                    &current_path_mode,
                    &path_mode,
                    has_explicit_gather,
                    count_all_visits,
                    &counted_paths,
                );
            }
            Object::Gather(_gather) => {
                // Create a named container for the gather
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
                    gather_count,
                    &gather_path_mode,
                    &path_mode,
                    has_explicit_gather,
                    count_all_visits,
                    &counted_paths,
                );
                if !gather_named_content.is_empty() {
                    gather_content.push(RuntimeObject::NamedContent(gather_named_content));
                }
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
                    merge_tail_metadata: true,
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
                    named_content.push(gather_container);
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
                push_before_trailing_named_content(
                    &mut container.content,
                    RuntimeObject::Container(done_container(
                        &format!("g-{gather_count}"),
                        count_all_visits,
                    )),
                );
            }
        }
    } else if !path_mode.is_nested_root()
        && !has_explicit_gather
        && needs_terminal_gather
        && path_mode.fallback_gather_target().is_none()
    {
        // Add a terminal gather for choices that divert to it.
        named_content.push(done_container(
            &format!("g-{gather_count}"),
            count_all_visits,
        ));
    }
    if !named_content.is_empty() {
        main_content.push(RuntimeObject::NamedContent(named_content));
    }

    main_content
}

fn push_before_trailing_named_content(content: &mut Vec<RuntimeObject>, object: RuntimeObject) {
    if matches!(content.last(), Some(RuntimeObject::NamedContent(_))) {
        content.insert(content.len() - 1, object);
    } else {
        content.push(object);
    }
}

fn lower_weave_section(
    objects: &[Object],
    index: &mut usize,
    content: &mut Vec<RuntimeObject>,
    named_content: &mut Vec<Container>,
    choice_count: &mut usize,
    needs_terminal_gather: &mut bool,
    choice_labels: &mut HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
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
    named_content: &'a mut [Container],
    location: &GatherLocation,
) -> Option<&'a mut Container> {
    match location {
        GatherLocation::Main(path) => nested_container_in_runtime_content_mut(main_content, path),
        GatherLocation::Named(path) => nested_container_in_named_content_mut(named_content, path),
    }
}

fn nested_container_in_named_content_mut<'a>(
    named_content: &'a mut [Container],
    path: &[usize],
) -> Option<&'a mut Container> {
    let (&first, rest) = path.split_first()?;
    let container = named_content.get_mut(first)?;
    nested_container_in_container_mut(container, rest)
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
    named_content: &mut Vec<Container>,
    choice_count: &mut usize,
    needs_terminal_gather: &mut bool,
    choice_labels: &mut HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
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

    named_content.push(Container {
        content: choice_content,
        name: Some(choice_container_name),
        flags: named_container_flags(
            count_all_visits
                || choice.once_only()
                || counted_paths.visits.contains(&choice_container_path),
            counted_paths.turns.contains(&choice_container_path),
            false,
        ),
        merge_tail_metadata: true,
    });
    if let Some(identifier) = choice.identifier() {
        choice_labels.insert(
            identifier.to_string(),
            path_mode.absolute_child_path(&format!("c-{choice_index}")),
        );
    }
}

fn collect_local_weave_labels(
    objects: &[Object],
    path_mode: &ChoicePathMode,
) -> HashMap<String, String> {
    let mut labels = HashMap::new();
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
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
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
        outer_content.push(RuntimeObject::Container(Container {
            content: Vec::new(),
            name: Some("$r1".to_string()),
            flags: None,
            merge_tail_metadata: true,
        }));
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
        );
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::EndString));
    }

    if let Some(condition) = choice.condition() {
        lower_expression_into(
            &mut outer_content,
            condition,
            choice_labels,
            global_labels,
            external_signatures,
            constants,
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
            )
        })
        .unwrap_or_default();
    start_content.push(RuntimeObject::Divert {
        target: "$r".to_string(),
        variable: true,
    });
    outer_content.push(RuntimeObject::NamedContent(vec![Container {
        content: start_content,
        name: Some("s".to_string()),
        flags: None,
        merge_tail_metadata: true,
    }]));

    ChoiceOuter::Nested(Container {
        content: outer_content,
        name: None,
        flags: None,
        merge_tail_metadata: true,
    })
}

fn choice_container_prefix(
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
        RuntimeObject::Container(Container {
            content: Vec::new(),
            name: Some(format!("$r{return_index}")),
            flags: None,
            merge_tail_metadata: true,
        }),
    ]
}

fn lower_content_list_with_context(
    content_list: &ContentList,
    path_mode: &ChoicePathMode,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
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
    );
    content
}

fn lower_content_list_into_context(
    content: &mut Vec<RuntimeObject>,
    content_list: &ContentList,
    path_mode: &ChoicePathMode,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
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
        );
    }
}

fn lower_expression_into(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
    has_start_content: bool,
) {
    let mut visiting_constants = HashSet::new();
    lower_expression_into_with_constants(
        content,
        expression,
        choice_labels,
        global_labels,
        external_signatures,
        constants,
        path_mode,
        has_start_content,
        &mut visiting_constants,
    );
}

fn lower_expression_into_with_constants(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
    has_start_content: bool,
    visiting_constants: &mut HashSet<String>,
) {
    match expression {
        Expression::String(value) => {
            content.push(RuntimeObject::ControlCommand(ControlCommand::BeginString));
            content.push(RuntimeObject::String(value.clone()));
            content.push(RuntimeObject::ControlCommand(ControlCommand::EndString));
        }
        Expression::StringContent(string_content) => {
            content.push(RuntimeObject::ControlCommand(ControlCommand::BeginString));
            lower_content_list_into_context(
                content,
                string_content,
                path_mode,
                choice_labels,
                global_labels,
                &HashSet::new(),
                external_signatures,
                constants,
            );
            content.push(RuntimeObject::ControlCommand(ControlCommand::EndString));
        }
        Expression::NumberInt(value) => content.push(RuntimeObject::Int(*value)),
        Expression::NumberFloat(value) => content.push(RuntimeObject::Float(*value)),
        Expression::NumberBool(value) => content.push(RuntimeObject::Bool(*value)),
        Expression::DivertTarget(target) => {
            let resolved_target = if let Some(choice_target) = choice_labels.get(target) {
                choice_target.clone()
            } else if let Some(label_target) = path_mode
                .scoped_label_target(target, global_labels)
                .filter(|label_target| label_target.as_str() != target)
            {
                path_mode.resolve_label_target(label_target)
            } else {
                path_mode.resolve_divert_target(target)
            };
            content.push(RuntimeObject::DivertTarget(resolved_target));
        }
        Expression::VariableReference(name) => {
            if let Some(constant) = constants.get(name) {
                if visiting_constants.insert(name.clone()) {
                    lower_expression_into_with_constants(
                        content,
                        constant,
                        choice_labels,
                        global_labels,
                        external_signatures,
                        constants,
                        path_mode,
                        has_start_content,
                        visiting_constants,
                    );
                    visiting_constants.remove(name);
                    return;
                }
            }

            if let Some(choice_target) = choice_labels.get(name) {
                let _ = has_start_content;
                content.push(RuntimeObject::ReadCount(choice_target.clone()));
            } else if let Some(label_target) = path_mode.scoped_label_target(name, global_labels) {
                content.push(RuntimeObject::ReadCount(
                    path_mode.resolve_label_target(label_target),
                ));
            } else if path_mode.is_flow_sibling_stitch(name) {
                content.push(RuntimeObject::ReadCount(
                    path_mode.resolve_single_stitch_target(name),
                ));
            } else {
                content.push(RuntimeObject::VariableReference(name.clone()));
            }
        }
        Expression::FunctionCall { name, args } => {
            lower_function_call_into(
                content,
                name,
                args,
                choice_labels,
                global_labels,
                external_signatures,
                constants,
                path_mode,
                has_start_content,
                visiting_constants,
            );
        }
        Expression::Binary {
            operator,
            left,
            right,
        } => {
            lower_expression_into_with_constants(
                content,
                left,
                choice_labels,
                global_labels,
                external_signatures,
                constants,
                path_mode,
                has_start_content,
                visiting_constants,
            );
            lower_expression_into_with_constants(
                content,
                right,
                choice_labels,
                global_labels,
                external_signatures,
                constants,
                path_mode,
                has_start_content,
                visiting_constants,
            );
            content.push(RuntimeObject::NativeFunction(
                operator_runtime_name(*operator).to_string(),
            ));
        }
        Expression::Unary {
            operator,
            expression,
        } => {
            lower_expression_into_with_constants(
                content,
                expression,
                choice_labels,
                global_labels,
                external_signatures,
                constants,
                path_mode,
                has_start_content,
                visiting_constants,
            );
            content.push(RuntimeObject::NativeFunction(
                operator.runtime_name().to_string(),
            ));
        }
        Expression::MultipleCondition(expressions) => {
            for (index, expression) in expressions.iter().enumerate() {
                lower_expression_into_with_constants(
                    content,
                    expression,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
                if index > 0 {
                    content.push(RuntimeObject::NativeFunction("&&".to_string()));
                }
            }
        }
    }
}

fn lower_function_call_into(
    content: &mut Vec<RuntimeObject>,
    name: &str,
    args: &[Expression],
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
    has_start_content: bool,
    visiting_constants: &mut HashSet<String>,
) {
    match name {
        "CHOICE_COUNT" => content.push(RuntimeObject::ControlCommand(ControlCommand::ChoiceCount)),
        "TURNS" => content.push(RuntimeObject::ControlCommand(ControlCommand::Turns)),
        "TURNS_SINCE" => {
            if let Some(arg) = args.first() {
                lower_function_arg_into(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::TurnsSince));
        }
        "READ_COUNT" => {
            if let Some(arg) = args.first() {
                lower_function_arg_into(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::ReadCount));
        }
        "RANDOM" => {
            for arg in args {
                lower_function_arg_into(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::Random));
        }
        "SEED_RANDOM" => {
            for arg in args {
                lower_function_arg_into(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::SeedRandom));
        }
        "LIST_RANGE" => {
            for arg in args {
                lower_function_arg_into(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::ListRange));
        }
        "LIST_RANDOM" => {
            for arg in args {
                lower_function_arg_into(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::ListRandom));
        }
        _ if is_builtin_function(name) => {
            for arg in args {
                lower_function_arg_into(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::NativeFunction(name.to_string()));
        }
        _ if matches!(
            external_signatures.get(name),
            Some(CallSignature::External { .. })
        ) =>
        {
            for arg in args {
                lower_function_arg_into(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::ExternalFunction {
                target: name.to_string(),
                args: args.len(),
            });
        }
        _ if matches!(
            external_signatures.get(name),
            Some(CallSignature::Ink { .. })
        ) =>
        {
            let expected_args = match external_signatures.get(name) {
                Some(CallSignature::Ink { args }) => args.as_slice(),
                _ => &[],
            };
            for (index, arg) in args.iter().enumerate() {
                lower_function_arg_into(
                    content,
                    arg,
                    expected_args.get(index),
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::FunctionDivert {
                target: name.to_string(),
            });
        }
        _ => {
            for arg in args {
                lower_function_arg_into(
                    content,
                    arg,
                    None,
                    choice_labels,
                    global_labels,
                    external_signatures,
                    constants,
                    path_mode,
                    has_start_content,
                    visiting_constants,
                );
            }
            content.push(RuntimeObject::FunctionDivert {
                target: name.to_string(),
            });
        }
    }
}

fn lower_function_arg_into(
    content: &mut Vec<RuntimeObject>,
    arg: &Expression,
    expected_arg: Option<&FlowArgument>,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
    has_start_content: bool,
    visiting_constants: &mut HashSet<String>,
) {
    if expected_arg.is_some_and(FlowArgument::is_by_reference) {
        if let Expression::VariableReference(name) = arg {
            content.push(RuntimeObject::VariablePointer {
                name: name.clone(),
                context_index: -1,
            });
            return;
        }
    }

    lower_expression_into_with_constants(
        content,
        arg,
        choice_labels,
        global_labels,
        external_signatures,
        constants,
        path_mode,
        has_start_content,
        visiting_constants,
    );
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

fn lower_output_expression_into(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
) {
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    lower_expression_into(
        content,
        expression,
        choice_labels,
        global_labels,
        external_signatures,
        constants,
        path_mode,
        false,
    );
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalOutput));
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
}

fn lower_logic_line_into(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    external_signatures: &ExternalSignatures,
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
) {
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    lower_expression_into(
        content,
        expression,
        choice_labels,
        global_labels,
        external_signatures,
        constants,
        path_mode,
        false,
    );
    content.push(RuntimeObject::ControlCommand(ControlCommand::Pop));
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    content.push(RuntimeObject::String("\n".to_string()));
}

fn operator_runtime_name(operator: BinaryOperator) -> &'static str {
    operator.runtime_name()
}

fn is_builtin_function(name: &str) -> bool {
    matches!(
        name,
        "LIST_VALUE"
            | "MIN"
            | "MAX"
            | "POW"
            | "FLOOR"
            | "CEILING"
            | "INT"
            | "FLOAT"
            | "LIST_MIN"
            | "LIST_MAX"
            | "LIST_ALL"
            | "LIST_COUNT"
            | "LIST_INVERT"
    )
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
