use crate::source::SourceSpan;

use super::{push_indent, Expression, Weave};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForLoopVariable {
    source_name: String,
    runtime_name: String,
}

impl ForLoopVariable {
    pub fn new(source_name: impl Into<String>, runtime_name: impl Into<String>) -> Self {
        Self {
            source_name: source_name.into(),
            runtime_name: runtime_name.into(),
        }
    }

    pub fn source_name(&self) -> &str {
        &self.source_name
    }

    pub fn runtime_name(&self) -> &str {
        &self.runtime_name
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForLoop {
    id: usize,
    variables: Vec<ForLoopVariable>,
    iterable: Expression,
    body: Weave,
    span: SourceSpan,
}

impl ForLoop {
    pub fn new(
        id: usize,
        variables: Vec<ForLoopVariable>,
        iterable: Expression,
        body: Weave,
        span: SourceSpan,
    ) -> Self {
        Self {
            id,
            variables,
            iterable,
            body,
            span,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn variables(&self) -> &[ForLoopVariable] {
        &self.variables
    }

    pub fn iterable(&self) -> &Expression {
        &self.iterable
    }

    pub fn body(&self) -> &Weave {
        &self.body
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub fn index_name(&self) -> String {
        format!("$for{}_index", self.id)
    }

    pub fn limit_name(&self) -> String {
        format!("$for{}_limit", self.id)
    }

    pub fn keys_name(&self) -> String {
        format!("$for{}_keys", self.id)
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        push_indent(out, indent);
        out.push_str("ForLoop(vars=");
        for (index, variable) in self.variables.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str(variable.source_name());
        }
        out.push_str(")");
        out.push('\n');
        self.iterable.write_parse_snapshot(out, indent + 2);
        self.body.write_parse_snapshot(out, indent + 2);
    }
}
