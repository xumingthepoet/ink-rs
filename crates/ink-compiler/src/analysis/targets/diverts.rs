use crate::{
    diagnostic::Diagnostic,
    parsed::{visit::VisitContext, DivertTarget, Expression, FlowArgument, TypeName},
    source::SourceSpan,
    syntax::parse_initial_expression,
};

use super::super::{
    expression_types::infer_expression_type,
    target_symbols::{is_cross_module_stitch_target, resolve_target_symbol},
};
use super::checker::CallTargetChecker;

impl<'a> CallTargetChecker<'a> {
    pub(super) fn check_plain_divert_target(
        &mut self,
        divert: &crate::parsed::Divert,
        context: &VisitContext,
    ) {
        let Some(target) = static_divert_target_name(divert.target()) else {
            return;
        };

        if self.check_plain_divert_target_expression(divert, context) {
            return;
        }

        let span = divert.span();
        if self.check_cross_module_stitch_target(target, span, context) {
            return;
        }

        let current_flow_path = self.current_flow_path(context);
        if let Some(symbol) = resolve_target_symbol(
            target,
            self.current_module(context),
            current_flow_path,
            self.target_symbols,
        ) {
            if symbol.is_function() {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "{target} can't be diverted to. It can only be called as a function since it's been marked as such: '{target}(...)'"
                    ),
                ));
            }
        } else if let Some((name, is_divert_target)) =
            resolve_current_flow_argument(target, self.current_flow_arguments(context))
                .map(|argument| (argument.name().to_string(), argument.is_divert_target()))
        {
            if !is_divert_target {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Since '{name}' is used as a variable divert target, it should be marked as: -> {name}"
                    ),
                ));
            }
        } else {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!("target not found: '{target}'"),
            ));
        }
    }

    pub(super) fn check_plain_divert_target_expression(
        &mut self,
        divert: &crate::parsed::Divert,
        context: &VisitContext,
    ) -> bool {
        let DivertTarget::Path(target) = divert.target() else {
            return false;
        };
        let current_flow_path = self.current_flow_path(context);

        if divert.has_argument_list()
            && resolve_target_symbol(
                target,
                self.current_module(context),
                current_flow_path,
                self.target_symbols,
            )
            .is_some_and(|symbol| {
                symbol.is_function() && symbol.return_type() == &TypeName::divert_target()
            })
        {
            let args = divert
                .arguments()
                .iter()
                .map(Expression::to_source_string)
                .collect::<Vec<_>>()
                .join(", ");
            self.diagnostics.push(Diagnostic::error(
                divert.span().clone(),
                format!(
                    "Static divert targets must be knot or stitch paths. Use `-> {{{target}({args})}}` for a divert-target expression."
                ),
            ));
            return true;
        }

        let Some(expression) = parse_initial_expression(target) else {
            return false;
        };

        if let Some(root) = expression_root_variable_name(&expression) {
            if self.variable_scopes.contains_visible_variable(
                root,
                self.current_module(context),
                current_flow_path,
            ) {
                self.diagnostics.push(Diagnostic::error(
                    divert.span().clone(),
                    format!(
                        "Static divert targets must be knot or stitch paths. Use `-> {{{target}}}` for a divert-target expression."
                    ),
                ));
                return true;
            }
        }

        false
    }

    pub(super) fn check_dynamic_divert_target(
        &mut self,
        expression: &Expression,
        arguments: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if let Expression::DynamicInterfaceAccess { target, member } = expression {
            self.check_dynamic_interface_divert_target(target, member, arguments, span, context);
            return;
        }

        self.check_expression(expression, span, context);
        match infer_expression_type(
            expression,
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            self.interface_members,
            self.current_module(context),
            self.current_flow_path(context),
        ) {
            Ok(actual_type) if actual_type == TypeName::divert_target() => {}
            Ok(actual_type) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Dynamic divert target has type {} but expected ->",
                    actual_type.display_name()
                ),
            )),
            Err(error) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Cannot type-check dynamic divert target: {}",
                    error.message()
                ),
            )),
        }
    }

    pub(super) fn check_divert_target_value(
        &mut self,
        target: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if self.check_cross_module_stitch_target(target, span, context) {
            return;
        }

        let variable_name = target.split('.').next().unwrap_or(target);
        if self.variable_scopes.contains_visible_variable(
            variable_name,
            self.current_module(context),
            self.current_flow_path(context),
        ) {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Since '{variable_name}' is a variable, it shouldn't be preceded by '->' here."
                ),
            ));
        }
    }

    pub(super) fn check_cross_module_stitch_target(
        &mut self,
        target: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) -> bool {
        if !is_cross_module_stitch_target(target, self.current_module(context)) {
            return false;
        }

        self.diagnostics.push(Diagnostic::error(
            span.clone(),
            format!(
                "Cross-module direct stitch access is not allowed: '{target}'. Import and reference the parent knot instead."
            ),
        ));
        true
    }
}

pub(super) fn static_divert_target_name(target: &DivertTarget) -> Option<&str> {
    match target {
        DivertTarget::Path(target) => Some(target),
        DivertTarget::QualifiedPath(target) => Some(target.as_str()),
        DivertTarget::Dynamic(_) | DivertTarget::Done | DivertTarget::End | DivertTarget::Empty => {
            None
        }
    }
}

fn expression_root_variable_name(expression: &Expression) -> Option<&str> {
    match expression {
        Expression::VariableReference(name) => Some(name),
        Expression::QualifiedReference(name) => Some(name.as_str()),
        Expression::FieldAccess { base, .. } | Expression::IndexAccess { base, .. } => {
            expression_root_variable_name(base)
        }
        Expression::DynamicInterfaceAccess { .. }
        | Expression::DynamicInterfaceFunctionCall { .. } => None,
        _ => None,
    }
}

fn resolve_current_flow_argument<'a>(
    target: &str,
    current_flow_arguments: Option<&'a [FlowArgument]>,
) -> Option<&'a FlowArgument> {
    let variable_target_name = target.split('.').next()?;
    current_flow_arguments?
        .iter()
        .find(|argument| argument.name() == variable_target_name)
}
