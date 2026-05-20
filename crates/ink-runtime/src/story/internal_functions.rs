use std::{collections::HashMap, rc::Rc};

use ink_story_json_format as format;

use crate::{
    container::Container,
    path::Path,
    story::Story,
    story_error::StoryError,
    value_type::{DictKeyType, ValueType},
};

#[derive(Debug, Clone)]
pub(crate) struct InternalFunctionDef {
    path: String,
    arg_types: Vec<RuntimeType>,
    return_type: RuntimeType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RuntimeType {
    Bool,
    Int,
    Float,
    String,
    DivertTarget,
    Void,
    Array(Box<RuntimeType>),
    Dict {
        key_type: DictKeyType,
        value_type: Box<RuntimeType>,
    },
    Object(String),
}

pub(crate) fn load_internal_function_defs(
    functions: std::collections::BTreeMap<String, format::InternalFunction>,
) -> Result<HashMap<String, InternalFunctionDef>, StoryError> {
    functions
        .into_iter()
        .map(|(name, function)| {
            let arg_types = function
                .arg_types
                .iter()
                .map(|type_name| parse_runtime_type(type_name))
                .collect::<Result<Vec<_>, _>>()?;
            let return_type = parse_runtime_type(&function.return_type)?;
            Ok((
                name,
                InternalFunctionDef {
                    path: function.path,
                    arg_types,
                    return_type,
                },
            ))
        })
        .collect()
}

impl Story {
    /// Call an ink `INTERNAL` function from host code.
    pub fn call_internal(
        &mut self,
        func_name: &str,
        args: Option<&Vec<ValueType>>,
    ) -> Result<Option<ValueType>, StoryError> {
        self.if_async_we_cant("call an INTERNAL function")?;

        let function = self
            .internal_functions
            .get(func_name)
            .cloned()
            .ok_or_else(|| {
                StoryError::BadArgument(format!(
                    "INTERNAL function '{func_name}' has not been declared."
                ))
            })?;

        if function.uses_divert_target() {
            return Err(StoryError::BadArgument(format!(
                "INTERNAL function '{func_name}' uses divert target types, which host calls do not support."
            )));
        }

        let empty_args = Vec::new();
        let args = args.unwrap_or(&empty_args);
        validate_arguments(func_name, args, &function.arg_types)?;

        let container = self.internal_function_container(func_name, &function.path)?;
        let mut text_output = String::new();
        let result =
            self.evaluate_function_container(func_name, container, Some(args), &mut text_output)?;

        if !text_output.is_empty() {
            return Err(StoryError::InvalidStoryState(format!(
                "INTERNAL function '{func_name}' produced text output; use ~ return for host-callable values."
            )));
        }

        validate_return_value(func_name, result, &function.return_type)
    }

    fn internal_function_container(
        &self,
        func_name: &str,
        path: &str,
    ) -> Result<Rc<Container>, StoryError> {
        let path = Path::new_with_components_string(Some(path));
        let result = self.content_at_path(&path);
        if result.approximate {
            return Err(StoryError::BadJson(format!(
                "INTERNAL function '{func_name}' points to missing path '{path}'."
            )));
        }
        result.container().ok_or_else(|| {
            StoryError::BadJson(format!(
                "INTERNAL function '{func_name}' path '{path}' is not a container."
            ))
        })
    }
}

impl InternalFunctionDef {
    fn uses_divert_target(&self) -> bool {
        self.return_type.contains_divert_target()
            || self
                .arg_types
                .iter()
                .any(RuntimeType::contains_divert_target)
    }
}

impl RuntimeType {
    fn contains_divert_target(&self) -> bool {
        match self {
            Self::DivertTarget => true,
            Self::Array(element) => element.contains_divert_target(),
            Self::Dict { value_type, .. } => value_type.contains_divert_target(),
            _ => false,
        }
    }

    fn display_name(&self) -> String {
        match self {
            Self::Bool => "bool".to_string(),
            Self::Int => "int".to_string(),
            Self::Float => "float".to_string(),
            Self::String => "string".to_string(),
            Self::DivertTarget => "->".to_string(),
            Self::Void => "void".to_string(),
            Self::Array(element) => format!("{}[]", element.display_name()),
            Self::Dict {
                key_type,
                value_type,
            } => {
                format!("Dict<{key_type}, {}>", value_type.display_name())
            }
            Self::Object(name) => name.clone(),
        }
    }
}

fn parse_runtime_type(type_name: &str) -> Result<RuntimeType, StoryError> {
    let mut base = type_name;
    let mut array_depth = 0;
    while let Some(stripped) = base.strip_suffix("[]") {
        array_depth += 1;
        base = stripped;
    }

    let mut parsed = if let Some(dict_type) = parse_dict_runtime_type(base)? {
        dict_type
    } else {
        match base {
            "bool" => RuntimeType::Bool,
            "int" => RuntimeType::Int,
            "float" => RuntimeType::Float,
            "string" => RuntimeType::String,
            "->" => RuntimeType::DivertTarget,
            "void" => RuntimeType::Void,
            name if !name.is_empty() => RuntimeType::Object(name.to_string()),
            _ => {
                return Err(StoryError::BadJson(format!(
                    "Invalid INTERNAL function type '{type_name}'."
                )))
            }
        }
    };

    for _ in 0..array_depth {
        parsed = RuntimeType::Array(Box::new(parsed));
    }

    Ok(parsed)
}

fn parse_dict_runtime_type(type_name: &str) -> Result<Option<RuntimeType>, StoryError> {
    if !type_name.starts_with("Dict<") {
        return Ok(None);
    };
    let Some(inner) = type_name
        .strip_prefix("Dict<")
        .and_then(|value| value.strip_suffix('>'))
    else {
        return Err(StoryError::BadJson(format!(
            "Invalid INTERNAL function type '{type_name}'."
        )));
    };

    let comma = top_level_comma(inner).ok_or_else(|| {
        StoryError::BadJson(format!("Invalid INTERNAL function type '{type_name}'."))
    })?;
    let key_type = match inner[..comma].trim() {
        "string" => DictKeyType::String,
        "int" => DictKeyType::Int,
        _ => {
            return Err(StoryError::BadJson(format!(
                "Invalid INTERNAL function Dict key type '{type_name}'."
            )))
        }
    };
    let value_type_text = inner[comma + 1..].trim();
    if value_type_text.is_empty() {
        return Err(StoryError::BadJson(format!(
            "Invalid INTERNAL function type '{type_name}'."
        )));
    }

    Ok(Some(RuntimeType::Dict {
        key_type,
        value_type: Box::new(parse_runtime_type(value_type_text)?),
    }))
}

fn top_level_comma(text: &str) -> Option<usize> {
    let mut angle_depth = 0_usize;
    let mut comma = None;
    for (index, character) in text.char_indices() {
        match character {
            '<' => angle_depth += 1,
            '>' => angle_depth = angle_depth.checked_sub(1)?,
            ',' if angle_depth == 0 => {
                if comma.replace(index).is_some() {
                    return None;
                }
            }
            _ => {}
        }
    }
    (angle_depth == 0).then_some(comma).flatten()
}

fn validate_arguments(
    func_name: &str,
    args: &[ValueType],
    expected_types: &[RuntimeType],
) -> Result<(), StoryError> {
    if args.len() != expected_types.len() {
        return Err(StoryError::BadArgument(format!(
            "INTERNAL function '{func_name}' expected {} argument(s), got {}.",
            expected_types.len(),
            args.len()
        )));
    }

    for (index, (value, expected_type)) in args.iter().zip(expected_types).enumerate() {
        if !value_matches_type(value, expected_type) {
            return Err(StoryError::BadArgument(format!(
                "INTERNAL function '{func_name}' argument {} expected {}, got {}.",
                index + 1,
                expected_type.display_name(),
                value_type_name(value)
            )));
        }
    }

    Ok(())
}

fn validate_return_value(
    func_name: &str,
    result: Option<ValueType>,
    expected_type: &RuntimeType,
) -> Result<Option<ValueType>, StoryError> {
    match (result, expected_type) {
        (None, RuntimeType::Void) => Ok(None),
        (None, expected) => Err(StoryError::InvalidStoryState(format!(
            "INTERNAL function '{func_name}' expected return type {}, got void.",
            expected.display_name()
        ))),
        (Some(value), RuntimeType::Void) => Err(StoryError::InvalidStoryState(format!(
            "INTERNAL function '{func_name}' expected return type void, got {}.",
            value_type_name(&value)
        ))),
        (Some(value), expected) if value_matches_type(&value, expected) => Ok(Some(value)),
        (Some(value), expected) => Err(StoryError::InvalidStoryState(format!(
            "INTERNAL function '{func_name}' expected return type {}, got {}.",
            expected.display_name(),
            value_type_name(&value)
        ))),
    }
}

fn value_matches_type(value: &ValueType, expected_type: &RuntimeType) -> bool {
    match (value, expected_type) {
        (ValueType::Bool(_), RuntimeType::Bool)
        | (ValueType::Int(_), RuntimeType::Int)
        | (ValueType::Float(_), RuntimeType::Float)
        | (ValueType::String(_), RuntimeType::String) => true,
        (ValueType::Array(values), RuntimeType::Array(element_type)) => values
            .iter()
            .all(|value| value_matches_type(value, element_type)),
        (
            ValueType::Dict(dict),
            RuntimeType::Dict {
                key_type,
                value_type,
            },
        ) => {
            dict.key_type() == *key_type
                && dict
                    .entries()
                    .values()
                    .all(|value| value_matches_type(value, value_type))
        }
        (ValueType::Object(_), RuntimeType::Object(_)) => true,
        _ => false,
    }
}

fn value_type_name(value: &ValueType) -> &'static str {
    match value {
        ValueType::Bool(_) => "bool",
        ValueType::Int(_) => "int",
        ValueType::Float(_) => "float",
        ValueType::String(_) => "string",
        ValueType::DivertTarget(_) => "->",
        ValueType::VariablePointer(_) => "variable pointer",
        ValueType::Array(_) => "array",
        ValueType::Object(_) => "object",
        ValueType::Dict(_) => "dict",
    }
}
