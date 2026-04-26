use super::{push_indent, TypeName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalDeclaration {
    name: String,
    argument_names: Vec<String>,
    argument_types: Vec<TypeName>,
    return_type: TypeName,
}

impl ExternalDeclaration {
    pub fn with_signature(
        name: impl Into<String>,
        arguments: Vec<(String, TypeName)>,
        return_type: TypeName,
    ) -> Self {
        let (argument_names, argument_types) = arguments.into_iter().unzip();
        Self {
            name: name.into(),
            argument_names,
            argument_types,
            return_type,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn argument_names(&self) -> &[String] {
        &self.argument_names
    }

    pub fn argument_types(&self) -> &[TypeName] {
        &self.argument_types
    }

    pub fn return_type(&self) -> &TypeName {
        &self.return_type
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("ExternalDeclaration(name=\"");
        out.push_str(&self.name);
        out.push_str("\", args=[");
        for (index, arg) in self.argument_names.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            out.push('"');
            out.push_str(arg);
            out.push('"');
        }
        out.push_str("])");
    }
}
