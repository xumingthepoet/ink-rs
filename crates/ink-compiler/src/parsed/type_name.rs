use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeName {
    Primitive(PrimitiveType),
    Struct(String),
    Void,
    Array(Box<TypeName>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DefaultValue {
    Int(i32),
    Float(f64),
    Bool(bool),
    String(String),
    Array { element_type: Box<TypeName> },
    Struct { type_name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Int,
    Float,
    Bool,
    String,
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

    pub fn void() -> Self {
        Self::Void
    }

    pub fn struct_type(name: impl Into<String>) -> Self {
        Self::Struct(name.into())
    }

    pub fn array(element_type: TypeName) -> Self {
        Self::Array(Box::new(element_type))
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

    pub fn as_struct_name(&self) -> Option<&str> {
        match self {
            Self::Struct(name) => Some(name),
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
            Self::Void => formatter.write_str("void"),
            Self::Array(element_type) => write!(formatter, "{element_type}[]"),
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
            TypeName::Array(element_type) => Some(Self::Array {
                element_type: element_type.clone(),
            }),
            TypeName::Struct(type_name) => Some(Self::Struct {
                type_name: type_name.clone(),
            }),
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
            Self::Struct { type_name } => TypeName::struct_type(type_name.clone()),
        }
    }

    pub fn is_abstract_struct(&self) -> bool {
        matches!(self, Self::Struct { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::{DefaultValue, PrimitiveType, TypeName};

    #[test]
    fn displays_primitive_and_void_type_names() {
        let cases = [
            (TypeName::int(), "int"),
            (TypeName::float(), "float"),
            (TypeName::bool(), "bool"),
            (TypeName::string(), "string"),
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
}
