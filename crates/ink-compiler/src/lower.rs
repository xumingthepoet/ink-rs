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
    String(String),
    ControlCommand(ControlCommand),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlCommand {
    Done,
}

pub(crate) fn lower(story: &CheckedStory) -> StageOutput<RuntimeProgram> {
    let mut main_content = Vec::new();

    for node in &story.parsed.nodes {
        match node {
            AstNode::TextLine { text, .. } => {
                main_content.push(RuntimeObject::String(text.clone()));
                main_content.push(RuntimeObject::String("\n".to_string()));
            }
        }
    }

    main_content.push(RuntimeObject::Container(Container {
        content: vec![RuntimeObject::ControlCommand(ControlCommand::Done)],
        name: Some("g-0".to_string()),
        flags: None,
    }));

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
