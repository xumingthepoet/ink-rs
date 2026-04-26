use crate::source::SourceSpan;

use super::{push_indent, Expression, TypeName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignmentTarget {
    Variable(String),
    FieldAccess {
        base: Box<AssignmentTarget>,
        field: String,
    },
    IndexAccess {
        base: Box<AssignmentTarget>,
        index: Expression,
    },
}

impl AssignmentTarget {
    pub fn variable(name: impl Into<String>) -> Self {
        Self::Variable(name.into())
    }

    pub fn from_expression(expression: Expression) -> Option<Self> {
        match expression {
            Expression::VariableReference(name) => Some(Self::Variable(name)),
            Expression::FieldAccess { base, field } => {
                let base = Self::from_expression(*base)?;
                Some(Self::FieldAccess {
                    base: Box::new(base),
                    field,
                })
            }
            Expression::IndexAccess { base, index } => {
                let base = Self::from_expression(*base)?;
                Some(Self::IndexAccess {
                    base: Box::new(base),
                    index: *index,
                })
            }
            _ => None,
        }
    }

    pub fn variable_name(&self) -> Option<&str> {
        match self {
            AssignmentTarget::Variable(name) => Some(name),
            AssignmentTarget::FieldAccess { .. } | AssignmentTarget::IndexAccess { .. } => None,
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            AssignmentTarget::Variable(name) => name.clone(),
            AssignmentTarget::FieldAccess { base, field } => {
                format!("{}.{}", base.display_name(), field)
            }
            AssignmentTarget::IndexAccess { base, index } => {
                let mut index_snapshot = String::new();
                index.write_parse_snapshot(&mut index_snapshot, 0);
                format!("{}[{}]", base.display_name(), index_snapshot)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableAssignment {
    name: String,
    target: AssignmentTarget,
    expression: Option<Expression>,
    declared_type: Option<TypeName>,
    is_global: bool,
    is_temporary: bool,
    span: SourceSpan,
}

impl VariableAssignment {
    pub fn new(
        name: impl Into<String>,
        expression: Option<Expression>,
        declared_type: Option<TypeName>,
        is_global: bool,
        is_temporary: bool,
        span: SourceSpan,
    ) -> Self {
        let name = name.into();
        Self::with_target(
            AssignmentTarget::variable(name),
            expression,
            declared_type,
            is_global,
            is_temporary,
            span,
        )
    }

    pub fn with_target(
        target: AssignmentTarget,
        expression: Option<Expression>,
        declared_type: Option<TypeName>,
        is_global: bool,
        is_temporary: bool,
        span: SourceSpan,
    ) -> Self {
        let name = target.display_name();
        Self {
            name,
            target,
            expression,
            declared_type,
            is_global,
            is_temporary,
            span,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn target(&self) -> &AssignmentTarget {
        &self.target
    }

    pub fn expression(&self) -> Option<&Expression> {
        self.expression.as_ref()
    }

    pub fn declared_type(&self) -> Option<&TypeName> {
        self.declared_type.as_ref()
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }

    pub fn is_temporary(&self) -> bool {
        self.is_temporary
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("VariableAssignment(name=\"");
        out.push_str(&self.name);
        out.push_str("\", global=");
        out.push_str(if self.is_global { "true" } else { "false" });
        out.push_str(", temp=");
        out.push_str(if self.is_temporary { "true" } else { "false" });
        if let Some(declared_type) = &self.declared_type {
            out.push_str(", type=");
            out.push_str(&declared_type.snapshot_name());
        }
        out.push(')');
        if let Some(expression) = &self.expression {
            out.push('\n');
            expression.write_parse_snapshot(out, indent + 2);
        }
    }
}
