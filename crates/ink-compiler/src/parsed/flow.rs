use crate::source::SourceSpan;

use super::{push_indent, Object, TypeName, Weave};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowLevel {
    Knot,
    Stitch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowArgument {
    name: String,
    declared_type: Option<TypeName>,
    is_by_reference: bool,
    is_divert_target: bool,
    span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flow {
    level: FlowLevel,
    name: String,
    weave: Weave,
    child_flows: Vec<Flow>,
    arguments: Vec<FlowArgument>,
    return_type: TypeName,
    is_function: bool,
    span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowParts {
    level: FlowLevel,
    name: String,
    content: Vec<Object>,
    child_flows: Vec<Flow>,
    arguments: Vec<FlowArgument>,
    return_type: TypeName,
    is_function: bool,
    span: SourceSpan,
}

impl FlowArgument {
    pub fn new(
        name: impl Into<String>,
        declared_type: Option<TypeName>,
        is_by_reference: bool,
        is_divert_target: bool,
        span: SourceSpan,
    ) -> Self {
        Self {
            name: name.into(),
            declared_type,
            is_by_reference,
            is_divert_target,
            span,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn declared_type(&self) -> Option<&TypeName> {
        self.declared_type.as_ref()
    }

    pub fn is_by_reference(&self) -> bool {
        self.is_by_reference
    }

    pub fn is_divert_target(&self) -> bool {
        self.is_divert_target
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }
}

impl FlowParts {
    pub fn new(level: FlowLevel, name: impl Into<String>, content: Vec<Object>) -> Self {
        Self {
            level,
            name: name.into(),
            content,
            child_flows: Vec::new(),
            arguments: Vec::new(),
            return_type: TypeName::void(),
            is_function: false,
            span: SourceSpan::new(None, 1, 1),
        }
    }

    pub fn child_flows(mut self, child_flows: Vec<Flow>) -> Self {
        self.child_flows = child_flows;
        self
    }

    pub fn arguments(mut self, arguments: Vec<FlowArgument>) -> Self {
        self.arguments = arguments;
        self
    }

    pub fn return_type(mut self, return_type: TypeName) -> Self {
        self.return_type = return_type;
        self
    }

    pub fn function(mut self, is_function: bool) -> Self {
        self.is_function = is_function;
        self
    }

    pub fn span(mut self, span: SourceSpan) -> Self {
        self.span = span;
        self
    }
}

impl Flow {
    pub fn from_parts(parts: FlowParts) -> Self {
        Self {
            level: parts.level,
            name: parts.name,
            weave: Weave::new(parts.content, 0),
            child_flows: parts.child_flows,
            arguments: parts.arguments,
            return_type: parts.return_type,
            is_function: parts.is_function,
            span: parts.span,
        }
    }

    pub fn level(&self) -> FlowLevel {
        self.level
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub fn weave(&self) -> &Weave {
        &self.weave
    }

    pub fn child_flows(&self) -> &[Flow] {
        &self.child_flows
    }

    pub fn arguments(&self) -> &[FlowArgument] {
        &self.arguments
    }

    pub fn return_type(&self) -> &TypeName {
        &self.return_type
    }

    pub fn has_typed_signature(&self) -> bool {
        self.is_function
            || self
                .arguments
                .iter()
                .any(|argument| argument.declared_type().is_some())
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
        if !self.weave.content().is_empty() {
            self.weave.write_parse_snapshot(out, indent + 2);
        }
        for child in &self.child_flows {
            child.write_parse_snapshot(out, indent + 2);
        }
    }
}
