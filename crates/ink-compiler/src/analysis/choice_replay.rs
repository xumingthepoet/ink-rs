use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_story, ParsedVisitor, VisitContext},
        Choice, ContentList, Expression, Object, Story,
    },
    source::SourceSpan,
};

pub(super) fn choice_replay_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let mut checker = ChoiceReplayChecker {
        diagnostics: Vec::new(),
    };
    walk_story(story, &mut checker);
    checker.diagnostics
}

struct ChoiceReplayChecker {
    diagnostics: Vec<Diagnostic>,
}

impl ParsedVisitor for ChoiceReplayChecker {
    fn visit_object(&mut self, object: &Object, _context: &VisitContext) {
        let Object::Choice(choice) = object else {
            return;
        };
        self.check_choice(choice);
    }
}

impl ChoiceReplayChecker {
    fn check_choice(&mut self, choice: &Choice) {
        let span = choice.span();

        if let Some(binding) = choice.dynamic_binding() {
            self.check_expression(
                binding.iterable(),
                ChoiceReplayContext::DynamicIterable,
                span,
            );
        }

        if let Some(condition) = choice.condition() {
            self.check_expression(condition, ChoiceReplayContext::Condition, span);
        }

        if let Some(content) = choice.start_content() {
            self.check_content(content, ChoiceReplayContext::Text, span);
        }
    }

    fn check_content(
        &mut self,
        content: &ContentList,
        context: ChoiceReplayContext,
        choice_span: &SourceSpan,
    ) {
        for object in content.objects() {
            self.check_object(object, context, choice_span);
        }
    }

    fn check_object(
        &mut self,
        object: &Object,
        context: ChoiceReplayContext,
        choice_span: &SourceSpan,
    ) {
        match object {
            Object::Text(_) | Object::Glue(_) | Object::Tag(_) => {}
            Object::ContentList(content) => self.check_content(content, context, choice_span),
            Object::Expression(expression) | Object::LogicLine(expression) => {
                self.check_expression(expression, context, choice_span);
            }
            Object::Conditional(conditional) => {
                if let Some(condition) = conditional.initial_condition() {
                    self.check_expression(condition, context, choice_span);
                }
                for branch in conditional.branches() {
                    if let Some(condition) = branch.own_condition() {
                        self.check_expression(condition, context, choice_span);
                    }
                    self.check_content(
                        &ContentList::new(branch.content().content().to_vec()),
                        context,
                        choice_span,
                    );
                }
            }
            Object::ForLoop(for_loop) => {
                self.check_expression(for_loop.iterable(), context, choice_span);
                self.check_content(
                    &ContentList::new(for_loop.body().content().to_vec()),
                    context,
                    choice_span,
                );
            }
            Object::VariableAssignment(_)
            | Object::IncDec(_)
            | Object::Divert(_)
            | Object::TunnelOnwards(_)
            | Object::Return(_) => {
                self.report(
                    context,
                    choice_span,
                    "statements with side effects are not allowed during choice generation",
                );
            }
            Object::Choice(_)
            | Object::Gather(_)
            | Object::Weave(_)
            | Object::AuthorWarning(_)
            | Object::ConstantDeclaration(_)
            | Object::EnumDeclaration(_)
            | Object::StructDeclaration(_)
            | Object::ExternalDeclaration(_) => {}
        }
    }

    fn check_expression(
        &mut self,
        expression: &Expression,
        context: ChoiceReplayContext,
        choice_span: &SourceSpan,
    ) {
        match expression {
            Expression::FunctionCall { name, args } => {
                if is_replay_safe_builtin(name) {
                    for arg in args {
                        self.check_expression(arg, context, choice_span);
                    }
                } else if matches!(name.as_str(), "RANDOM" | "SEED_RANDOM") {
                    self.report(
                        context,
                        choice_span,
                        format!("builtin '{name}' is not allowed during choice generation"),
                    );
                } else {
                    self.report(
                        context,
                        choice_span,
                        format!("function call '{name}' is not allowed during choice generation"),
                    );
                }
            }
            Expression::QualifiedFunctionCall { name, args } => {
                self.report(
                    context,
                    choice_span,
                    format!(
                        "function call '{}' is not allowed during choice generation",
                        name.as_str()
                    ),
                );
                for arg in args {
                    self.check_expression(arg, context, choice_span);
                }
            }
            Expression::DynamicInterfaceFunctionCall {
                target,
                member,
                args,
            } => {
                self.report(
                    context,
                    choice_span,
                    format!(
                        "dynamic interface function '{member}' is not allowed during choice generation"
                    ),
                );
                self.check_expression(target, context, choice_span);
                for arg in args {
                    self.check_expression(arg, context, choice_span);
                }
            }
            Expression::DynamicInterfaceAccess { target, .. } => {
                self.check_expression(target, context, choice_span);
            }
            Expression::StringContent(content) => self.check_content(content, context, choice_span),
            Expression::ArrayLiteral(elements) | Expression::MultipleCondition(elements) => {
                for element in elements {
                    self.check_expression(element, context, choice_span);
                }
            }
            Expression::StructLiteral { fields, .. } => {
                for field in fields {
                    self.check_expression(field.expression(), context, choice_span);
                }
            }
            Expression::DictLiteral(entries) => {
                for entry in entries {
                    self.check_expression(entry.value(), context, choice_span);
                }
            }
            Expression::FieldAccess { base, .. } => {
                self.check_expression(base, context, choice_span);
            }
            Expression::IndexAccess { base, index } => {
                self.check_expression(base, context, choice_span);
                self.check_expression(index, context, choice_span);
            }
            Expression::Binary { left, right, .. } => {
                self.check_expression(left, context, choice_span);
                self.check_expression(right, context, choice_span);
            }
            Expression::Unary { expression, .. } => {
                self.check_expression(expression, context, choice_span);
            }
            Expression::String(_)
            | Expression::NumberInt(_)
            | Expression::NumberFloat(_)
            | Expression::NumberBool(_)
            | Expression::DivertTarget(_)
            | Expression::VariableReference(_)
            | Expression::QualifiedReference(_) => {}
        }
    }

    fn report(
        &mut self,
        context: ChoiceReplayContext,
        span: &SourceSpan,
        reason: impl Into<String>,
    ) {
        self.diagnostics.push(Diagnostic::error(
            span.clone(),
            format!("{} must be replay-safe; {}", context.label(), reason.into()),
        ));
    }
}

#[derive(Debug, Clone, Copy)]
enum ChoiceReplayContext {
    Condition,
    DynamicIterable,
    Text,
}

impl ChoiceReplayContext {
    fn label(self) -> &'static str {
        match self {
            Self::Condition => "Choice condition",
            Self::DynamicIterable => "Dynamic choice iterable",
            Self::Text => "Choice text",
        }
    }
}

fn is_replay_safe_builtin(name: &str) -> bool {
    matches!(
        name,
        "MIN"
            | "MAX"
            | "POW"
            | "FLOOR"
            | "CEILING"
            | "INT"
            | "FLOAT"
            | "LEN"
            | "DICT_HAS"
            | "DICT_SIZE"
            | "DICT_KEYS"
            | "to_str"
    )
}
