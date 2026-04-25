use crate::parsed::FloatLiteral;

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

impl Container {
    pub(super) fn unnamed(content: Vec<RuntimeObject>) -> Self {
        Self {
            content,
            name: None,
            flags: None,
            merge_tail_metadata: true,
        }
    }

    pub(super) fn named(name: impl Into<String>, content: Vec<RuntimeObject>) -> Self {
        Self {
            content,
            name: Some(name.into()),
            flags: None,
            merge_tail_metadata: true,
        }
    }

    pub(super) fn named_with_flags(
        name: impl Into<String>,
        content: Vec<RuntimeObject>,
        flags: Option<i32>,
    ) -> Self {
        Self {
            content,
            name: Some(name.into()),
            flags,
            merge_tail_metadata: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeObject {
    Container(Container),
    NamedContent(Vec<Container>),
    String(String),
    ControlCommand(ControlCommand),
    Divert { target: String, variable: bool },
    TunnelDivert { target: String, variable: bool },
    FunctionDivert { target: String },
    ExternalFunction { target: String, args: usize },
    ConditionalDivert { target: String },
    DivertTarget(String),
    ReadCount(String),
    VariableAssignment(String),
    GlobalVariableAssignment(String),
    TempVariableReassignment(String),
    VariableReassignment(String),
    VariableReference(String),
    VariablePointer { name: String, context_index: i32 },
    ChoicePoint { target: String, flags: i32 },
    Glue,
    Tag { is_start: bool },
    Bool(bool),
    Int(i32),
    Float(FloatLiteral),
    Void,
    NativeFunction(String),
}

impl RuntimeObject {
    pub(super) fn container(content: Vec<RuntimeObject>) -> Self {
        Self::Container(Container::unnamed(content))
    }

    pub(super) fn named_container(name: impl Into<String>, content: Vec<RuntimeObject>) -> Self {
        Self::Container(Container::named(name, content))
    }

    pub(super) fn named_content(name: impl Into<String>, content: Vec<RuntimeObject>) -> Self {
        Self::NamedContent(vec![Container::named(name, content)])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlCommand {
    Done,
    End,
    EvalStart,
    EvalOutput,
    EvalEnd,
    BeginString,
    EndString,
    VisitIndex,
    SequenceShuffleIndex,
    Duplicate,
    NoOp,
    Pop,
    PopFunction,
    PopTunnel,
    StartThread,
    ChoiceCount,
    Turns,
    TurnsSince,
    ReadCount,
    Random,
    SeedRandom,
    ListRange,
    ListRandom,
}
