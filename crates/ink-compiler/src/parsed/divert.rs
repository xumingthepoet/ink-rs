use crate::source::SourceSpan;

use super::{escape_snapshot_text, push_indent, Expression, QualifiedName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Divert {
    target: DivertTarget,
    arguments: Vec<Expression>,
    span: SourceSpan,
    is_tunnel: bool,
    has_argument_list: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DivertTarget {
    Path(String),
    QualifiedPath(QualifiedName),
    Dynamic(Expression),
    Empty,
}

impl Divert {
    pub fn new(target: DivertTarget, span: SourceSpan) -> Self {
        Self {
            target,
            arguments: Vec::new(),
            span,
            is_tunnel: false,
            has_argument_list: false,
        }
    }

    pub fn with_arguments(
        target: DivertTarget,
        arguments: Vec<Expression>,
        span: SourceSpan,
    ) -> Self {
        Self {
            target,
            arguments,
            span,
            is_tunnel: false,
            has_argument_list: true,
        }
    }

    pub fn with_tunnel(mut self) -> Self {
        self.is_tunnel = true;
        self
    }

    pub fn target(&self) -> &DivertTarget {
        &self.target
    }

    pub fn arguments(&self) -> &[Expression] {
        &self.arguments
    }

    pub fn has_argument_list(&self) -> bool {
        self.has_argument_list
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub fn is_tunnel(&self) -> bool {
        self.is_tunnel
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("Divert(target=\"-> ");
        out.push_str(&escape_snapshot_text(&self.target.to_snapshot_string()));
        out.push_str("\", empty=");
        out.push_str(if matches!(self.target, DivertTarget::Empty) {
            "true"
        } else {
            "false"
        });
        out.push_str(", tunnel=");
        out.push_str(if self.is_tunnel { "true" } else { "false" });
        out.push(')');
    }
}

impl DivertTarget {
    pub fn from_source(text: &str) -> Self {
        let normalized = text.trim();
        match normalized {
            "" => Self::Empty,
            other => Self::Path(other.to_string()),
        }
    }

    pub fn from_qualified(name: QualifiedName) -> Self {
        Self::QualifiedPath(name)
    }

    pub fn as_runtime_target(&self) -> Option<&str> {
        match self {
            Self::Path(target) => Some(target),
            Self::QualifiedPath(target) => Some(target.as_str()),
            Self::Dynamic(_) | Self::Empty => None,
        }
    }

    pub fn to_snapshot_string(&self) -> String {
        match self {
            Self::Path(target) => target.clone(),
            Self::QualifiedPath(target) => target.as_str().to_string(),
            Self::Dynamic(expression) => format!("{{{}}}", expression.to_source_string()),
            Self::Empty => String::new(),
        }
    }
}
