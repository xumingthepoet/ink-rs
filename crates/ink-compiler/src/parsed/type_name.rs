use std::{
    fmt,
    hash::{Hash, Hasher},
};

use crate::source::SourceSpan;

use super::QualifiedName;

#[derive(Debug, Clone)]
pub enum TypeName {
    Primitive(PrimitiveType),
    Struct(String),
    QualifiedStruct(QualifiedName),
    Interface {
        name: String,
        name_span: SourceSpan,
        span: SourceSpan,
    },
    Void,
    Array(Box<TypeName>),
    Dict {
        key_type: PrimitiveType,
        value_type: Box<TypeName>,
    },
}

impl PartialEq for TypeName {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Primitive(left), Self::Primitive(right)) => left == right,
            (Self::Struct(left), Self::Struct(right)) => left == right,
            (Self::QualifiedStruct(left), Self::QualifiedStruct(right)) => left == right,
            (Self::Interface { name: left, .. }, Self::Interface { name: right, .. }) => {
                left == right
            }
            (Self::Void, Self::Void) => true,
            (Self::Array(left), Self::Array(right)) => left == right,
            (
                Self::Dict {
                    key_type: left_key,
                    value_type: left_value,
                },
                Self::Dict {
                    key_type: right_key,
                    value_type: right_value,
                },
            ) => left_key == right_key && left_value == right_value,
            _ => false,
        }
    }
}

impl Eq for TypeName {}

impl Hash for TypeName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Primitive(primitive) => {
                0_u8.hash(state);
                primitive.hash(state);
            }
            Self::Struct(name) => {
                1_u8.hash(state);
                name.hash(state);
            }
            Self::QualifiedStruct(name) => {
                2_u8.hash(state);
                name.hash(state);
            }
            Self::Interface { name, .. } => {
                3_u8.hash(state);
                name.hash(state);
            }
            Self::Void => {
                4_u8.hash(state);
            }
            Self::Array(element_type) => {
                5_u8.hash(state);
                element_type.hash(state);
            }
            Self::Dict {
                key_type,
                value_type,
            } => {
                6_u8.hash(state);
                key_type.hash(state);
                value_type.hash(state);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DefaultValue {
    Int(i32),
    Float(f64),
    Bool(bool),
    String(String),
    Array {
        element_type: Box<TypeName>,
    },
    Dict {
        key_type: PrimitiveType,
        value_type: Box<TypeName>,
    },
    Struct {
        type_name: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Int,
    Float,
    Bool,
    String,
    DivertTarget,
}

impl TypeName {
    pub fn int() -> Self {
        Self::Primitive(PrimitiveType::Int)
    }

    pub fn float() -> Self {
        Self::Primitive(PrimitiveType::Float)
    }

    pub fn bool() -> Self {
        Self::Primitive(PrimitiveType::Bool)
    }

    pub fn string() -> Self {
        Self::Primitive(PrimitiveType::String)
    }

    pub fn divert_target() -> Self {
        Self::Primitive(PrimitiveType::DivertTarget)
    }

    pub fn void() -> Self {
        Self::Void
    }

    pub fn struct_type(name: impl Into<String>) -> Self {
        Self::Struct(name.into())
    }

    pub fn qualified_struct_type(name: QualifiedName) -> Self {
        Self::QualifiedStruct(name)
    }

    pub fn interface_type(
        name: impl Into<String>,
        name_span: SourceSpan,
        span: SourceSpan,
    ) -> Self {
        Self::Interface {
            name: name.into(),
            name_span,
            span,
        }
    }

    pub fn array(element_type: TypeName) -> Self {
        Self::Array(Box::new(element_type))
    }

    pub fn dict(key_type: PrimitiveType, value_type: TypeName) -> Self {
        Self::Dict {
            key_type,
            value_type: Box::new(value_type),
        }
    }

    pub fn display_name(&self) -> String {
        self.to_string()
    }

    pub fn snapshot_name(&self) -> String {
        self.to_string()
    }

    pub fn array_element_type(&self) -> Option<&TypeName> {
        match self {
            Self::Array(element_type) => Some(element_type),
            _ => None,
        }
    }

    pub fn dict_key_value_types(&self) -> Option<(PrimitiveType, &TypeName)> {
        match self {
            Self::Dict {
                key_type,
                value_type,
            } => Some((*key_type, value_type)),
            _ => None,
        }
    }

    pub fn as_struct_name(&self) -> Option<&str> {
        match self {
            Self::Struct(name) => Some(name),
            Self::QualifiedStruct(name) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn as_interface_name(&self) -> Option<&str> {
        match self {
            Self::Interface { name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn interface_name_span(&self) -> Option<&SourceSpan> {
        match self {
            Self::Interface { name_span, .. } => Some(name_span),
            _ => None,
        }
    }

    pub fn interface_span(&self) -> Option<&SourceSpan> {
        match self {
            Self::Interface { span, .. } => Some(span),
            _ => None,
        }
    }

    pub fn primitive_type(&self) -> Option<PrimitiveType> {
        match self {
            Self::Primitive(primitive) => Some(*primitive),
            _ => None,
        }
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void)
    }

    pub fn default_value(&self) -> Option<DefaultValue> {
        DefaultValue::for_type(self)
    }
}

impl fmt::Display for TypeName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitive(primitive) => primitive.fmt(formatter),
            Self::Struct(name) => formatter.write_str(name),
            Self::QualifiedStruct(name) => formatter.write_str(name.as_str()),
            Self::Interface { name, .. } => write!(formatter, "interface<{name}>"),
            Self::Void => formatter.write_str("void"),
            Self::Array(element_type) => write!(formatter, "{element_type}[]"),
            Self::Dict {
                key_type,
                value_type,
            } => write!(formatter, "Dict<{key_type}, {value_type}>"),
        }
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Int => "int",
            Self::Float => "float",
            Self::Bool => "bool",
            Self::String => "string",
            Self::DivertTarget => "->",
        })
    }
}

impl DefaultValue {
    pub fn for_type(type_name: &TypeName) -> Option<Self> {
        match type_name {
            TypeName::Primitive(PrimitiveType::Int) => Some(Self::Int(0)),
            TypeName::Primitive(PrimitiveType::Float) => Some(Self::Float(0.0)),
            TypeName::Primitive(PrimitiveType::Bool) => Some(Self::Bool(false)),
            TypeName::Primitive(PrimitiveType::String) => Some(Self::String(String::new())),
            TypeName::Primitive(PrimitiveType::DivertTarget) => None,
            TypeName::Array(element_type) => Some(Self::Array {
                element_type: element_type.clone(),
            }),
            TypeName::Dict {
                key_type,
                value_type,
            } => Some(Self::Dict {
                key_type: *key_type,
                value_type: value_type.clone(),
            }),
            TypeName::Struct(type_name) => Some(Self::Struct {
                type_name: type_name.clone(),
            }),
            TypeName::QualifiedStruct(type_name) => Some(Self::Struct {
                type_name: type_name.as_str().to_string(),
            }),
            TypeName::Interface { .. } => None,
            TypeName::Void => None,
        }
    }

    pub fn type_name(&self) -> TypeName {
        match self {
            Self::Int(_) => TypeName::int(),
            Self::Float(_) => TypeName::float(),
            Self::Bool(_) => TypeName::bool(),
            Self::String(_) => TypeName::string(),
            Self::Array { element_type } => TypeName::array((**element_type).clone()),
            Self::Dict {
                key_type,
                value_type,
            } => TypeName::dict(*key_type, (**value_type).clone()),
            Self::Struct { type_name } => TypeName::struct_type(type_name.clone()),
        }
    }

    pub fn is_abstract_struct(&self) -> bool {
        matches!(self, Self::Struct { .. })
    }
}

#[cfg(test)]
mod tests {
    use crate::source::SourceSpan;

    use super::{DefaultValue, PrimitiveType, TypeName};

    #[test]
    fn displays_primitive_and_void_type_names() {
        let cases = [
            (TypeName::int(), "int"),
            (TypeName::float(), "float"),
            (TypeName::bool(), "bool"),
            (TypeName::string(), "string"),
            (TypeName::divert_target(), "->"),
            (TypeName::void(), "void"),
        ];

        for (type_name, expected) in cases {
            assert_eq!(type_name.display_name(), expected);
            assert_eq!(type_name.snapshot_name(), expected);
            assert_eq!(type_name.to_string(), expected);
        }
    }

    #[test]
    fn displays_struct_and_array_type_names() {
        let player = TypeName::struct_type("Player");
        let player_array = TypeName::array(player.clone());
        let nested_int_array = TypeName::array(TypeName::array(TypeName::int()));

        assert_eq!(player.display_name(), "Player");
        assert_eq!(player_array.display_name(), "Player[]");
        assert_eq!(nested_int_array.display_name(), "int[][]");
        assert_eq!(nested_int_array.snapshot_name(), "int[][]");
    }

    #[test]
    fn displays_dict_type_names() {
        let scores = TypeName::dict(PrimitiveType::String, TypeName::int());
        let nested = TypeName::dict(
            PrimitiveType::Int,
            TypeName::array(TypeName::dict(PrimitiveType::String, TypeName::bool())),
        );

        assert_eq!(scores.display_name(), "Dict<string, int>");
        assert_eq!(scores.snapshot_name(), "Dict<string, int>");
        assert_eq!(scores.to_string(), "Dict<string, int>");
        assert_eq!(nested.display_name(), "Dict<int, Dict<string, bool>[]>");
        assert_eq!(
            scores.dict_key_value_types(),
            Some((PrimitiveType::String, &TypeName::int()))
        );
    }

    #[test]
    fn displays_interface_type_names() {
        let interface_type = TypeName::interface_type("IItem", span_at(1, 11), span_at(1, 1));
        let interface_array = TypeName::array(interface_type.clone());

        assert_eq!(interface_type.display_name(), "interface<IItem>");
        assert_eq!(interface_type.snapshot_name(), "interface<IItem>");
        assert_eq!(interface_type.to_string(), "interface<IItem>");
        assert_eq!(interface_array.display_name(), "interface<IItem>[]");
        assert_eq!(interface_type.as_interface_name(), Some("IItem"));
        assert_eq!(interface_type.interface_name_span(), Some(&span_at(1, 11)));
        assert_eq!(interface_type.interface_span(), Some(&span_at(1, 1)));
    }

    #[test]
    fn interface_type_equality_ignores_source_spans() {
        let first = TypeName::interface_type("IItem", span_at(1, 11), span_at(1, 1));
        let second = TypeName::interface_type("IItem", span_at(20, 5), span_at(20, 1));

        assert_eq!(first, second);
        assert_ne!(
            first,
            TypeName::interface_type("IOther", span_at(1, 11), span_at(1, 1))
        );
    }

    #[test]
    fn exposes_type_shape_helpers() {
        let nested_array = TypeName::array(TypeName::array(TypeName::int()));
        let element = nested_array
            .array_element_type()
            .expect("array should expose an element type");

        assert_eq!(element.display_name(), "int[]");
        assert_eq!(element.array_element_type(), Some(&TypeName::int()));
        assert_eq!(
            TypeName::struct_type("Player").as_struct_name(),
            Some("Player")
        );
        assert_eq!(TypeName::int().primitive_type(), Some(PrimitiveType::Int));
        assert_eq!(
            TypeName::divert_target().primitive_type(),
            Some(PrimitiveType::DivertTarget)
        );
        assert_eq!(
            TypeName::dict(PrimitiveType::Int, TypeName::string()).dict_key_value_types(),
            Some((PrimitiveType::Int, &TypeName::string()))
        );
        assert!(TypeName::void().is_void());
        assert!(!TypeName::string().is_void());
    }

    #[test]
    fn builds_primitive_default_values() {
        let cases = [
            (TypeName::int(), DefaultValue::Int(0)),
            (TypeName::float(), DefaultValue::Float(0.0)),
            (TypeName::bool(), DefaultValue::Bool(false)),
            (TypeName::string(), DefaultValue::String(String::new())),
        ];

        for (type_name, expected) in cases {
            let default_value = type_name
                .default_value()
                .expect("primitive types have defaults");
            assert_eq!(default_value, expected);
            assert_eq!(default_value.type_name(), type_name);
        }

        assert_eq!(TypeName::void().default_value(), None);
        assert_eq!(TypeName::divert_target().default_value(), None);
        assert_eq!(
            TypeName::interface_type("IItem", span_at(1, 11), span_at(1, 1)).default_value(),
            None
        );
    }

    #[test]
    fn builds_array_default_value_metadata() {
        let int_array_type = TypeName::array(TypeName::int());
        let nested_array_type = TypeName::array(int_array_type.clone());

        assert_eq!(
            int_array_type.default_value(),
            Some(DefaultValue::Array {
                element_type: Box::new(TypeName::int())
            })
        );
        assert_eq!(
            nested_array_type.default_value(),
            Some(DefaultValue::Array {
                element_type: Box::new(int_array_type)
            })
        );
    }

    #[test]
    fn builds_dict_default_value_metadata() {
        let dict_type = TypeName::dict(PrimitiveType::String, TypeName::array(TypeName::int()));

        assert_eq!(
            dict_type.default_value(),
            Some(DefaultValue::Dict {
                key_type: PrimitiveType::String,
                value_type: Box::new(TypeName::array(TypeName::int()))
            })
        );
        assert_eq!(
            dict_type.default_value().map(|value| value.type_name()),
            Some(dict_type)
        );
    }

    #[test]
    fn keeps_struct_defaults_abstract_until_struct_declarations_exist() {
        let default_value = TypeName::struct_type("Player")
            .default_value()
            .expect("struct type has an abstract default");

        assert_eq!(
            default_value,
            DefaultValue::Struct {
                type_name: "Player".to_string()
            }
        );
        assert_eq!(default_value.type_name(), TypeName::struct_type("Player"));
        assert!(default_value.is_abstract_struct());
    }

    fn span_at(line: usize, column: usize) -> SourceSpan {
        SourceSpan::new(Some("types.ink".to_string()), line, column)
    }
}
