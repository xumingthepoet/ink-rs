use serde_json::{Map, Value as JsonValue};

use crate::{Container, FormatError, NamedContainer};

use super::object::{object_from_value, object_to_value};
use super::scalar::json_value_to_string;

pub(crate) fn container_from_value(
    value: &JsonValue,
    name_hint: Option<String>,
) -> Result<Container, FormatError> {
    let array = value
        .as_array()
        .ok_or_else(|| FormatError::new("container must be a JSON array"))?;
    let Some((terminator, content_values)) = array.split_last() else {
        return Err(FormatError::new(
            "container array must include a terminator",
        ));
    };

    let mut content = Vec::with_capacity(content_values.len());
    for value in content_values {
        content.push(object_from_value(value)?);
    }

    let mut name = name_hint;
    let mut named_content = Vec::new();

    match terminator {
        JsonValue::Null => {}
        JsonValue::Object(obj) if obj.is_empty() => {
            return Err(FormatError::new(
                "container terminator object must contain metadata or named content",
            ));
        }
        JsonValue::Object(obj) => {
            for (key, value) in obj {
                match key.as_str() {
                    "#f" => {
                        return Err(FormatError::new(
                            "container count flags '#f' are not supported in current compiled JSON",
                        ));
                    }
                    "#n" => name = Some(json_value_to_string(value, "#n")?.to_string()),
                    name => {
                        let container = container_from_value(value, Some(name.to_string()))?;
                        named_content.push(NamedContainer::new(name, container));
                    }
                }
            }
        }
        _ => {
            return Err(FormatError::new(
                "container terminator must be null or an object",
            ));
        }
    }

    Ok(Container {
        content,
        named_content,
        name,
    })
}

pub(crate) fn container_to_value(container: &Container, include_name: bool) -> JsonValue {
    let mut values = container
        .content
        .iter()
        .map(object_to_value)
        .collect::<Vec<_>>();
    values.push(container_terminator_to_value(container, include_name));
    JsonValue::Array(values)
}

fn container_terminator_to_value(container: &Container, include_name: bool) -> JsonValue {
    let mut obj = Map::new();

    for named in &container.named_content {
        obj.insert(
            named.name.clone(),
            container_to_value(&named.container, false),
        );
    }

    if include_name {
        if let Some(name) = &container.name {
            obj.insert("#n".to_string(), JsonValue::String(name.clone()));
        }
    }

    if obj.is_empty() {
        JsonValue::Null
    } else {
        JsonValue::Object(obj)
    }
}
