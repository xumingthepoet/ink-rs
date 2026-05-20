use crate::parsed::Expression;

use super::super::is_identifier;
use super::{
    error::{describe_token_kind, ExpressionParseError, ExpressionParseErrorKind},
    parser::TokenExpressionParser,
    token::ExpressionTokenKind,
};

impl<'a> TokenExpressionParser<'a> {
    pub(super) fn current_brace_pair_is_dynamic_interface_target(&self) -> bool {
        let mut depth = 1;
        for (index, token) in self.tokens.iter().enumerate().skip(self.index) {
            match &token.kind {
                ExpressionTokenKind::OpenBrace => depth += 1,
                ExpressionTokenKind::CloseBrace => {
                    depth -= 1;
                    if depth == 0 {
                        return self.tokens.get(index + 1).is_some_and(|next| {
                            matches!(&next.kind, ExpressionTokenKind::DoubleColon)
                        });
                    }
                }
                _ => {}
            }
        }

        false
    }

    pub(super) fn parse_dynamic_interface_target(
        &mut self,
    ) -> Result<Expression, ExpressionParseError> {
        let target = self.parse_expression(0)?;
        self.expect_kind(
            |kind| matches!(kind, ExpressionTokenKind::CloseBrace),
            |found| ExpressionParseErrorKind::ExpectedDynamicInterfaceTargetClose { found },
        )?;
        self.expect_kind(
            |kind| matches!(kind, ExpressionTokenKind::DoubleColon),
            |found| ExpressionParseErrorKind::ExpectedDynamicInterfaceMember { found },
        )?;
        self.parse_dynamic_interface_member_after_target(target)
    }

    fn parse_dynamic_interface_member_after_target(
        &mut self,
        target: Expression,
    ) -> Result<Expression, ExpressionParseError> {
        let Some(token) = self.advance() else {
            return Err(self.error_at_eof(
                ExpressionParseErrorKind::ExpectedDynamicInterfaceMember { found: None },
            ));
        };
        let kind = token.kind.clone();
        let ExpressionTokenKind::Identifier(member) = kind else {
            return Err(ExpressionParseError::new(
                ExpressionParseErrorKind::ExpectedDynamicInterfaceMember {
                    found: Some(describe_token_kind(&kind)),
                },
                token.span.clone(),
            ));
        };
        if !is_identifier(&member) {
            return Err(ExpressionParseError::new(
                ExpressionParseErrorKind::ExpectedDynamicInterfaceMember {
                    found: Some(member),
                },
                token.span.clone(),
            ));
        }

        if !self.match_kind(|kind| matches!(kind, ExpressionTokenKind::OpenParen)) {
            return Ok(Expression::DynamicInterfaceAccess {
                target: Box::new(target),
                member,
            });
        }

        let args = self.parse_argument_list(&member)?;
        Ok(Expression::DynamicInterfaceFunctionCall {
            target: Box::new(target),
            member,
            args,
        })
    }
}
