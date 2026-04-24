use std::collections::HashMap;

use crate::{
    analysis::CheckedStory,
    compiler::StageOutput,
    parsed::{
        BinaryOperator, Choice, ContentList, DivertTarget, Expression, Flow, Object, Sequence,
        SequenceType, Story, Weave,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeProgram {
    pub root: Container,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub content: Vec<RuntimeObject>,
    pub name: Option<String>,
    pub flags: Option<i32>,
    pub merge_tail_metadata: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeObject {
    Container(Container),
    NamedContent(Vec<Container>),
    String(String),
    ControlCommand(ControlCommand),
    Divert { target: String, variable: bool },
    ConditionalDivert { target: String },
    DivertTarget(String),
    ReadCount(String),
    VariableAssignment(String),
    VariableReference(String),
    ChoicePoint { target: String, flags: i32 },
    Glue,
    Tag { is_start: bool },
    Bool(bool),
    Int(i32),
    NativeFunction(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlCommand {
    Done,
    End,
    EvalStart,
    EvalEnd,
    BeginString,
    EndString,
    VisitIndex,
    Duplicate,
    NoOp,
    Pop,
}

enum ChoiceOuter {
    Inline(Vec<RuntimeObject>),
    Nested(Container),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ChoicePathMode {
    Root,
    RootGather {
        gather_name: String,
    },
    NestedRoot {
        container_path: String,
        gather_target: String,
    },
    Flow {
        flow_name: String,
        container_path: String,
        parent_flow_name: Option<String>,
        sibling_stitch_names: Vec<String>,
        self_target_relative: bool,
    },
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

impl ChoicePathMode {
    fn with_self_target_relative(&self, self_target_relative: bool) -> Self {
        match self {
            ChoicePathMode::Root => ChoicePathMode::Root,
            ChoicePathMode::RootGather { gather_name } => ChoicePathMode::RootGather {
                gather_name: gather_name.clone(),
            },
            ChoicePathMode::NestedRoot {
                container_path,
                gather_target,
            } => ChoicePathMode::NestedRoot {
                container_path: container_path.clone(),
                gather_target: gather_target.clone(),
            },
            ChoicePathMode::Flow {
                flow_name,
                container_path,
                parent_flow_name,
                sibling_stitch_names,
                ..
            } => ChoicePathMode::Flow {
                flow_name: flow_name.clone(),
                container_path: container_path.clone(),
                parent_flow_name: parent_flow_name.clone(),
                sibling_stitch_names: sibling_stitch_names.clone(),
                self_target_relative,
            },
        }
    }

    fn for_choice_nested_content(
        &self,
        choice_container_name: &str,
        gather_container_name: &str,
        has_following_gather: bool,
    ) -> Self {
        match self {
            ChoicePathMode::Root => ChoicePathMode::NestedRoot {
                container_path: format!("0.{choice_container_name}"),
                gather_target: format!("0.{gather_container_name}"),
            },
            ChoicePathMode::RootGather { gather_name } => ChoicePathMode::NestedRoot {
                container_path: format!("0.{gather_name}.{choice_container_name}"),
                gather_target: format!("0.{gather_container_name}"),
            },
            ChoicePathMode::NestedRoot {
                container_path,
                gather_target,
            } => ChoicePathMode::NestedRoot {
                container_path: format!("{container_path}.{choice_container_name}"),
                gather_target: if has_following_gather {
                    format!("{container_path}.{gather_container_name}")
                } else {
                    gather_target.clone()
                },
            },
            ChoicePathMode::Flow { .. } => self.with_self_target_relative(true),
        }
    }

    fn for_nested_weave(&self, container_index: usize) -> Self {
        match self {
            ChoicePathMode::NestedRoot {
                container_path,
                gather_target,
            } => ChoicePathMode::NestedRoot {
                container_path: format!("{container_path}.{container_index}"),
                gather_target: gather_target.clone(),
            },
            ChoicePathMode::Root => ChoicePathMode::NestedRoot {
                container_path: format!("0.{container_index}"),
                gather_target: "0.g-0".to_string(),
            },
            ChoicePathMode::RootGather { gather_name } => ChoicePathMode::NestedRoot {
                container_path: format!("0.{gather_name}.{container_index}"),
                gather_target: "0.g-0".to_string(),
            },
            ChoicePathMode::Flow {
                flow_name,
                container_path,
                parent_flow_name,
                sibling_stitch_names,
                self_target_relative,
            } => ChoicePathMode::Flow {
                flow_name: flow_name.clone(),
                container_path: format!("{container_path}.{container_index}"),
                parent_flow_name: parent_flow_name.clone(),
                sibling_stitch_names: sibling_stitch_names.clone(),
                self_target_relative: *self_target_relative,
            },
        }
    }

    fn for_gather(&self, gather_name: &str) -> Self {
        match self {
            ChoicePathMode::Root => ChoicePathMode::RootGather {
                gather_name: gather_name.to_string(),
            },
            ChoicePathMode::NestedRoot {
                container_path,
                gather_target,
            } => ChoicePathMode::NestedRoot {
                container_path: format!("{container_path}.{gather_name}"),
                gather_target: gather_target.clone(),
            },
            ChoicePathMode::Flow {
                flow_name,
                container_path,
                parent_flow_name,
                sibling_stitch_names,
                self_target_relative,
            } => ChoicePathMode::Flow {
                flow_name: flow_name.clone(),
                container_path: format!("{container_path}.{gather_name}"),
                parent_flow_name: parent_flow_name.clone(),
                sibling_stitch_names: sibling_stitch_names.clone(),
                self_target_relative: *self_target_relative,
            },
            other => other.clone(),
        }
    }

    fn fallback_gather_target(&self) -> Option<String> {
        match self {
            ChoicePathMode::NestedRoot { gather_target, .. } => Some(gather_target.clone()),
            _ => None,
        }
    }
}

pub(crate) fn lower(story: &CheckedStory) -> StageOutput<RuntimeProgram> {
    let global_labels = build_label_index(&story.parsed);
    let root_weave = story.parsed.root_weave();
    let main_content = lower_root_weave(root_weave, &global_labels);

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

    let flow_containers = story
        .parsed
        .flows()
        .iter()
        .map(|flow| lower_flow(flow, &global_labels))
        .collect::<Vec<_>>();
    if !flow_containers.is_empty() {
        root_content.push(RuntimeObject::NamedContent(flow_containers));
    }

    let root = Container {
        content: root_content,
        name: None,
        flags: None,
        merge_tail_metadata: true,
    };

    StageOutput {
        artifact: Some(RuntimeProgram { root }),
        diagnostics: Vec::new(),
    }
}

fn build_label_index(story: &Story) -> HashMap<String, String> {
    let mut labels = HashMap::new();
    collect_weave_labels(story.root_weave(), "0", &mut labels);
    for flow in story.flows() {
        collect_flow_labels(flow, None, &mut labels);
    }
    labels
}

fn collect_flow_labels(
    flow: &Flow,
    parent_flow_name: Option<&str>,
    labels: &mut HashMap<String, String>,
) {
    let flow_path = parent_flow_name
        .map(|parent| format!("{parent}.{}", flow.name()))
        .unwrap_or_else(|| flow.name().to_string());
    collect_weave_labels(flow.weave(), &format!("{flow_path}.0"), labels);
    for child in flow.child_flows() {
        collect_flow_labels(child, Some(&flow_path), labels);
    }
}

fn collect_weave_labels(weave: &Weave, container_path: &str, labels: &mut HashMap<String, String>) {
    let mut choice_count = 0;
    let mut gather_count = 0;
    for object in weave.content() {
        match object {
            Object::Choice(choice) => {
                if let Some(identifier) = choice.identifier() {
                    insert_label_aliases(
                        labels,
                        identifier,
                        container_path,
                        &format!("{container_path}.c-{choice_count}"),
                    );
                }
                choice_count += 1;
            }
            Object::Gather(gather) => {
                let gather_name = gather.identifier().map(str::to_string).unwrap_or_else(|| {
                    let name = format!("g-{gather_count}");
                    gather_count += 1;
                    name
                });
                if let Some(identifier) = gather.identifier() {
                    insert_label_aliases(
                        labels,
                        identifier,
                        container_path,
                        &format!("{container_path}.{gather_name}"),
                    );
                }
            }
            Object::Weave(weave) => {
                collect_weave_labels(weave, container_path, labels);
            }
            _ => {}
        }
    }
}

fn insert_label_aliases(
    labels: &mut HashMap<String, String>,
    identifier: &str,
    container_path: &str,
    target_path: &str,
) {
    labels.insert(identifier.to_string(), target_path.to_string());
    if let Some(flow_path) = container_path.strip_suffix(".0") {
        labels.insert(format!("{flow_path}.{identifier}"), target_path.to_string());
    }
}

fn lower_root_weave(weave: &Weave, global_labels: &HashMap<String, String>) -> Vec<RuntimeObject> {
    if weave_has_choice(weave) {
        lower_choice_weave(weave, ChoicePathMode::Root, global_labels)
    } else {
        let mut content = lower_linear_weave(weave, global_labels);
        content.push(RuntimeObject::Container(done_container("g-0")));
        content
    }
}

fn lower_flow(flow: &Flow, global_labels: &HashMap<String, String>) -> Container {
    // Collect child stitch names upfront so knot-level choices can reference them
    let child_stitch_names: Vec<String> = flow
        .child_flows()
        .iter()
        .map(|f| f.name().to_string())
        .collect();
    lower_flow_with_context(flow, None, &child_stitch_names, global_labels)
}

fn lower_flow_with_context(
    flow: &Flow,
    parent_knot_name: Option<&str>,
    sibling_stitch_names: &[String],
    global_labels: &HashMap<String, String>,
) -> Container {
    let mut content = Vec::new();

    // Lower any content in the flow's own weave
    if weave_has_choice(flow.weave()) {
        // For stitches inside a knot, pass the parent knot name and sibling stitch names
        let flow_container_path = parent_knot_name
            .map(|parent| format!("{parent}.{}.0", flow.name()))
            .unwrap_or_else(|| format!("{}.0", flow.name()));
        let path_mode = ChoicePathMode::Flow {
            flow_name: flow.name().to_string(),
            container_path: flow_container_path,
            parent_flow_name: parent_knot_name.map(|s| s.to_string()),
            sibling_stitch_names: sibling_stitch_names.to_vec(),
            self_target_relative: false,
        };
        content.push(RuntimeObject::Container(Container {
            content: lower_choice_weave(flow.weave(), path_mode, global_labels),
            name: None,
            flags: None,
            merge_tail_metadata: true,
        }));
    } else if !flow.weave().content().is_empty() {
        content.extend(lower_linear_weave(flow.weave(), global_labels));
    }

    // Lower child flows (stitches)
    if !flow.child_flows().is_empty() {
        // Only add auto-divert to first child flow if the knot's weave doesn't have choices
        // When choices are present, they explicitly divert to stitches
        let weave_has_choices = weave_has_choice(flow.weave());
        if !weave_has_choices {
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
                )
            })
            .collect();
        content.push(RuntimeObject::NamedContent(child_containers));
    }

    Container {
        content,
        name: Some(flow.name().to_string()),
        flags: None,
        merge_tail_metadata: true,
    }
}

fn weave_has_choice(weave: &Weave) -> bool {
    weave
        .content()
        .iter()
        .any(|object| matches!(object, Object::Choice(_)))
}

fn lower_linear_weave(
    weave: &Weave,
    global_labels: &HashMap<String, String>,
) -> Vec<RuntimeObject> {
    let mut content = Vec::new();
    for object in weave.content() {
        lower_object_into(&mut content, object, global_labels);
    }
    content
}

fn lower_choice_weave(
    weave: &Weave,
    path_mode: ChoicePathMode,
    global_labels: &HashMap<String, String>,
) -> Vec<RuntimeObject> {
    let mut main_content = Vec::new();
    let mut named_content = Vec::new();
    let mut index = 0;
    let mut gather_count = 0;
    let mut choice_count = 0;
    let mut needs_terminal_gather = false;
    let mut last_gather_location = None;
    let mut last_section_had_choice = false;
    let mut choice_labels = HashMap::new();
    let objects = weave.content();

    // Check if there's an explicit gather anywhere in the weave
    let has_explicit_gather = objects.iter().any(|o| matches!(o, Object::Gather(_)));

    while index < objects.len() {
        match &objects[index] {
            Object::Text(_)
            | Object::ContentList(_)
            | Object::Glue(_)
            | Object::Divert(_)
            | Object::Tag(_)
            | Object::Sequence(_)
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
                    gather_count,
                    &path_mode,
                    has_explicit_gather,
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
                if let Some(identifier) = gather.identifier() {
                    choice_labels.insert(identifier.to_string(), gather_name.clone());
                }
                let auto_enter_gather = !last_section_had_choice;

                let mut gather_content = Vec::new();
                let mut gather_named_content = Vec::new();
                index += 1;

                let gather_path_mode = path_mode.for_gather(&gather_name);
                let gather_has_choice = lower_weave_section(
                    objects,
                    &mut index,
                    &mut gather_content,
                    &mut gather_named_content,
                    &mut choice_count,
                    &mut needs_terminal_gather,
                    &mut choice_labels,
                    global_labels,
                    gather_count,
                    &gather_path_mode,
                    has_explicit_gather,
                );
                if !gather_named_content.is_empty() {
                    gather_content.push(RuntimeObject::NamedContent(gather_named_content));
                }
                if matches!(path_mode, ChoicePathMode::NestedRoot { .. })
                    && !gather_has_choice
                    && !ends_with_flow_terminator(&gather_content)
                {
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
                    flags: if gather.identifier().is_some() {
                        Some(5)
                    } else {
                        None
                    },
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
                last_section_had_choice = gather_has_choice;
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
                    gather_count,
                    &path_mode,
                    has_explicit_gather,
                );
            }
        }
    }

    if matches!(path_mode, ChoicePathMode::Root) && has_explicit_gather {
        if let Some(location) = last_gather_location {
            if let Some(container) =
                gather_container_at_location_mut(&mut main_content, &mut named_content, &location)
            {
                container
                    .content
                    .push(RuntimeObject::Container(done_container(&format!(
                        "g-{gather_count}"
                    ))));
            }
        }
    } else if !matches!(path_mode, ChoicePathMode::NestedRoot { .. })
        && !has_explicit_gather
        && needs_terminal_gather
    {
        // Add a terminal gather for choices that divert to it.
        named_content.push(done_container(&format!("g-{gather_count}")));
    }
    if !named_content.is_empty() {
        main_content.push(RuntimeObject::NamedContent(named_content));
    }

    main_content
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
    gather_count: usize,
    path_mode: &ChoicePathMode,
    has_explicit_gather: bool,
) -> bool {
    let mut section_has_choice = false;
    while *index < objects.len() {
        match &objects[*index] {
            Object::Gather(_) => break,
            Object::Text(_)
            | Object::ContentList(_)
            | Object::Glue(_)
            | Object::Divert(_)
            | Object::Tag(_)
            | Object::Sequence(_)
            | Object::Weave(_) => {
                lower_object_into_with_context(content, &objects[*index], path_mode, global_labels);
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
                    gather_count,
                    path_mode,
                    has_explicit_gather,
                );
            }
        }
    }
    section_has_choice
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
    gather_count: usize,
    path_mode: &ChoicePathMode,
    has_explicit_gather: bool,
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
    let choice_container_path =
        choice_point_target(path_mode, choice.has_start_content(), choice_index);

    match choice_outer(
        choice,
        &choice_container_path,
        content.len(),
        path_mode,
        choice_labels,
        global_labels,
    ) {
        ChoiceOuter::Inline(objects) => content.extend(objects),
        ChoiceOuter::Nested(container) => content.push(RuntimeObject::Container(container)),
    }

    let mut choice_content = Vec::new();
    let choice_content_path_mode = path_mode.with_self_target_relative(choice.has_start_content());
    if choice.has_start_content() {
        choice_content =
            choice_container_prefix(path_mode, &choice_container_name, content.len() - 1, 2);
    }
    choice_content.extend(lower_content_list_with_context(
        choice.inner_content(),
        &choice_content_path_mode,
        global_labels,
    ));
    let nested_choice_content_path_mode = path_mode.for_choice_nested_content(
        &choice_container_name,
        &gather_container_name,
        has_following_gather,
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
        lower_object_into_with_context(
            &mut choice_content,
            &objects[*index],
            &nested_choice_content_path_mode,
            global_labels,
        );
        *index += 1;
    }

    let include_gather = if has_nested_weave_content {
        false
    } else if has_explicit_gather {
        true
    } else {
        match path_mode {
            ChoicePathMode::Root
            | ChoicePathMode::RootGather { .. }
            | ChoicePathMode::NestedRoot { .. } => true,
            ChoicePathMode::Flow { .. } => false,
        }
    };
    if include_gather {
        choice_content.push(RuntimeObject::Divert {
            target: gather_target(path_mode, &gather_container_name, has_following_gather),
            variable: false,
        });
        *needs_terminal_gather = true;
    }

    named_content.push(Container {
        content: choice_content,
        name: Some(choice_container_name),
        // Only set visitsShouldBeCounted flag (5) for once-only choices.
        flags: if choice.once_only() { Some(5) } else { None },
        merge_tail_metadata: true,
    });
    if let Some(identifier) = choice.identifier() {
        choice_labels.insert(identifier.to_string(), format!("c-{choice_index}"));
    }
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
            outer_return_target(path_mode, choice_point_index)
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
        outer_content.extend(lower_content_list_with_context(
            choice_only_content,
            path_mode,
            global_labels,
        ));
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::EndString));
    }

    if let Some(condition) = choice.condition() {
        lower_expression_into(
            &mut outer_content,
            condition,
            choice_labels,
            global_labels,
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
        .map(|cl| lower_content_list_with_context(cl, path_mode, global_labels))
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
            choice_content_return_target(path_mode, choice_container_name)
        )),
        RuntimeObject::ControlCommand(ControlCommand::EvalEnd),
        RuntimeObject::VariableAssignment("$r".to_string()),
        RuntimeObject::Divert {
            target: start_content_target(path_mode, choice_point_index),
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
    global_labels: &HashMap<String, String>,
) -> Vec<RuntimeObject> {
    let mut content = Vec::new();
    for object in content_list.objects() {
        lower_object_into_with_context(&mut content, object, path_mode, global_labels);
    }
    content
}

fn lower_expression_into(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    path_mode: &ChoicePathMode,
    has_start_content: bool,
) {
    match expression {
        Expression::NumberBool(value) => content.push(RuntimeObject::Bool(*value)),
        Expression::VariableReference(name) => {
            if let Some(choice_container_name) = choice_labels.get(name) {
                content.push(RuntimeObject::ReadCount(choice_label_count_target(
                    path_mode,
                    choice_container_name,
                    has_start_content,
                )));
            } else if let Some(label_target) = global_labels.get(name) {
                content.push(RuntimeObject::ReadCount(label_target.clone()));
            } else {
                content.push(RuntimeObject::VariableReference(name.clone()));
            }
        }
        Expression::Binary {
            operator,
            left,
            right,
        } => {
            lower_expression_into(
                content,
                left,
                choice_labels,
                global_labels,
                path_mode,
                has_start_content,
            );
            lower_expression_into(
                content,
                right,
                choice_labels,
                global_labels,
                path_mode,
                has_start_content,
            );
            content.push(RuntimeObject::NativeFunction(
                operator_runtime_name(*operator).to_string(),
            ));
        }
        Expression::MultipleCondition(expressions) => {
            for (index, expression) in expressions.iter().enumerate() {
                lower_expression_into(
                    content,
                    expression,
                    choice_labels,
                    global_labels,
                    path_mode,
                    has_start_content,
                );
                if index > 0 {
                    content.push(RuntimeObject::NativeFunction("&&".to_string()));
                }
            }
        }
    }
}

fn operator_runtime_name(operator: BinaryOperator) -> &'static str {
    operator.runtime_name()
}

fn lower_sequence(sequence: &Sequence, global_labels: &HashMap<String, String>) -> Container {
    let mut content = vec![
        RuntimeObject::ControlCommand(ControlCommand::EvalStart),
        RuntimeObject::ControlCommand(ControlCommand::VisitIndex),
    ];

    match sequence.sequence_type() {
        SequenceType::Cycle => {
            content.push(RuntimeObject::Int(sequence.elements().len() as i32));
            content.push(RuntimeObject::NativeFunction("%".to_string()));
        }
        SequenceType::Stopping => {
            content.push(RuntimeObject::Int(
                sequence.elements().len().saturating_sub(1) as i32,
            ));
            content.push(RuntimeObject::NativeFunction("MIN".to_string()));
        }
        SequenceType::Once => {
            content.push(RuntimeObject::Int(sequence.elements().len() as i32));
            content.push(RuntimeObject::NativeFunction("MIN".to_string()));
        }
    }

    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));

    let branch_count = match sequence.sequence_type() {
        SequenceType::Once => sequence.elements().len() + 1,
        SequenceType::Cycle | SequenceType::Stopping => sequence.elements().len(),
    };

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
            let mut branch_content = vec![RuntimeObject::ControlCommand(ControlCommand::Pop)];
            if let Some(element) = element {
                branch_content.extend(lower_content_list_with_context(
                    element,
                    &ChoicePathMode::Root,
                    global_labels,
                ));
            }
            branch_content.push(RuntimeObject::Divert {
                target: format!(".^.^.{post_sequence_index}"),
                variable: false,
            });
            Container {
                content: branch_content,
                name: Some(format!("s{index}")),
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

fn lower_object_into(
    content: &mut Vec<RuntimeObject>,
    object: &Object,
    global_labels: &HashMap<String, String>,
) {
    match object {
        Object::Text(text) => content.push(RuntimeObject::String(text.text().to_string())),
        Object::ContentList(content_list) => {
            content.extend(lower_content_list_with_context(
                content_list,
                &ChoicePathMode::Root,
                global_labels,
            ));
        }
        Object::Glue(_) => content.push(RuntimeObject::Glue),
        Object::Divert(divert) => push_divert(content, divert.target()),
        Object::Choice(_) => {}
        Object::Gather(_) => {} // Handled in lower_choice_weave
        Object::Tag(tag) => content.push(RuntimeObject::Tag {
            is_start: tag.is_start(),
        }),
        Object::Sequence(sequence) => content.push(RuntimeObject::Container(lower_sequence(
            sequence,
            global_labels,
        ))),
        Object::Weave(weave) => content.push(RuntimeObject::Container(Container {
            content: lower_choice_weave(weave, ChoicePathMode::Root, global_labels),
            name: None,
            flags: None,
            merge_tail_metadata: true,
        })),
    }
}

fn lower_object_into_with_context(
    content: &mut Vec<RuntimeObject>,
    object: &Object,
    path_mode: &ChoicePathMode,
    global_labels: &HashMap<String, String>,
) {
    match object {
        Object::Text(text) => content.push(RuntimeObject::String(text.text().to_string())),
        Object::ContentList(content_list) => {
            content.extend(lower_content_list_with_context(
                content_list,
                path_mode,
                global_labels,
            ));
        }
        Object::Glue(_) => content.push(RuntimeObject::Glue),
        Object::Divert(divert) => push_divert_with_context(content, divert.target(), path_mode),
        Object::Choice(_) => {}
        Object::Gather(_) => {} // Handled in lower_choice_weave
        Object::Tag(tag) => content.push(RuntimeObject::Tag {
            is_start: tag.is_start(),
        }),
        Object::Sequence(sequence) => content.push(RuntimeObject::Container(lower_sequence(
            sequence,
            global_labels,
        ))),
        Object::Weave(weave) => {
            let nested_path_mode = path_mode.for_nested_weave(content.len());
            content.push(RuntimeObject::Container(Container {
                content: lower_choice_weave(weave, nested_path_mode, global_labels),
                name: None,
                flags: None,
                merge_tail_metadata: true,
            }));
        }
    }
}

fn push_divert_with_context(
    content: &mut Vec<RuntimeObject>,
    target: &DivertTarget,
    path_mode: &ChoicePathMode,
) {
    match target {
        DivertTarget::Done => content.push(RuntimeObject::ControlCommand(ControlCommand::Done)),
        DivertTarget::End => content.push(RuntimeObject::ControlCommand(ControlCommand::End)),
        DivertTarget::Path(target) => {
            let resolved_target = resolve_divert_target(target, path_mode);
            content.push(RuntimeObject::Divert {
                target: resolved_target,
                variable: false,
            });
        }
        DivertTarget::Empty => content.push(RuntimeObject::Divert {
            target: String::new(),
            variable: false,
        }),
    }
}

/// Resolve a divert target path, converting absolute flow names to relative paths
/// when the target is a sibling stitch or child stitch inside a choice container.
fn resolve_divert_target(target: &str, path_mode: &ChoicePathMode) -> String {
    match path_mode {
        ChoicePathMode::Root
        | ChoicePathMode::RootGather { .. }
        | ChoicePathMode::NestedRoot { .. } => target.to_string(),
        ChoicePathMode::Flow {
            parent_flow_name,
            flow_name,
            sibling_stitch_names,
            ..
        } => {
            if let Some((first_part, second_part)) = target.split_once('.') {
                if parent_flow_name.is_none()
                    && first_part == flow_name
                    && sibling_stitch_names.iter().any(|name| name == second_part)
                {
                    return resolve_single_stitch_target(second_part, path_mode);
                }
                return target.to_string();
            }

            // Handle simple target names
            resolve_single_stitch_target(target, path_mode)
        }
    }
}

/// Resolve a single stitch name to a relative path if applicable.
fn resolve_single_stitch_target(target: &str, path_mode: &ChoicePathMode) -> String {
    match path_mode {
        ChoicePathMode::Root
        | ChoicePathMode::RootGather { .. }
        | ChoicePathMode::NestedRoot { .. } => target.to_string(),
        ChoicePathMode::Flow {
            sibling_stitch_names,
            parent_flow_name,
            flow_name,
            self_target_relative,
            ..
        } => {
            if parent_flow_name.is_some() {
                // We're in a stitch - check if target is a sibling stitch
                if sibling_stitch_names.iter().any(|s| s == target) {
                    // From inside a choice container in a stitch, we need 4 levels up:
                    // 1. Named content container (containing c-0, c-1, g-0)
                    // 2. Weave content array
                    // 3. Stitch container
                    // 4. Knot container (where sibling stitches are defined)
                    ".^.^.^.^.".to_string() + target
                } else if Some(target) == parent_flow_name.as_deref() {
                    // Target is the parent knot itself
                    ".^.^.^.^".to_string()
                } else {
                    target.to_string()
                }
            } else {
                // We're in a knot
                if sibling_stitch_names.iter().any(|s| s == target) {
                    // Target is a child stitch - 3 levels up:
                    // 1. Named content container (containing c-0, c-1, g-0)
                    // 2. Weave content array
                    // 3. Knot container (where child stitches are defined)
                    ".^.^.^.".to_string() + target
                } else if *self_target_relative && target == *flow_name {
                    // Target is the knot itself - 3 levels up
                    ".^.^.^".to_string()
                } else {
                    target.to_string()
                }
            }
        }
    }
}

fn choice_point_target(
    path_mode: &ChoicePathMode,
    has_start_content: bool,
    choice_index: usize,
) -> String {
    match path_mode {
        ChoicePathMode::Root => format!("0.c-{choice_index}"),
        ChoicePathMode::RootGather { .. } if has_start_content => format!(".^.^.c-{choice_index}"),
        ChoicePathMode::RootGather { .. } => format!(".^.c-{choice_index}"),
        ChoicePathMode::NestedRoot { .. } if has_start_content => format!(".^.^.c-{choice_index}"),
        ChoicePathMode::NestedRoot { .. } => format!(".^.c-{choice_index}"),
        ChoicePathMode::Flow { .. } if has_start_content => format!(".^.^.c-{choice_index}"),
        ChoicePathMode::Flow { .. } => format!(".^.c-{choice_index}"),
    }
}

fn outer_return_target(path_mode: &ChoicePathMode, choice_point_index: usize) -> String {
    match path_mode {
        ChoicePathMode::Root => format!("0.{choice_point_index}"),
        ChoicePathMode::RootGather { gather_name } => {
            format!("0.{gather_name}.{choice_point_index}")
        }
        ChoicePathMode::NestedRoot { container_path, .. } => {
            format!("{container_path}.{choice_point_index}")
        }
        ChoicePathMode::Flow { container_path, .. } => {
            format!("{container_path}.{choice_point_index}")
        }
    }
}

fn choice_content_return_target(path_mode: &ChoicePathMode, choice_container_name: &str) -> String {
    match path_mode {
        ChoicePathMode::Root => format!("0.{choice_container_name}"),
        ChoicePathMode::RootGather { gather_name } => {
            format!("0.{gather_name}.{choice_container_name}")
        }
        ChoicePathMode::NestedRoot { container_path, .. } => {
            format!("{container_path}.{choice_container_name}")
        }
        ChoicePathMode::Flow { container_path, .. } => {
            format!("{container_path}.{choice_container_name}")
        }
    }
}

fn start_content_target(path_mode: &ChoicePathMode, choice_point_index: usize) -> String {
    match path_mode {
        ChoicePathMode::Root => format!("0.{choice_point_index}.s"),
        ChoicePathMode::RootGather { .. } => format!(".^.^.{choice_point_index}.s"),
        ChoicePathMode::NestedRoot { .. } => format!(".^.^.{choice_point_index}.s"),
        ChoicePathMode::Flow { .. } => format!(".^.^.{choice_point_index}.s"),
    }
}

fn gather_target(
    path_mode: &ChoicePathMode,
    gather_container_name: &str,
    has_following_gather: bool,
) -> String {
    match path_mode {
        ChoicePathMode::Root => format!("0.{gather_container_name}"),
        ChoicePathMode::RootGather { .. } => format!("0.{gather_container_name}"),
        ChoicePathMode::NestedRoot { .. } if has_following_gather => {
            format!(".^.^.{gather_container_name}")
        }
        ChoicePathMode::NestedRoot { gather_target, .. } => gather_target.clone(),
        ChoicePathMode::Flow {
            flow_name,
            container_path,
            ..
        } => up_path(
            2 + flow_container_extra_depth(flow_name, container_path),
            gather_container_name,
        ),
    }
}

fn flow_container_extra_depth(flow_name: &str, container_path: &str) -> usize {
    let root_path = format!("{flow_name}.0");
    let Some(rest) = container_path.strip_prefix(&root_path) else {
        return 0;
    };
    rest.trim_start_matches('.')
        .split('.')
        .filter(|part| !part.is_empty())
        .count()
}

fn up_path(levels: usize, target: &str) -> String {
    format!("{}.{target}", vec!["^"; levels].join(".")).replacen('^', ".^", 1)
}

fn choice_label_count_target(
    path_mode: &ChoicePathMode,
    choice_container_name: &str,
    has_start_content: bool,
) -> String {
    match path_mode {
        ChoicePathMode::Root
        | ChoicePathMode::RootGather { .. }
        | ChoicePathMode::NestedRoot { .. }
            if has_start_content =>
        {
            format!(".^.^.^.{choice_container_name}")
        }
        ChoicePathMode::Root
        | ChoicePathMode::RootGather { .. }
        | ChoicePathMode::NestedRoot { .. } => {
            format!(".^.^.{choice_container_name}")
        }
        ChoicePathMode::Flow { .. } if has_start_content => {
            format!(".^.^.^.{choice_container_name}")
        }
        ChoicePathMode::Flow { .. } => format!(".^.^.{choice_container_name}"),
    }
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

fn push_divert(content: &mut Vec<RuntimeObject>, target: &DivertTarget) {
    match target {
        DivertTarget::Done => content.push(RuntimeObject::ControlCommand(ControlCommand::Done)),
        DivertTarget::End => content.push(RuntimeObject::ControlCommand(ControlCommand::End)),
        DivertTarget::Path(target) => content.push(RuntimeObject::Divert {
            target: target.clone(),
            variable: false,
        }),
        DivertTarget::Empty => content.push(RuntimeObject::Divert {
            target: String::new(),
            variable: false,
        }),
    }
}

fn done_container(name: &str) -> Container {
    Container {
        content: vec![RuntimeObject::ControlCommand(ControlCommand::Done)],
        name: Some(name.to_string()),
        flags: None,
        merge_tail_metadata: true,
    }
}
