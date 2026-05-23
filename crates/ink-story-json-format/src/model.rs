use std::collections::BTreeMap;

use serde_json::Value as JsonValue;

use crate::{json, FormatError};

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub root: Container,
    pub internal_functions: BTreeMap<String, InternalFunction>,
    pub interfaces: BTreeMap<String, InterfaceDefinition>,
}

impl Program {
    pub fn new(root: Container) -> Self {
        Self {
            root,
            internal_functions: BTreeMap::new(),
            interfaces: BTreeMap::new(),
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

/// Top-level compiled-story JSON key for interface runtime metadata.
pub const INTERFACES_METADATA_KEY: &str = "interfaces";

/// Interface metadata object key for declared member names and kinds.
pub const INTERFACE_MEMBERS_KEY: &str = "members";

/// Interface metadata object key for source module names implementing an interface.
pub const INTERFACE_IMPLEMENTATIONS_KEY: &str = "implementations";

/// Object key for constructing a dynamic interface divert target from a runtime interface value.
pub const DYNAMIC_INTERFACE_TARGET_KEY: &str = "i->";

/// Object key for dispatching a dynamic interface function call from a runtime interface value.
pub const DYNAMIC_INTERFACE_FUNCTION_KEY: &str = "i()";

/// Object key for the interface name on dynamic interface instruction objects.
pub const DYNAMIC_INTERFACE_NAME_KEY: &str = "interface";

/// Object key for the argument count on dynamic interface function instruction objects.
pub const DYNAMIC_INTERFACE_ARGS_KEY: &str = "args";

/// String token for the start of a tag in compiled-story JSON.
pub const TAG_START_TOKEN: &str = "#";

/// String token for the end of a tag in compiled-story JSON.
pub const TAG_END_TOKEN: &str = "/#";

/// Array marker token for dynamic Dict values.
pub const DICT_VALUE_MARKER: &str = "dict";

/// Dict key-type token for string-key dictionaries.
pub const DICT_KEY_TYPE_STRING: &str = "string";

/// Dict key-type token for int-key dictionaries.
pub const DICT_KEY_TYPE_INT: &str = "int";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceDefinition {
    pub members: BTreeMap<String, InterfaceMemberKind>,
    pub implementations: Vec<String>,
}

impl InterfaceDefinition {
    pub fn new(
        members: BTreeMap<String, InterfaceMemberKind>,
        implementations: Vec<String>,
    ) -> Self {
        Self {
            members,
            implementations,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceMemberKind {
    Knot,
    Function,
}

impl InterfaceMemberKind {
    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "knot" => Some(Self::Knot),
            "function" => Some(Self::Function),
            _ => None,
        }
    }

    pub fn token(self) -> &'static str {
        match self {
            Self::Knot => "knot",
            Self::Function => "function",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalFunction {
    pub path: String,
    pub args: usize,
    pub arg_types: Vec<String>,
    pub return_type: String,
}

impl InternalFunction {
    pub fn new(
        path: impl Into<String>,
        arg_types: Vec<String>,
        return_type: impl Into<String>,
    ) -> Self {
        let args = arg_types.len();
        Self {
            path: path.into(),
            args,
            arg_types,
            return_type: return_type.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DictKeyType {
    String,
    Int,
}

impl DictKeyType {
    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            DICT_KEY_TYPE_STRING => Some(Self::String),
            DICT_KEY_TYPE_INT => Some(Self::Int),
            _ => None,
        }
    }

    pub fn token(self) -> &'static str {
        match self {
            Self::String => DICT_KEY_TYPE_STRING,
            Self::Int => DICT_KEY_TYPE_INT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DictKey {
    String(String),
    Int(i32),
}

impl DictKey {
    pub fn key_type(&self) -> DictKeyType {
        match self {
            Self::String(_) => DictKeyType::String,
            Self::Int(_) => DictKeyType::Int,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DictValue {
    pub key_type: DictKeyType,
    pub entries: BTreeMap<DictKey, Object>,
}

impl DictValue {
    pub fn new(
        key_type: DictKeyType,
        entries: BTreeMap<DictKey, Object>,
    ) -> Result<Self, FormatError> {
        for key in entries.keys() {
            if key.key_type() != key_type {
                return Err(FormatError::new(format!(
                    "Dict key {key:?} does not match {} key type",
                    key_type.token()
                )));
            }
        }
        Ok(Self { key_type, entries })
    }

    pub fn empty(key_type: DictKeyType) -> Self {
        Self {
            key_type,
            entries: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Container {
    pub content: Vec<Object>,
    pub named_content: Vec<NamedContainer>,
    pub name: Option<String>,
}

impl Container {
    pub fn unnamed(content: Vec<Object>) -> Self {
        Self {
            content,
            named_content: Vec::new(),
            name: None,
        }
    }

    pub fn named(name: impl Into<String>, content: Vec<Object>) -> Self {
        Self {
            content,
            named_content: Vec::new(),
            name: Some(name.into()),
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
    Divert {
        target: String,
        variable: bool,
    },
    TunnelDivert {
        target: String,
        variable: bool,
    },
    FunctionDivert {
        target: String,
    },
    ExternalFunction {
        target: String,
        args: usize,
    },
    DynamicInterfaceTarget {
        interface: String,
        member: String,
    },
    DynamicInterfaceFunctionCall {
        interface: String,
        member: String,
        args: usize,
    },
    ConditionalDivert {
        target: String,
    },
    DivertTarget(String),
    VariableAssignment(String),
    GlobalVariableAssignment(String),
    TempVariableReassignment(String),
    VariableReassignment(String),
    VariableReference(String),
    VariablePointer {
        name: String,
        context_index: i32,
    },
    ChoicePoint {
        target: String,
        flags: i32,
    },
    Glue,
    Tag {
        is_start: bool,
    },
    Bool(bool),
    Int(i32),
    Float(f64),
    ValueArray(Vec<Object>),
    ValueObject(BTreeMap<String, Object>),
    ValueDict(DictValue),
    Void,
    NativeFunction(NativeFunction),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlCommand {
    EvalStart,
    EvalOutput,
    EvalEnd,
    BeginString,
    EndString,
    Duplicate,
    NoOp,
    Pop,
    PopFunction,
    PopTunnel,
    Random,
    SeedRandom,
}

impl ControlCommand {
    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "ev" => Some(Self::EvalStart),
            "out" => Some(Self::EvalOutput),
            "/ev" => Some(Self::EvalEnd),
            "str" => Some(Self::BeginString),
            "/str" => Some(Self::EndString),
            "du" => Some(Self::Duplicate),
            "nop" => Some(Self::NoOp),
            "pop" => Some(Self::Pop),
            "~ret" => Some(Self::PopFunction),
            "->->" => Some(Self::PopTunnel),
            "rnd" => Some(Self::Random),
            "srnd" => Some(Self::SeedRandom),
            _ => None,
        }
    }

    pub fn token(self) -> &'static str {
        match self {
            Self::EvalStart => "ev",
            Self::EvalOutput => "out",
            Self::EvalEnd => "/ev",
            Self::BeginString => "str",
            Self::EndString => "/str",
            Self::Duplicate => "du",
            Self::NoOp => "nop",
            Self::Pop => "pop",
            Self::PopFunction => "~ret",
            Self::PopTunnel => "->->",
            Self::Random => "rnd",
            Self::SeedRandom => "srnd",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeFunction {
    Add,
    Subtract,
    Divide,
    Multiply,
    Mod,
    Negate,
    Equal,
    Greater,
    Less,
    GreaterThanOrEquals,
    LessThanOrEquals,
    NotEquals,
    Not,
    And,
    Or,
    Min,
    Max,
    Pow,
    Floor,
    Ceiling,
    Int,
    Float,
    ToStr,
    Has,
    Hasnt,
    FieldRead,
    IndexRead,
    FieldWrite,
    IndexWrite,
    Len,
    ArrayRemove,
    ArrayPush,
    ArrayInsert,
    DictHas,
    DictSize,
    DictRemove,
    DictKeys,
}

impl NativeFunction {
    pub const ALL: [Self; 37] = [
        Self::Add,
        Self::Subtract,
        Self::Divide,
        Self::Multiply,
        Self::Mod,
        Self::Negate,
        Self::Equal,
        Self::Greater,
        Self::Less,
        Self::GreaterThanOrEquals,
        Self::LessThanOrEquals,
        Self::NotEquals,
        Self::Not,
        Self::And,
        Self::Or,
        Self::Min,
        Self::Max,
        Self::Pow,
        Self::Floor,
        Self::Ceiling,
        Self::Int,
        Self::Float,
        Self::ToStr,
        Self::Has,
        Self::Hasnt,
        Self::FieldRead,
        Self::IndexRead,
        Self::FieldWrite,
        Self::IndexWrite,
        Self::Len,
        Self::ArrayRemove,
        Self::ArrayPush,
        Self::ArrayInsert,
        Self::DictHas,
        Self::DictSize,
        Self::DictRemove,
        Self::DictKeys,
    ];

    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "+" => Some(Self::Add),
            "-" => Some(Self::Subtract),
            "/" => Some(Self::Divide),
            "*" => Some(Self::Multiply),
            "%" => Some(Self::Mod),
            "_" => Some(Self::Negate),
            "==" => Some(Self::Equal),
            ">" => Some(Self::Greater),
            "<" => Some(Self::Less),
            ">=" => Some(Self::GreaterThanOrEquals),
            "<=" => Some(Self::LessThanOrEquals),
            "!=" => Some(Self::NotEquals),
            "!" => Some(Self::Not),
            "&&" => Some(Self::And),
            "||" => Some(Self::Or),
            "MIN" => Some(Self::Min),
            "MAX" => Some(Self::Max),
            "POW" => Some(Self::Pow),
            "FLOOR" => Some(Self::Floor),
            "CEILING" => Some(Self::Ceiling),
            "INT" => Some(Self::Int),
            "FLOAT" => Some(Self::Float),
            "to_str" => Some(Self::ToStr),
            "?" => Some(Self::Has),
            "!?" => Some(Self::Hasnt),
            "FIELD" => Some(Self::FieldRead),
            "INDEX" => Some(Self::IndexRead),
            "SET_FIELD" => Some(Self::FieldWrite),
            "SET_INDEX" => Some(Self::IndexWrite),
            "LEN" => Some(Self::Len),
            "ARRAY_REMOVE" => Some(Self::ArrayRemove),
            "ARRAY_PUSH" => Some(Self::ArrayPush),
            "ARRAY_INSERT" => Some(Self::ArrayInsert),
            "DICT_HAS" => Some(Self::DictHas),
            "DICT_SIZE" => Some(Self::DictSize),
            "DICT_REMOVE" => Some(Self::DictRemove),
            "DICT_KEYS" => Some(Self::DictKeys),
            _ => None,
        }
    }

    pub fn token(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Divide => "/",
            Self::Multiply => "*",
            Self::Mod => "%",
            Self::Negate => "_",
            Self::Equal => "==",
            Self::Greater => ">",
            Self::Less => "<",
            Self::GreaterThanOrEquals => ">=",
            Self::LessThanOrEquals => "<=",
            Self::NotEquals => "!=",
            Self::Not => "!",
            Self::And => "&&",
            Self::Or => "||",
            Self::Min => "MIN",
            Self::Max => "MAX",
            Self::Pow => "POW",
            Self::Floor => "FLOOR",
            Self::Ceiling => "CEILING",
            Self::Int => "INT",
            Self::Float => "FLOAT",
            Self::ToStr => "to_str",
            Self::Has => "?",
            Self::Hasnt => "!?",
            Self::FieldRead => "FIELD",
            Self::IndexRead => "INDEX",
            Self::FieldWrite => "SET_FIELD",
            Self::IndexWrite => "SET_INDEX",
            Self::Len => "LEN",
            Self::ArrayRemove => "ARRAY_REMOVE",
            Self::ArrayPush => "ARRAY_PUSH",
            Self::ArrayInsert => "ARRAY_INSERT",
            Self::DictHas => "DICT_HAS",
            Self::DictSize => "DICT_SIZE",
            Self::DictRemove => "DICT_REMOVE",
            Self::DictKeys => "DICT_KEYS",
        }
    }

    pub fn arity(self) -> usize {
        match self {
            Self::Add => 2,
            Self::Subtract => 2,
            Self::Divide => 2,
            Self::Multiply => 2,
            Self::Mod => 2,
            Self::Negate => 1,
            Self::Equal => 2,
            Self::Greater => 2,
            Self::Less => 2,
            Self::GreaterThanOrEquals => 2,
            Self::LessThanOrEquals => 2,
            Self::NotEquals => 2,
            Self::Not => 1,
            Self::And => 2,
            Self::Or => 2,
            Self::Min => 2,
            Self::Max => 2,
            Self::Pow => 2,
            Self::Floor => 1,
            Self::Ceiling => 1,
            Self::Int => 1,
            Self::Float => 1,
            Self::ToStr => 1,
            Self::Has => 2,
            Self::Hasnt => 2,
            Self::FieldRead => 2,
            Self::IndexRead => 2,
            Self::FieldWrite => 3,
            Self::IndexWrite => 3,
            Self::Len => 1,
            Self::ArrayRemove => 2,
            Self::ArrayPush => 2,
            Self::ArrayInsert => 3,
            Self::DictHas => 2,
            Self::DictSize => 1,
            Self::DictRemove => 2,
            Self::DictKeys => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::NativeFunction;

    #[test]
    fn native_function_tokens_roundtrip_and_expose_arity() {
        let cases = [
            (NativeFunction::Add, "+", 2),
            (NativeFunction::Subtract, "-", 2),
            (NativeFunction::Divide, "/", 2),
            (NativeFunction::Multiply, "*", 2),
            (NativeFunction::Mod, "%", 2),
            (NativeFunction::Negate, "_", 1),
            (NativeFunction::Equal, "==", 2),
            (NativeFunction::Greater, ">", 2),
            (NativeFunction::Less, "<", 2),
            (NativeFunction::GreaterThanOrEquals, ">=", 2),
            (NativeFunction::LessThanOrEquals, "<=", 2),
            (NativeFunction::NotEquals, "!=", 2),
            (NativeFunction::Not, "!", 1),
            (NativeFunction::And, "&&", 2),
            (NativeFunction::Or, "||", 2),
            (NativeFunction::Min, "MIN", 2),
            (NativeFunction::Max, "MAX", 2),
            (NativeFunction::Pow, "POW", 2),
            (NativeFunction::Floor, "FLOOR", 1),
            (NativeFunction::Ceiling, "CEILING", 1),
            (NativeFunction::Int, "INT", 1),
            (NativeFunction::Float, "FLOAT", 1),
            (NativeFunction::ToStr, "to_str", 1),
            (NativeFunction::Has, "?", 2),
            (NativeFunction::Hasnt, "!?", 2),
            (NativeFunction::FieldRead, "FIELD", 2),
            (NativeFunction::IndexRead, "INDEX", 2),
            (NativeFunction::FieldWrite, "SET_FIELD", 3),
            (NativeFunction::IndexWrite, "SET_INDEX", 3),
            (NativeFunction::Len, "LEN", 1),
            (NativeFunction::ArrayRemove, "ARRAY_REMOVE", 2),
            (NativeFunction::ArrayPush, "ARRAY_PUSH", 2),
            (NativeFunction::ArrayInsert, "ARRAY_INSERT", 3),
            (NativeFunction::DictHas, "DICT_HAS", 2),
            (NativeFunction::DictSize, "DICT_SIZE", 1),
            (NativeFunction::DictRemove, "DICT_REMOVE", 2),
            (NativeFunction::DictKeys, "DICT_KEYS", 1),
        ];

        assert_eq!(NativeFunction::ALL.len(), cases.len());

        let mut tokens = BTreeSet::new();
        for (function, token, arity) in cases {
            assert_eq!(NativeFunction::from_token(token), Some(function));
            assert_eq!(function.token(), token);
            assert_eq!(function.arity(), arity);
            assert!(NativeFunction::ALL.contains(&function));
            assert!(tokens.insert(token));
        }
    }

    #[test]
    fn native_function_from_token_rejects_unknown_tokens() {
        for token in ["UNKNOWN_NATIVE", "done", "void", "^text", "FIELD?"] {
            assert_eq!(NativeFunction::from_token(token), None);
        }
    }
}
