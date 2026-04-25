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
    pub fn unnamed(content: Vec<Object>) -> Self {
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

    pub fn from_json_value(
        value: JsonValue,
        name_hint: Option<String>,
    ) -> Result<Self, FormatError> {
        json::container_from_value(&value, name_hint)
    }

    pub fn to_json_value(&self) -> JsonValue {
        json::container_to_value(self, true)
    }

    pub fn to_json_value_with_name(&self, include_name: bool) -> JsonValue {
        json::container_to_value(self, include_name)
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
    Float(f64),
    List(ListValue),
    Void,
    NativeFunction(String),
}

impl Object {
    pub fn container(content: Vec<Object>) -> Self {
        Self::Container(Container::unnamed(content))
    }

    pub fn named_container(name: impl Into<String>, content: Vec<Object>) -> Self {
        Self::Container(Container::named(name, content))
    }

    pub fn direct_divert(target: impl Into<String>, variable: bool) -> Self {
        Self::Divert {
            target: target.into(),
            variable,
        }
    }

    pub fn from_json_value(value: JsonValue) -> Result<Self, FormatError> {
        json::object_from_value(&value)
    }

    pub fn to_json_value(&self) -> JsonValue {
        json::object_to_value(self)
    }
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
        }
    }
}

pub fn native_function_name_from_token(token: &str) -> String {
    if token == "L^" {
        "^".to_string()
    } else {
        token.to_string()
    }
}

pub fn native_function_token(name: &str) -> &str {
    if name == "^" {
        "L^"
    } else {
        name
    }
}
