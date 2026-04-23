use crate::{analysis::CheckedStory, ast::AstNode, compiler::StageOutput};

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

pub(crate) fn lower(story: &CheckedStory) -> StageOutput<RuntimeProgram> {
    let main_content = if story
        .parsed
        .nodes
        .iter()
        .any(|node| matches!(node, AstNode::Choice { .. }))
    {
        lower_choice_weave(&story.parsed.nodes)
    } else {
        let mut content = Vec::new();

        for node in &story.parsed.nodes {
            match node {
                AstNode::TextLine { text, .. } => push_text_line(&mut content, text),
                AstNode::Divert { target, .. } => push_divert(&mut content, target),
                AstNode::Choice { .. } => {}
            }
        }

        content.push(RuntimeObject::Container(done_container("g-0")));
        content
    };

    let main_container = RuntimeObject::Container(Container {
        content: main_content,
        name: None,
        flags: None,
    });

    let root = Container {
        content: vec![
            main_container,
            RuntimeObject::ControlCommand(ControlCommand::Done),
        ],
        name: None,
        flags: None,
    };

    StageOutput {
        artifact: Some(RuntimeProgram { root }),
        diagnostics: Vec::new(),
    }
}

fn lower_choice_weave(nodes: &[AstNode]) -> Vec<RuntimeObject> {
    let mut main_content = Vec::new();
    let mut iter = nodes.iter().peekable();

    while let Some(node) = iter.next() {
        match node {
            AstNode::TextLine { text, .. } => push_text_line(&mut main_content, text),
            AstNode::Divert { target, .. } => push_divert(&mut main_content, target),
            AstNode::Choice { text, .. } => {
                let choice_index = 0;
                let choice_point_index = main_content.len();
                let choice_container_name = format!("c-{choice_index}");
                let choice_point_path = format!("0.{choice_point_index}");
                let choice_container_path = format!("0.{choice_container_name}");
                let gather_container_name = "g-0";

                main_content.push(choice_point(
                    text,
                    &choice_point_path,
                    &choice_container_path,
                    1,
                ));

                let mut choice_content =
                    choice_container_prefix(&choice_container_path, &choice_point_path, 2);
                choice_content.push(RuntimeObject::String("\n".to_string()));

                for remaining in iter {
                    match remaining {
                        AstNode::TextLine { text, .. } => push_text_line(&mut choice_content, text),
                        AstNode::Divert { target, .. } => push_divert(&mut choice_content, target),
                        AstNode::Choice { .. } => {}
                    }
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

fn choice_point(
    choice_text: &str,
    choice_point_path: &str,
    choice_container_path: &str,
    return_index: usize,
) -> RuntimeObject {
    RuntimeObject::Container(Container {
        content: vec![
            RuntimeObject::ControlCommand(ControlCommand::EvalStart),
            RuntimeObject::DivertTarget(format!("{choice_point_path}.$r{return_index}")),
            RuntimeObject::VariableAssignment("$r".to_string()),
            RuntimeObject::ControlCommand(ControlCommand::BeginString),
            RuntimeObject::Divert {
                target: ".^.s".to_string(),
                variable: false,
            },
            RuntimeObject::Container(Container {
                content: Vec::new(),
                name: Some(format!("$r{return_index}")),
                flags: None,
            }),
            RuntimeObject::ControlCommand(ControlCommand::EndString),
            RuntimeObject::ControlCommand(ControlCommand::EvalEnd),
            RuntimeObject::ChoicePoint {
                target: choice_container_path.to_string(),
                flags: 18,
            },
            RuntimeObject::NamedContent(vec![Container {
                content: vec![
                    RuntimeObject::String(choice_text.to_string()),
                    RuntimeObject::Divert {
                        target: "$r".to_string(),
                        variable: true,
                    },
                ],
                name: Some("s".to_string()),
                flags: None,
            }]),
        ],
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

fn push_text_line(content: &mut Vec<RuntimeObject>, text: &str) {
    content.push(RuntimeObject::String(text.to_string()));
    content.push(RuntimeObject::String("\n".to_string()));
}

fn push_divert(content: &mut Vec<RuntimeObject>, target: &str) {
    match target {
        "DONE" => content.push(RuntimeObject::ControlCommand(ControlCommand::Done)),
        "END" => content.push(RuntimeObject::ControlCommand(ControlCommand::End)),
        other => content.push(RuntimeObject::Divert {
            target: other.to_string(),
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
