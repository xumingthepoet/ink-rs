use std::collections::HashMap;

use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_story, ParsedVisitor, VisitContext},
        DivertTarget, Expression, Flow, FlowArgument, Object, Story,
    },
    source::SourceSpan,
};

use super::{
    context::{FlowContext, TargetSymbolIndex, VariableScopeIndex, VariableTargetIndex},
    span::object_span,
    target_symbols::{build_target_symbol_index, resolve_target_symbol},
    variable_targets::build_variable_target_index,
    variables::build_variable_scope_index,
};

pub(super) fn call_target_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let target_symbols = build_target_symbol_index(story);
    let variable_targets = build_variable_target_index(story);
    let variable_scopes = build_variable_scope_index(story);
    let mut checker = CallTargetChecker::new(&target_symbols, &variable_targets, &variable_scopes);
    walk_story(story, &mut checker);
    checker.diagnostics
}

struct CallTargetChecker<'a> {
    target_symbols: &'a TargetSymbolIndex,
    variable_targets: &'a VariableTargetIndex,
    variable_scopes: &'a VariableScopeIndex,
    diagnostics: Vec<Diagnostic>,
    flow_contexts_by_path: HashMap<String, FlowContext>,
}

impl<'a> CallTargetChecker<'a> {
    fn new(
        target_symbols: &'a TargetSymbolIndex,
        variable_targets: &'a VariableTargetIndex,
        variable_scopes: &'a VariableScopeIndex,
    ) -> Self {
        Self {
            target_symbols,
            variable_targets,
            variable_scopes,
            diagnostics: Vec::new(),
            flow_contexts_by_path: HashMap::new(),
        }
    }

    fn current_flow_path<'context>(
        &self,
        context: &'context VisitContext,
    ) -> Option<&'context str> {
        context.current_flow_path.as_deref()
    }

    fn current_flow_context(&self, context: &VisitContext) -> Option<&FlowContext> {
        self.current_flow_path(context)
            .and_then(|flow_path| self.flow_contexts_by_path.get(flow_path))
    }

    fn current_flow_arguments(&self, context: &VisitContext) -> Option<&[FlowArgument]> {
        self.current_flow_context(context)
            .map(FlowContext::arguments)
    }

    fn current_flow_is_function(&self, context: &VisitContext) -> bool {
        self.current_flow_context(context)
            .is_some_and(FlowContext::is_function)
    }

    fn check_plain_divert_target(
        &mut self,
        target: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let current_flow_path = self.current_flow_path(context);
        if let Some(symbol) = resolve_target_symbol(target, current_flow_path, self.target_symbols)
        {
            if symbol.is_function {
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
        } else if !self.variable_targets.contains(target) {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!("target not found: '{target}'"),
            ));
        }
    }

    fn check_expression(
        &mut self,
        expression: &Expression,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        match expression {
            Expression::FunctionCall { name, args } => {
                if let Some(symbol) = resolve_target_symbol(
                    name,
                    self.current_flow_path(context),
                    self.target_symbols,
                ) {
                    if !symbol.is_function {
                        self.diagnostics.push(Diagnostic::error(
                            span.clone(),
                            format!(
                                "{name} hasn't been marked as a function, but it's being called as one. Do you need to declare the knot as '== function {name} =='?"
                            ),
                        ));
                    }
                }
                for arg in args {
                    self.check_expression(arg, span, context);
                }
            }
            Expression::Binary { left, right, .. } => {
                self.check_expression(left, span, context);
                self.check_expression(right, span, context);
            }
            Expression::Unary { expression, .. } => {
                self.check_expression(expression, span, context);
            }
            Expression::MultipleCondition(expressions) => {
                for expression in expressions {
                    self.check_expression(expression, span, context);
                }
            }
            Expression::DivertTarget(target) => {
                self.check_divert_target_value(target, span, context);
            }
            Expression::VariableReference(name) => {
                self.check_variable_reference(name, span, context);
            }
            Expression::StringContent(_)
            | Expression::String(_)
            | Expression::NumberInt(_)
            | Expression::NumberFloat(_)
            | Expression::NumberBool(_) => {}
        }
    }

    fn check_variable_reference(&mut self, name: &str, span: &SourceSpan, context: &VisitContext) {
        let current_flow_path = self.current_flow_path(context);
        if name.contains('.')
            || resolve_target_symbol(name, current_flow_path, self.target_symbols).is_some()
        {
            return;
        }

        if self
            .variable_scopes
            .contains_visible_variable(name, current_flow_path)
        {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            span.clone(),
            format!("Unresolved variable: {name}"),
        ));
    }

    fn check_divert_target_value(
        &mut self,
        target: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let variable_name = target.split('.').next().unwrap_or(target);
        if self
            .variable_scopes
            .contains_visible_variable(variable_name, self.current_flow_path(context))
        {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Since '{variable_name}' is a variable, it shouldn't be preceded by '->' here."
                ),
            ));
        }
    }
}

impl ParsedVisitor for CallTargetChecker<'_> {
    fn visit_flow(&mut self, flow: &Flow, context: &VisitContext) {
        let Some(flow_path) = self.current_flow_path(context) else {
            return;
        };
        self.flow_contexts_by_path
            .entry(flow_path.to_string())
            .or_insert_with(|| FlowContext::new(flow.arguments().to_vec(), flow.is_function()));
    }

    fn visit_object(&mut self, object: &Object, context: &VisitContext) {
        match object {
            Object::Divert(divert) => {
                match divert.target() {
                    DivertTarget::Empty => self.diagnostics.push(Diagnostic::error(
                        divert.span().clone(),
                        "Empty diverts (->) are only valid on choices",
                    )),
                    DivertTarget::Path(target) if !self.current_flow_is_function(context) => {
                        self.check_plain_divert_target(target, divert.span(), context);
                    }
                    DivertTarget::Path(_) | DivertTarget::Done | DivertTarget::End => {}
                }
                for argument in divert.arguments() {
                    self.check_expression(argument, divert.span(), context);
                }
            }
            Object::ConstantDeclaration(declaration) => {
                self.check_expression(declaration.expression(), declaration.span(), context);
            }
            Object::Expression(expression) | Object::LogicLine(expression) => {
                self.check_expression(expression, &object_span(object), context);
            }
            Object::VariableAssignment(assignment) => {
                self.check_expression(assignment.expression(), assignment.span(), context);
            }
            Object::IncDec(inc_dec) => {
                self.check_expression(inc_dec.expression(), inc_dec.span(), context);
            }
            Object::Return(ret) => {
                if let Some(expression) = ret.returned_expression() {
                    self.check_expression(expression, ret.span(), context);
                }
            }
            Object::Conditional(conditional) => {
                let span = object_span(object);
                if let Some(condition) = conditional.initial_condition() {
                    self.check_expression(condition, &span, context);
                }
                for branch in conditional.branches() {
                    if let Some(condition) = branch.own_condition() {
                        self.check_expression(condition, &span, context);
                    }
                }
            }
            Object::Choice(choice) => {
                if let Some(condition) = choice.condition() {
                    self.check_expression(condition, choice.span(), context);
                }
            }
            Object::TunnelOnwards(tunnel_onwards) => {
                for argument in tunnel_onwards.arguments() {
                    self.check_expression(argument, tunnel_onwards.span(), context);
                }
            }
            Object::AuthorWarning(_)
            | Object::ContentList(_)
            | Object::ExternalDeclaration(_)
            | Object::Gather(_)
            | Object::Glue(_)
            | Object::Sequence(_)
            | Object::Tag(_)
            | Object::Text(_)
            | Object::Weave(_) => {}
        }
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

#[cfg(test)]
mod tests {
    use crate::diagnostic::DiagnosticSeverity;

    use super::{
        super::test_support::{assert_single_diagnostic, parse_story},
        *,
    };

    #[test]
    fn reports_missing_divert_targets() {
        let story = parse_story("-> missing");
        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "target not found: 'missing'",
        );
    }

    #[test]
    fn reports_non_function_call_targets() {
        let story = parse_story("~ knot()\n== knot ==\n-> DONE");
        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "knot hasn't been marked as a function, but it's being called as one. Do you need to declare the knot as '== function knot =='?",
        );
    }
}
