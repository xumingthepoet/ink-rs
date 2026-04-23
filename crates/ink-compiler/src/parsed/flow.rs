use super::{push_indent, Weave};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowLevel {
    Knot,
    Stitch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowArgument {
    name: String,
    is_by_reference: bool,
    is_divert_target: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flow {
    level: FlowLevel,
    name: String,
    weave: Weave,
    arguments: Vec<FlowArgument>,
    is_function: bool,
}

impl FlowArgument {
    pub fn new(name: impl Into<String>, is_by_reference: bool, is_divert_target: bool) -> Self {
        Self {
            name: name.into(),
            is_by_reference,
            is_divert_target,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_by_reference(&self) -> bool {
        self.is_by_reference
    }

    pub fn is_divert_target(&self) -> bool {
        self.is_divert_target
    }
}

impl Flow {
    pub fn new(
        level: FlowLevel,
        name: impl Into<String>,
        content: Vec<super::Object>,
        arguments: Vec<FlowArgument>,
        is_function: bool,
    ) -> Self {
        Self {
            level,
            name: name.into(),
            weave: Weave::new(content, 0),
            arguments,
            is_function,
        }
    }

    pub fn level(&self) -> FlowLevel {
        self.level
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn weave(&self) -> &Weave {
        &self.weave
    }

    pub fn arguments(&self) -> &[FlowArgument] {
        &self.arguments
    }

    pub fn is_function(&self) -> bool {
        self.is_function
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("Flow(level=");
        out.push_str(match self.level {
            FlowLevel::Knot => "Knot",
            FlowLevel::Stitch => "Stitch",
        });
        out.push_str(", name=\"");
        out.push_str(&self.name);
        out.push_str("\", function=");
        out.push_str(if self.is_function { "true" } else { "false" });
        out.push(')');
        self.weave.write_parse_snapshot(out, indent + 2);
    }
}
