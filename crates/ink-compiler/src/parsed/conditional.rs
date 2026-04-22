use std::fmt;

use super::{Object, ObjectKind, ObjectRef};

#[derive(Debug, Clone)]
pub struct Conditional {
    object: ObjectRef,
}

impl Conditional {
    pub fn new(
        initial_condition: Option<ObjectRef>,
        branches: Vec<ConditionalSingleBranch>,
    ) -> Self {
        let object = Object::new_ref();
        object.borrow_mut().set_conditional_kind();

        if let Some(initial_condition) = initial_condition {
            Object::add_content(&object, initial_condition);
        }

        for branch in branches {
            Object::add_content(&object, branch.object());
        }

        Self { object }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn content(&self) -> Vec<ObjectRef> {
        self.object.borrow().content().to_vec()
    }
}

impl fmt::Display for Conditional {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Conditional")
    }
}

#[derive(Debug, Clone)]
pub struct ConditionalSingleBranch {
    object: ObjectRef,
    own_expression: Option<ObjectRef>,
    matching_equality: bool,
    is_true_branch: bool,
    is_else: bool,
    is_inline: bool,
}

impl ConditionalSingleBranch {
    pub fn new(content: Vec<ObjectRef>) -> Self {
        let object = Object::new_ref();
        object
            .borrow_mut()
            .set_conditional_branch_kind(false, false, false);

        for child in content {
            Object::add_content(&object, child);
        }

        Self {
            object,
            own_expression: None,
            matching_equality: false,
            is_true_branch: false,
            is_else: false,
            is_inline: false,
        }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn set_is_true_branch(&mut self, value: bool) {
        self.is_true_branch = value;
        if let ObjectKind::ConditionalSingleBranch { is_true_branch, .. } =
            self.object.borrow_mut().kind_mut()
        {
            *is_true_branch = value;
        }
    }

    pub fn set_is_else(&mut self, value: bool) {
        self.is_else = value;
        if let ObjectKind::ConditionalSingleBranch { is_else, .. } =
            self.object.borrow_mut().kind_mut()
        {
            *is_else = value;
        }
    }

    pub fn set_own_expression(&mut self, value: Option<ObjectRef>) {
        self.own_expression = value.clone();
        if let Some(own_expression) = value {
            Object::insert_content(&self.object, 0, own_expression);
        }
    }

    pub fn set_matching_equality(&mut self, value: bool) {
        self.matching_equality = value;
    }

    pub fn set_is_inline(&mut self, value: bool) {
        self.is_inline = value;
        if let ObjectKind::ConditionalSingleBranch { is_inline, .. } =
            self.object.borrow_mut().kind_mut()
        {
            *is_inline = value;
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

    pub fn matching_equality(&self) -> bool {
        self.matching_equality
    }

    pub fn own_expression(&self) -> Option<ObjectRef> {
        self.own_expression.clone()
    }

    pub fn content(&self) -> Vec<ObjectRef> {
        self.object.borrow().content().to_vec()
    }
}

impl fmt::Display for ConditionalSingleBranch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ConditionalBranch(true={}, else={}, inline={})",
            self.is_true_branch, self.is_else, self.is_inline
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Conditional, ConditionalSingleBranch};
    use crate::parsed::{ObjectKind, Text};

    #[test]
    fn conditional_wraps_branches_and_condition() {
        let condition = Text::new("x > 3").object();
        let mut branch = ConditionalSingleBranch::new(vec![Text::new("yes").object()]);
        branch.set_is_true_branch(true);
        branch.set_is_inline(true);
        let conditional = Conditional::new(Some(condition), vec![branch]);

        assert_eq!(conditional.content().len(), 2);
        assert!(matches!(
            conditional.object().borrow().kind(),
            ObjectKind::Conditional
        ));
        assert_eq!(conditional.to_string(), "Conditional");
    }

    #[test]
    fn conditional_branch_tracks_flags_and_content() {
        let mut branch = ConditionalSingleBranch::new(vec![Text::new("hello").object()]);
        branch.set_is_true_branch(true);
        branch.set_is_else(false);
        branch.set_is_inline(true);

        assert!(branch.is_true_branch());
        assert!(!branch.is_else());
        assert!(branch.is_inline());
        assert_eq!(branch.content().len(), 1);
        assert!(matches!(
            branch.object().borrow().kind(),
            ObjectKind::ConditionalSingleBranch {
                is_true_branch: true,
                is_else: false,
                is_inline: true,
            }
        ));
        assert_eq!(
            branch.to_string(),
            "ConditionalBranch(true=true, else=false, inline=true)"
        );
    }

    #[test]
    fn conditional_branch_tracks_own_expression() {
        let mut branch = ConditionalSingleBranch::new(vec![Text::new("hello").object()]);
        branch.set_own_expression(Some(Text::new("x == 3").object()));
        branch.set_matching_equality(true);

        assert!(branch.own_expression().is_some());
        assert!(branch.matching_equality());
        assert_eq!(branch.content().len(), 2);
    }
}
