use crate::{
    analysis::CheckedStory,
    compiler::StageOutput,
    parsed::{Choice, ContentList, DivertTarget, Flow, Object, Weave},
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeObject {
    Container(Container),
    NamedContent(Vec<Container>),
    String(String),
    ControlCommand(ControlCommand),
    Divert { target: String, variable: bool },
    DivertTarget(String),
    VariableAssignment(String),
    ChoicePoint { target: String, flags: i32 },
    Glue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlCommand {
    Done,
    End,
    EvalStart,
    EvalEnd,
    BeginString,
    EndString,
}

enum ChoiceOuter {
    Inline(Vec<RuntimeObject>),
    Nested(Container),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ChoicePathMode {
    Root,
    Flow {
        flow_name: String,
        parent_flow_name: Option<String>,
        sibling_stitch_names: Vec<String>,
    },
}

pub(crate) fn lower(story: &CheckedStory) -> StageOutput<RuntimeProgram> {
    let root_weave = story.parsed.root_weave();
    let main_content = lower_root_weave(root_weave);

    let main_container = RuntimeObject::Container(Container {
        content: main_content,
        name: None,
        flags: None,
    });

    let mut root_content = vec![
        main_container,
        RuntimeObject::ControlCommand(ControlCommand::Done),
    ];

    let flow_containers = story
        .parsed
        .flows()
        .iter()
        .map(lower_flow)
        .collect::<Vec<_>>();
    if !flow_containers.is_empty() {
        root_content.push(RuntimeObject::NamedContent(flow_containers));
    }

    let root = Container {
        content: root_content,
        name: None,
        flags: None,
    };

    StageOutput {
        artifact: Some(RuntimeProgram { root }),
        diagnostics: Vec::new(),
    }
}

fn lower_root_weave(weave: &Weave) -> Vec<RuntimeObject> {
    if weave_has_choice(weave) {
        lower_choice_weave(weave, ChoicePathMode::Root)
    } else {
        let mut content = lower_linear_weave(weave);
        content.push(RuntimeObject::Container(done_container("g-0")));
        content
    }
}

fn lower_flow(flow: &Flow) -> Container {
    // Collect child stitch names upfront so knot-level choices can reference them
    let child_stitch_names: Vec<String> = flow
        .child_flows()
        .iter()
        .map(|f| f.name().to_string())
        .collect();
    lower_flow_with_context(flow, None, &child_stitch_names)
}

fn lower_flow_with_context(
    flow: &Flow,
    parent_knot_name: Option<&str>,
    sibling_stitch_names: &[String],
) -> Container {
    let mut content = Vec::new();

    // Lower any content in the flow's own weave
    if weave_has_choice(flow.weave()) {
        // For stitches inside a knot, pass the parent knot name and sibling stitch names
        let path_mode = ChoicePathMode::Flow {
            flow_name: flow.name().to_string(),
            parent_flow_name: parent_knot_name.map(|s| s.to_string()),
            sibling_stitch_names: sibling_stitch_names.to_vec(),
        };
        content.push(RuntimeObject::Container(Container {
            content: lower_choice_weave(flow.weave(), path_mode),
            name: None,
            flags: None,
        }));
    } else if !flow.weave().content().is_empty() {
        content.extend(lower_linear_weave(flow.weave()));
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
                lower_flow_with_context(child, Some(flow.name()), &child_stitch_names)
            })
            .collect();
        content.push(RuntimeObject::NamedContent(child_containers));
    }

    Container {
        content,
        name: Some(flow.name().to_string()),
        flags: None,
    }
}

fn weave_has_choice(weave: &Weave) -> bool {
    weave
        .content()
        .iter()
        .any(|object| matches!(object, Object::Choice(_)))
}

fn lower_linear_weave(weave: &Weave) -> Vec<RuntimeObject> {
    let mut content = Vec::new();
    for object in weave.content() {
        lower_object_into(&mut content, object);
    }
    content
}

fn lower_choice_weave(weave: &Weave, path_mode: ChoicePathMode) -> Vec<RuntimeObject> {
    let mut main_content = Vec::new();
    let mut named_content = Vec::new();
    let mut index = 0;
    let mut any_gather = false;
    let objects = weave.content();

    while index < objects.len() {
        match &objects[index] {
            Object::Text(_) | Object::Glue(_) | Object::Divert(_) => {
                lower_object_into(&mut main_content, &objects[index]);
                index += 1;
            }
            Object::Choice(choice) => {
                let choice_index = named_content.len();
                let choice_container_name = format!("c-{choice_index}");
                let gather_container_name = "g-0";
                let choice_container_path =
                    choice_point_target(&path_mode, choice.has_start_content(), choice_index);

                match choice_outer(
                    choice,
                    &choice_container_path,
                    main_content.len(),
                    &path_mode,
                ) {
                    ChoiceOuter::Inline(objects) => main_content.extend(objects),
                    ChoiceOuter::Nested(container) => {
                        main_content.push(RuntimeObject::Container(container))
                    }
                }

                let mut choice_content = Vec::new();
                if choice.has_start_content() {
                    choice_content = choice_container_prefix(
                        &path_mode,
                        &choice_container_name,
                        main_content.len() - 1,
                        2,
                    );
                }
                choice_content.extend(lower_content_list_with_context(choice.inner_content(), &path_mode));

                index += 1;
                while index < objects.len() {
                    if matches!(objects[index], Object::Choice(_)) {
                        break;
                    }
                    lower_object_into_with_context(
                        &mut choice_content,
                        &objects[index],
                        &path_mode,
                    );
                    index += 1;
                }

                let include_gather = match path_mode {
                    ChoicePathMode::Root => true,
                    ChoicePathMode::Flow { .. } => !ends_with_flow_terminator(&choice_content),
                };
                if include_gather {
                    choice_content.push(RuntimeObject::Divert {
                        target: gather_target(&path_mode, gather_container_name),
                        variable: false,
                    });
                    any_gather = true;
                }

                named_content.push(Container {
                    content: choice_content,
                    name: Some(choice_container_name),
                    flags: Some(5),
                });
            }
        }
    }

    if any_gather {
        named_content.push(done_container("g-0"));
    }
    if !named_content.is_empty() {
        main_content.push(RuntimeObject::NamedContent(named_content));
    }

    main_content
}

fn choice_outer(
    choice: &Choice,
    choice_container_path: &str,
    choice_point_index: usize,
    path_mode: &ChoicePathMode,
) -> ChoiceOuter {
    let mut outer_content = Vec::new();
    let has_eval_content = choice.has_start_content() || choice.has_choice_only_content();

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
        }));
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::EndString));
    }

    if let Some(choice_only_content) = choice.choice_only_content() {
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::BeginString));
        outer_content.extend(lower_content_list_with_context(choice_only_content, path_mode));
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::EndString));
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
        .map(|cl| lower_content_list_with_context(cl, path_mode))
        .unwrap_or_default();
    start_content.push(RuntimeObject::Divert {
        target: "$r".to_string(),
        variable: true,
    });
    outer_content.push(RuntimeObject::NamedContent(vec![Container {
        content: start_content,
        name: Some("s".to_string()),
        flags: None,
    }]));

    ChoiceOuter::Nested(Container {
        content: outer_content,
        name: None,
        flags: None,
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
        }),
    ]
}

fn lower_content_list_with_context(content_list: &ContentList, path_mode: &ChoicePathMode) -> Vec<RuntimeObject> {
    let mut content = Vec::new();
    for object in content_list.objects() {
        lower_object_into_with_context(&mut content, object, path_mode);
    }
    content
}

fn lower_object_into(content: &mut Vec<RuntimeObject>, object: &Object) {
    match object {
        Object::Text(text) => content.push(RuntimeObject::String(text.text().to_string())),
        Object::Glue(_) => content.push(RuntimeObject::Glue),
        Object::Divert(divert) => push_divert(content, divert.target()),
        Object::Choice(_) => {}
    }
}

fn lower_object_into_with_context(
    content: &mut Vec<RuntimeObject>,
    object: &Object,
    path_mode: &ChoicePathMode,
) {
    match object {
        Object::Text(text) => content.push(RuntimeObject::String(text.text().to_string())),
        Object::Glue(_) => content.push(RuntimeObject::Glue),
        Object::Divert(divert) => push_divert_with_context(content, divert.target(), path_mode),
        Object::Choice(_) => {}
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
        ChoicePathMode::Root => target.to_string(),
        ChoicePathMode::Flow {
            parent_flow_name,
            flow_name,
            sibling_stitch_names,
            ..
        } => {
            // Check if target is a dotted path like "knot.stitch"
            if let Some(dot_pos) = target.find('.') {
                let first_part = &target[..dot_pos];
                let second_part = &target[dot_pos + 1..];

                // Only strip prefix if first part matches parent/current flow
                // AND second part is a known child stitch
                let prefix_matches = parent_flow_name.as_deref() == Some(first_part)
                    || first_part == flow_name;
                if prefix_matches
                    && sibling_stitch_names.iter().any(|s| s == second_part)
                {
                    return resolve_single_stitch_target(second_part, path_mode);
                }
            }

            // Handle simple target names
            resolve_single_stitch_target(target, path_mode)
        }
    }
}

/// Resolve a single stitch name to a relative path if applicable.
fn resolve_single_stitch_target(target: &str, path_mode: &ChoicePathMode) -> String {
    match path_mode {
        ChoicePathMode::Root => target.to_string(),
        ChoicePathMode::Flow {
            sibling_stitch_names,
            parent_flow_name,
            flow_name,
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
                } else if target == *flow_name {
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
        ChoicePathMode::Flow { .. } if has_start_content => format!(".^.^.c-{choice_index}"),
        ChoicePathMode::Flow { .. } => format!(".^.c-{choice_index}"),
    }
}

fn outer_return_target(path_mode: &ChoicePathMode, choice_point_index: usize) -> String {
    match path_mode {
        ChoicePathMode::Root => format!("0.{choice_point_index}"),
        ChoicePathMode::Flow { flow_name, .. } => format!("{flow_name}.0.{choice_point_index}"),
    }
}

fn choice_content_return_target(path_mode: &ChoicePathMode, choice_container_name: &str) -> String {
    match path_mode {
        ChoicePathMode::Root => format!("0.{choice_container_name}"),
        ChoicePathMode::Flow { flow_name, .. } => format!("{flow_name}.0.{choice_container_name}"),
    }
}

fn start_content_target(path_mode: &ChoicePathMode, choice_point_index: usize) -> String {
    match path_mode {
        ChoicePathMode::Root => format!("0.{choice_point_index}.s"),
        ChoicePathMode::Flow { .. } => format!(".^.^.{choice_point_index}.s"),
    }
}

fn gather_target(path_mode: &ChoicePathMode, gather_container_name: &str) -> String {
    match path_mode {
        ChoicePathMode::Root => format!("0.{gather_container_name}"),
        ChoicePathMode::Flow { .. } => format!(".^.^.{gather_container_name}"),
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
    }
}
