use std::collections::{HashMap, HashSet};

use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_story, ParsedVisitor, VisitContext},
        ContentList, DivertTarget, Expression, Flow, FlowArgument, Object, Story, Weave,
    },
    source::SourceSpan,
};

use super::{
    span::object_span,
    variables::{build_variable_scope_index, VariableScopeIndex},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FlowSymbol {
    is_function: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum TargetSymbolCollectionPhase {
    #[default]
    Flows,
    Labels,
}

pub(super) fn call_target_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let target_symbols = build_target_symbol_index(story);
    let variable_targets = build_variable_target_index(story);
    let variable_scopes = build_variable_scope_index(story);
    let mut checker = CallTargetChecker::new(&target_symbols, &variable_targets, &variable_scopes);
    walk_story(story, &mut checker);
    checker.diagnostics
}

struct CallTargetChecker<'a> {
    target_symbols: &'a HashMap<String, FlowSymbol>,
    variable_targets: &'a HashSet<String>,
    variable_scopes: &'a VariableScopeIndex,
    diagnostics: Vec<Diagnostic>,
    arguments_by_flow_path: HashMap<String, Vec<FlowArgument>>,
    functions_by_flow_path: HashMap<String, bool>,
}

impl<'a> CallTargetChecker<'a> {
    fn new(
        target_symbols: &'a HashMap<String, FlowSymbol>,
        variable_targets: &'a HashSet<String>,
        variable_scopes: &'a VariableScopeIndex,
    ) -> Self {
        Self {
            target_symbols,
            variable_targets,
            variable_scopes,
            diagnostics: Vec::new(),
            arguments_by_flow_path: HashMap::new(),
            functions_by_flow_path: HashMap::new(),
        }
    }

    fn current_flow_path<'context>(
        &self,
        context: &'context VisitContext,
    ) -> Option<&'context str> {
        context.current_flow_path.as_deref()
    }

    fn current_flow_arguments(&self, context: &VisitContext) -> Option<&[FlowArgument]> {
        self.current_flow_path(context).and_then(|flow_path| {
            self.arguments_by_flow_path
                .get(flow_path)
                .map(Vec::as_slice)
        })
    }

    fn current_flow_is_function(&self, context: &VisitContext) -> bool {
        self.current_flow_path(context)
            .and_then(|flow_path| self.functions_by_flow_path.get(flow_path))
            .copied()
            .unwrap_or(false)
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
                                "{name} hasn't been marked as a function, but it's being called as one. Do you need to delcare the knot as '== function {name} =='?"
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
        self.arguments_by_flow_path
            .entry(flow_path.to_string())
            .or_insert_with(|| flow.arguments().to_vec());
        self.functions_by_flow_path
            .entry(flow_path.to_string())
            .or_insert(flow.is_function());
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

fn build_target_symbol_index(story: &Story) -> HashMap<String, FlowSymbol> {
    #[derive(Default)]
    struct TargetSymbolVisitor {
        phase: TargetSymbolCollectionPhase,
        symbols: HashMap<String, FlowSymbol>,
    }

    impl ParsedVisitor for TargetSymbolVisitor {
        fn visit_flow(&mut self, flow: &Flow, context: &VisitContext) {
            if self.phase != TargetSymbolCollectionPhase::Flows {
                return;
            }

            let Some(flow_path) = &context.current_flow_path else {
                return;
            };
            let symbol = FlowSymbol {
                is_function: flow.is_function(),
            };

            self.symbols.entry(flow_path.clone()).or_insert(symbol);
            if context.parent_flow_path.is_none() {
                self.symbols
                    .entry(flow.name().to_string())
                    .or_insert(symbol);
            }
        }

        fn visit_object(&mut self, object: &Object, context: &VisitContext) {
            if self.phase != TargetSymbolCollectionPhase::Labels {
                return;
            }

            match object {
                Object::Choice(choice) => {
                    if let Some(identifier) = choice.identifier() {
                        insert_label_symbol(
                            &mut self.symbols,
                            identifier,
                            context.current_flow_path.as_deref(),
                        );
                    }
                }
                Object::Gather(gather) => {
                    if let Some(identifier) = gather.identifier() {
                        insert_label_symbol(
                            &mut self.symbols,
                            identifier,
                            context.current_flow_path.as_deref(),
                        );
                    }
                }
                _ => {}
            }
        }
    }

    let mut visitor = TargetSymbolVisitor::default();
    walk_story(story, &mut visitor);
    visitor.phase = TargetSymbolCollectionPhase::Labels;
    walk_story(story, &mut visitor);
    visitor.symbols
}

fn insert_label_symbol(
    symbols: &mut HashMap<String, FlowSymbol>,
    identifier: &str,
    flow_path: Option<&str>,
) {
    let symbol = FlowSymbol { is_function: false };
    symbols.entry(identifier.to_string()).or_insert(symbol);
    if let Some(flow_path) = flow_path {
        symbols
            .entry(format!("{flow_path}.{identifier}"))
            .or_insert(symbol);
    }
}

fn build_variable_target_index(story: &Story) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_variable_targets_in_weave(story.root_weave(), &mut names);
    for flow in story.flows() {
        collect_variable_targets_in_flow(flow, &mut names);
    }
    names
}

fn collect_variable_targets_in_flow(flow: &Flow, names: &mut HashSet<String>) {
    for argument in flow.arguments() {
        names.insert(argument.name().to_string());
    }
    collect_variable_targets_in_weave(flow.weave(), names);
    for child in flow.child_flows() {
        collect_variable_targets_in_flow(child, names);
    }
}

fn collect_variable_targets_in_weave(weave: &Weave, names: &mut HashSet<String>) {
    for object in weave.content() {
        collect_variable_targets_in_object(object, names);
    }
}

fn collect_variable_targets_in_content_list(content: &ContentList, names: &mut HashSet<String>) {
    for object in content.objects() {
        collect_variable_targets_in_object(object, names);
    }
}

fn collect_variable_targets_in_object(object: &Object, names: &mut HashSet<String>) {
    match object {
        Object::VariableAssignment(assignment) => {
            names.insert(assignment.name().to_string());
            collect_variable_targets_in_expression(assignment.expression(), names);
        }
        Object::Choice(choice) => {
            if let Some(condition) = choice.condition() {
                collect_variable_targets_in_expression(condition, names);
            }
            if let Some(content) = choice.start_content() {
                collect_variable_targets_in_content_list(content, names);
            }
            if let Some(content) = choice.choice_only_content() {
                collect_variable_targets_in_content_list(content, names);
            }
            collect_variable_targets_in_content_list(choice.inner_content(), names);
        }
        Object::ContentList(content) => collect_variable_targets_in_content_list(content, names),
        Object::Conditional(conditional) => {
            if let Some(condition) = conditional.initial_condition() {
                collect_variable_targets_in_expression(condition, names);
            }
            for branch in conditional.branches() {
                if let Some(condition) = branch.own_condition() {
                    collect_variable_targets_in_expression(condition, names);
                }
                collect_variable_targets_in_weave(branch.content(), names);
            }
        }
        Object::Expression(expression) | Object::LogicLine(expression) => {
            collect_variable_targets_in_expression(expression, names);
        }
        Object::IncDec(inc_dec) => {
            collect_variable_targets_in_expression(inc_dec.expression(), names)
        }
        Object::Return(ret) => {
            if let Some(expression) = ret.returned_expression() {
                collect_variable_targets_in_expression(expression, names);
            }
        }
        Object::Sequence(sequence) => {
            for content in sequence.elements() {
                collect_variable_targets_in_content_list(content, names);
            }
        }
        Object::Weave(weave) => collect_variable_targets_in_weave(weave, names),
        Object::AuthorWarning(_)
        | Object::ConstantDeclaration(_)
        | Object::Divert(_)
        | Object::ExternalDeclaration(_)
        | Object::Gather(_)
        | Object::Glue(_)
        | Object::Tag(_)
        | Object::Text(_)
        | Object::TunnelOnwards(_) => {}
    }
}

fn collect_variable_targets_in_expression(expression: &Expression, names: &mut HashSet<String>) {
    match expression {
        Expression::StringContent(content) => {
            collect_variable_targets_in_content_list(content, names)
        }
        Expression::FunctionCall { args, .. } => {
            for arg in args {
                collect_variable_targets_in_expression(arg, names);
            }
        }
        Expression::Binary { left, right, .. } => {
            collect_variable_targets_in_expression(left, names);
            collect_variable_targets_in_expression(right, names);
        }
        Expression::Unary { expression, .. } => {
            collect_variable_targets_in_expression(expression, names)
        }
        Expression::MultipleCondition(expressions) => {
            for expression in expressions {
                collect_variable_targets_in_expression(expression, names);
            }
        }
        Expression::String(_)
        | Expression::NumberInt(_)
        | Expression::NumberFloat(_)
        | Expression::NumberBool(_)
        | Expression::DivertTarget(_)
        | Expression::VariableReference(_) => {}
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

fn resolve_target_symbol<'a>(
    target: &str,
    current_flow_path: Option<&str>,
    target_symbols: &'a HashMap<String, FlowSymbol>,
) -> Option<&'a FlowSymbol> {
    if target.contains('.') {
        return target_symbols.get(target);
    }

    if let Some(flow_path) = current_flow_path {
        if let Some(symbol) = target_symbols.get(&format!("{flow_path}.{target}")) {
            return Some(symbol);
        }
        if let Some((parent_flow_path, _)) = flow_path.rsplit_once('.') {
            if let Some(symbol) = target_symbols.get(&format!("{parent_flow_path}.{target}")) {
                return Some(symbol);
            }
        }
    }

    target_symbols.get(target)
}
