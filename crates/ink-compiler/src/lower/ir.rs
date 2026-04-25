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
