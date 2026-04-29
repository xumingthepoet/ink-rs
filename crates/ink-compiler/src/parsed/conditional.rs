use super::{push_indent, Expression, Weave};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalKind {
    If,
    Switch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conditional {
    kind: ConditionalKind,
    initial_condition: Option<Expression>,
    branches: Vec<ConditionalBranch>,
}

impl Conditional {
    pub fn new(
        kind: ConditionalKind,
        initial_condition: Option<Expression>,
        branches: Vec<ConditionalBranch>,
    ) -> Self {
        Self {
            kind,
            initial_condition,
            branches,
        }
    }

    pub fn if_block(
        initial_condition: Option<Expression>,
        branches: Vec<ConditionalBranch>,
    ) -> Self {
        Self::new(ConditionalKind::If, initial_condition, branches)
    }

    pub fn switch(selector: Expression, branches: Vec<ConditionalBranch>) -> Self {
        Self::new(ConditionalKind::Switch, Some(selector), branches)
    }

    pub fn kind(&self) -> ConditionalKind {
        self.kind
    }

    pub fn initial_condition(&self) -> Option<&Expression> {
        self.initial_condition.as_ref()
    }

    pub fn branches(&self) -> &[ConditionalBranch] {
        &self.branches
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        push_indent(out, indent);
        out.push_str("Conditional");
        out.push_str(match self.kind {
            ConditionalKind::If => "(kind=if)",
            ConditionalKind::Switch => "(kind=switch)",
        });
        if let Some(condition) = &self.initial_condition {
            out.push('\n');
            condition.write_parse_snapshot(out, indent + 2);
        }
        for branch in &self.branches {
            branch.write_parse_snapshot(out, indent + 2);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalBranch {
    is_true_branch: bool,
    is_else: bool,
    is_inline: bool,
    content: Weave,
    own_condition: Option<Expression>,
}

impl ConditionalBranch {
    pub fn new(
        is_true_branch: bool,
        is_else: bool,
        is_inline: bool,
        content: Weave,
        own_condition: Option<Expression>,
    ) -> Self {
        Self {
            is_true_branch,
            is_else,
            is_inline,
            content,
            own_condition,
        }
    }

    pub fn is_true_branch(&self) -> bool {
        self.is_true_branch
    }

    pub fn is_else(&self) -> bool {
        self.is_else
    }

    pub fn is_inline(&self) -> bool {
        self.is_inline
    }

    pub fn content(&self) -> &Weave {
        &self.content
    }

    pub fn own_condition(&self) -> Option<&Expression> {
        self.own_condition.as_ref()
    }

    fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("ConditionalBranch(true=");
        out.push_str(if self.is_true_branch { "true" } else { "false" });
        out.push_str(", else=");
        out.push_str(if self.is_else { "true" } else { "false" });
        out.push_str(", inline=");
        out.push_str(if self.is_inline { "true" } else { "false" });
        out.push(')');
        self.content.write_parse_snapshot(out, indent + 2);
        if let Some(condition) = &self.own_condition {
            out.push('\n');
            condition.write_parse_snapshot(out, indent + 2);
        }
    }
}
