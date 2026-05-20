use std::collections::HashSet;

use crate::parsed::Expression;

#[derive(Debug, Default)]
pub(super) struct ExpectedExpressionSet {
    expression_ids: HashSet<usize>,
}

impl ExpectedExpressionSet {
    pub(super) fn mark(&mut self, expression: &Expression) {
        self.expression_ids
            .insert(expression as *const Expression as usize);
    }

    pub(super) fn contains(&self, expression: &Expression) -> bool {
        self.expression_ids
            .contains(&(expression as *const Expression as usize))
    }
}
