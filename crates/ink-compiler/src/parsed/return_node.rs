use super::push_indent;
use super::Expression;
use crate::source::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Return {
    returned_expression: Option<Expression>,
    span: SourceSpan,
}

impl Return {
    pub fn new(returned_expression: Option<Expression>, span: SourceSpan) -> Self {
        Self {
            returned_expression,
            span,
        }
    }

    pub fn returned_expression(&self) -> Option<&Expression> {
        self.returned_expression.as_ref()
    }

    pub(crate) fn direct_self_tail_call_args(
        &self,
        current_function_name: &str,
    ) -> Option<&[Expression]> {
        let Expression::FunctionCall { name, args } = self.returned_expression.as_ref()? else {
            return None;
        };
        (name == current_function_name).then_some(args)
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("Return");
        if let Some(expr) = &self.returned_expression {
            out.push('\n');
            expr.write_parse_snapshot(out, indent + 2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsed::BinaryOperator;
    use crate::source::SourceSpan;

    fn span() -> SourceSpan {
        SourceSpan::new(None, 1, 1)
    }

    fn self_call() -> Expression {
        Expression::FunctionCall {
            name: "countdown".to_string(),
            args: vec![
                Expression::NumberInt(1),
                Expression::VariableReference("n".into()),
            ],
        }
    }

    #[test]
    fn detects_direct_self_tail_return() {
        let ret = Return::new(Some(self_call()), span());

        let args = ret
            .direct_self_tail_call_args("countdown")
            .expect("return countdown(...) should be classified as direct self tail call");

        assert_eq!(args.len(), 2);
    }

    #[test]
    fn rejects_non_tail_recursive_call_inside_larger_expression() {
        let ret = Return::new(
            Some(Expression::Binary {
                operator: BinaryOperator::Add,
                left: Box::new(Expression::NumberInt(1)),
                right: Box::new(self_call()),
            }),
            span(),
        );

        assert!(ret.direct_self_tail_call_args("countdown").is_none());
    }

    #[test]
    fn rejects_non_self_function_call_and_bare_return() {
        let other_call = Return::new(
            Some(Expression::FunctionCall {
                name: "other".to_string(),
                args: vec![Expression::NumberInt(1)],
            }),
            span(),
        );
        let bare_return = Return::new(None, span());

        assert!(other_call.direct_self_tail_call_args("countdown").is_none());
        assert!(bare_return
            .direct_self_tail_call_args("countdown")
            .is_none());
    }
}
