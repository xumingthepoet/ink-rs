use crate::{
    parsed::{QualifiedName, TypeName},
    source::SourceSpan,
};

pub(super) fn qualify_type_name_for_module(type_name: &TypeName, module: &str) -> TypeName {
    match type_name {
        TypeName::Struct(name) => {
            TypeName::qualified_struct_type(qualified_type_name(module, name))
        }
        TypeName::Array(element_type) => {
            TypeName::array(qualify_type_name_for_module(element_type, module))
        }
        TypeName::Dict {
            key_type,
            value_type,
        } => TypeName::dict(*key_type, qualify_type_name_for_module(value_type, module)),
        TypeName::Primitive(_)
        | TypeName::QualifiedStruct(_)
        | TypeName::Interface { .. }
        | TypeName::Void => type_name.clone(),
    }
}

pub(super) fn type_name_module(type_name: &TypeName) -> Option<&str> {
    match type_name {
        TypeName::QualifiedStruct(name) => Some(name.module()),
        TypeName::Primitive(_)
        | TypeName::Struct(_)
        | TypeName::Interface { .. }
        | TypeName::Void
        | TypeName::Dict { .. }
        | TypeName::Array(_) => None,
    }
}

fn qualified_type_name(module: &str, name: &str) -> QualifiedName {
    let span = SourceSpan::new(None, 1, 1);
    QualifiedName::new(module, span.clone(), name, span)
}
