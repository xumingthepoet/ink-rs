use super::push_indent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalDeclaration {
    name: String,
    argument_names: Vec<String>,
}

impl ExternalDeclaration {
    pub fn new(name: impl Into<String>, argument_names: Vec<String>) -> Self {
        Self {
            name: name.into(),
            argument_names,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn argument_names(&self) -> &[String] {
        &self.argument_names
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
