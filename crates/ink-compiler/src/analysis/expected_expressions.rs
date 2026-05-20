use std::collections::HashSet;

use crate::{
    parsed::{visit::VisitContext, Expression},
    source::SourceSpan,
};

#[derive(Debug, Default)]
pub(super) struct ExpectedExpressionSet {
    expression_ids: HashSet<usize>,
}

pub(super) fn expression_context_span(context: &VisitContext) -> SourceSpan {
    context
        .current_object_span
        .clone()
        .unwrap_or_else(|| SourceSpan::new(None, 1, 1))
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
