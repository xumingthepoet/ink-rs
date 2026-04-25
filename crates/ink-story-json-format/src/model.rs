use serde_json::{Map, Value as JsonValue};

use crate::{json, FormatError, INK_VERSION_CURRENT};

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub ink_version: i32,
    pub root: Container,
    pub list_defs: Map<String, JsonValue>,
}

impl Program {
    pub fn new(root: Container) -> Self {
        Self {
            ink_version: INK_VERSION_CURRENT,
            root,
            list_defs: Map::new(),
        }
    }

    pub fn from_json_str(input: &str) -> Result<Self, FormatError> {
        json::program_from_str(input)
    }

    pub fn from_json_value(value: JsonValue) -> Result<Self, FormatError> {
        json::program_from_value(value)
    }

    pub fn to_json_string(&self) -> Result<String, FormatError> {
        json::program_to_string(self)
    }

    pub fn to_json_value(&self) -> JsonValue {
        json::program_to_value(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Container {
    pub content: Vec<Object>,
    pub named_content: Vec<NamedContainer>,
    pub name: Option<String>,
    pub flags: Option<i32>,
}

impl Container {
    pub fn new(content: Vec<Object>) -> Self {
        Self {
            content,
            named_content: Vec::new(),
            name: None,
            flags: None,
        }
    }

    pub fn named(name: impl Into<String>, content: Vec<Object>) -> Self {
        Self {
            content,
            named_content: Vec::new(),
            name: Some(name.into()),
            flags: None,
        }
    }

    pub fn named_with_flags(
        name: impl Into<String>,
        content: Vec<Object>,
        flags: Option<i32>,
    ) -> Self {
        Self {
            content,
            named_content: Vec::new(),
            name: Some(name.into()),
            flags,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamedContainer {
    pub name: String,
    pub container: Container,
}

impl NamedContainer {
    pub fn new(name: impl Into<String>, container: Container) -> Self {
        Self {
            name: name.into(),
            container,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Container(Container),
    Value(Value),
    ControlCommand(ControlCommand),
    NativeFunction(NativeFunction),
    Divert(Divert),
    ChoicePoint(ChoicePoint),
    VariableAssignment(VariableAssignment),
    VariableReference(VariableReference),
    Glue,
    Void,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Bool(bool),
    Int(i32),
    Float(f64),
    DivertTarget(String),
    VariablePointer(VariablePointer),
    List(ListValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariablePointer {
    pub name: String,
    pub context_index: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListValue {
    pub items: Vec<ListItemValue>,
    pub origins: Vec<String>,
}

impl ListValue {
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            origins: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListItemValue {
    pub name: String,
    pub value: i32,
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
    BeginTag,
    EndTag,
}

impl ControlCommand {
    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "done" => Some(Self::Done),
            "end" => Some(Self::End),
            "ev" => Some(Self::EvalStart),
            "out" => Some(Self::EvalOutput),
            "/ev" => Some(Self::EvalEnd),
            "str" => Some(Self::BeginString),
            "/str" => Some(Self::EndString),
            "visit" => Some(Self::VisitIndex),
            "seq" => Some(Self::SequenceShuffleIndex),
            "du" => Some(Self::Duplicate),
            "nop" => Some(Self::NoOp),
            "pop" => Some(Self::Pop),
            "~ret" => Some(Self::PopFunction),
            "->->" => Some(Self::PopTunnel),
            "thread" => Some(Self::StartThread),
            "choiceCnt" => Some(Self::ChoiceCount),
            "turn" => Some(Self::Turns),
            "turns" => Some(Self::TurnsSince),
            "readc" => Some(Self::ReadCount),
            "rnd" => Some(Self::Random),
            "srnd" => Some(Self::SeedRandom),
            "range" => Some(Self::ListRange),
            "lrnd" => Some(Self::ListRandom),
            "#" => Some(Self::BeginTag),
            "/#" => Some(Self::EndTag),
            _ => None,
        }
    }

    pub fn token(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::End => "end",
            Self::EvalStart => "ev",
            Self::EvalOutput => "out",
            Self::EvalEnd => "/ev",
            Self::BeginString => "str",
            Self::EndString => "/str",
            Self::VisitIndex => "visit",
            Self::SequenceShuffleIndex => "seq",
            Self::Duplicate => "du",
            Self::NoOp => "nop",
            Self::Pop => "pop",
            Self::PopFunction => "~ret",
            Self::PopTunnel => "->->",
            Self::StartThread => "thread",
            Self::ChoiceCount => "choiceCnt",
            Self::Turns => "turn",
            Self::TurnsSince => "turns",
            Self::ReadCount => "readc",
            Self::Random => "rnd",
            Self::SeedRandom => "srnd",
            Self::ListRange => "range",
            Self::ListRandom => "lrnd",
            Self::BeginTag => "#",
            Self::EndTag => "/#",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeFunction {
    name: String,
}

impl NativeFunction {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn from_token(token: &str) -> Self {
        if token == "L^" {
            Self::new("^")
        } else {
            Self::new(token)
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn token(&self) -> &str {
        if self.name == "^" {
            "L^"
        } else {
            &self.name
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Divert {
    pub kind: DivertKind,
    pub target: String,
    pub variable: bool,
    pub conditional: bool,
    pub external_args: Option<usize>,
}

impl Divert {
    pub fn new(kind: DivertKind, target: impl Into<String>) -> Self {
        Self {
            kind,
            target: target.into(),
            variable: false,
            conditional: false,
            external_args: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DivertKind {
    Direct,
    Function,
    Tunnel,
    External,
}

impl DivertKind {
    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Direct => "->",
            Self::Function => "f()",
            Self::Tunnel => "->t->",
            Self::External => "x()",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoicePoint {
    pub target: String,
    pub flags: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableAssignment {
    pub kind: VariableAssignmentKind,
    pub name: String,
    pub is_new_declaration: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableAssignmentKind {
    Global,
    Temporary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableReference {
    pub kind: VariableReferenceKind,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableReferenceKind {
    Variable,
    ReadCount,
}
