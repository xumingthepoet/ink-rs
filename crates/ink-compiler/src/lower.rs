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
    if weave
        .content()
        .iter()
        .any(|object| matches!(object, Object::Choice(_)))
    {
        lower_choice_weave(weave)
    } else {
        let mut content = lower_linear_weave(weave);
        content.push(RuntimeObject::Container(done_container("g-0")));
        content
    }
}

fn lower_flow(flow: &Flow) -> Container {
    Container {
        content: lower_linear_weave(flow.weave()),
        name: Some(flow.name().to_string()),
        flags: None,
    }
}

fn lower_linear_weave(weave: &Weave) -> Vec<RuntimeObject> {
    let mut content = Vec::new();
    for object in weave.content() {
        lower_object_into(&mut content, object);
    }
    content
}

fn lower_choice_weave(weave: &Weave) -> Vec<RuntimeObject> {
    let mut main_content = Vec::new();
    let mut iter = weave.content().iter().peekable();

    while let Some(object) = iter.next() {
        match object {
            Object::Text(_) | Object::Glue(_) | Object::Divert(_) => {
                lower_object_into(&mut main_content, object)
            }
            Object::Choice(choice) => {
                let choice_index = 0;
                let choice_container_name = format!("c-{choice_index}");
                let choice_container_path = format!("0.{choice_container_name}");
                let gather_container_name = "g-0";

                match choice_outer(choice, &choice_container_path, main_content.len()) {
                    ChoiceOuter::Inline(objects) => main_content.extend(objects),
                    ChoiceOuter::Nested(container) => {
                        main_content.push(RuntimeObject::Container(container))
                    }
                }

                let mut choice_content = Vec::new();
                if choice.has_start_content() {
                    let choice_point_path = format!("0.{}", main_content.len() - 1);
                    choice_content =
                        choice_container_prefix(&choice_container_path, &choice_point_path, 2);
                }
                choice_content.extend(lower_content_list(choice.inner_content()));

                for remaining in iter {
                    lower_object_into(&mut choice_content, remaining);
                }

                choice_content.push(RuntimeObject::Divert {
                    target: format!("0.{gather_container_name}"),
                    variable: false,
                });

                main_content.push(RuntimeObject::NamedContent(vec![
                    Container {
                        content: choice_content,
                        name: Some(choice_container_name),
                        flags: Some(5),
                    },
                    done_container(gather_container_name),
                ]));
                break;
            }
        }
    }

    main_content
}

fn choice_outer(
    choice: &Choice,
    choice_container_path: &str,
    choice_point_index: usize,
) -> ChoiceOuter {
    let mut outer_content = Vec::new();
    let has_eval_content = choice.has_start_content() || choice.has_choice_only_content();

    if has_eval_content {
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    }

    if choice.has_start_content() {
        let choice_point_path = format!("0.{choice_point_index}");
        outer_content.push(RuntimeObject::DivertTarget(format!(
            "{choice_point_path}.$r1"
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
        outer_content.extend(lower_content_list(choice_only_content));
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
        .map(lower_content_list)
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
    choice_container_path: &str,
    choice_point_path: &str,
    return_index: usize,
) -> Vec<RuntimeObject> {
    vec![
        RuntimeObject::ControlCommand(ControlCommand::EvalStart),
        RuntimeObject::DivertTarget(format!("{choice_container_path}.$r{return_index}")),
        RuntimeObject::ControlCommand(ControlCommand::EvalEnd),
        RuntimeObject::VariableAssignment("$r".to_string()),
        RuntimeObject::Divert {
            target: format!("{choice_point_path}.s"),
            variable: false,
        },
        RuntimeObject::Container(Container {
            content: Vec::new(),
            name: Some(format!("$r{return_index}")),
            flags: None,
        }),
    ]
}

fn lower_content_list(content_list: &ContentList) -> Vec<RuntimeObject> {
    let mut content = Vec::new();
    for object in content_list.objects() {
        lower_object_into(&mut content, object);
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
