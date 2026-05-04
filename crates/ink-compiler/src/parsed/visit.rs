use super::{
    Choice, Conditional, ContentList, Expression, Flow, ImportDeclaration, Module, Object, Story,
    Weave,
};

pub(crate) trait ParsedVisitor {
    fn visit_story(&mut self, _story: &Story, _context: &VisitContext) {}
    fn visit_module(&mut self, _module: &Module, _context: &VisitContext) {}
    fn visit_import(&mut self, _import: &ImportDeclaration, _context: &VisitContext) {}
    fn visit_flow(&mut self, _flow: &Flow, _context: &VisitContext) {}
    fn visit_weave(&mut self, _weave: &Weave, _context: &VisitContext) {}
    fn visit_content_list(&mut self, _content: &ContentList, _context: &VisitContext) {}
    fn visit_object(&mut self, _object: &Object, _context: &VisitContext) {}
    fn visit_expression(&mut self, _expression: &Expression, _context: &VisitContext) {}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct VisitContext {
    pub(crate) current_module: Option<String>,
    pub(crate) current_flow_path: Option<String>,
    pub(crate) parent_flow_path: Option<String>,
    pub(crate) inside_function: bool,
    pub(crate) inside_choice_content: bool,
    pub(crate) inside_expression: bool,
}

impl VisitContext {
    fn enter_module(&self, module: &Module) -> Self {
        Self {
            current_module: Some(module.name().to_string()),
            current_flow_path: None,
            parent_flow_path: None,
            inside_function: false,
            inside_choice_content: false,
            inside_expression: false,
        }
    }

    fn enter_flow(&self, flow: &Flow) -> Self {
        let current_flow_path = self
            .current_flow_path
            .as_ref()
            .map(|parent| format!("{parent}.{}", flow.name()))
            .unwrap_or_else(|| flow.name().to_string());
        Self {
            current_module: self.current_module.clone(),
            current_flow_path: Some(current_flow_path),
            parent_flow_path: self.current_flow_path.clone(),
            inside_function: self.inside_function || flow.is_function(),
            inside_choice_content: self.inside_choice_content,
            inside_expression: self.inside_expression,
        }
    }

    fn enter_choice_content(&self) -> Self {
        Self {
            inside_choice_content: true,
            ..self.clone()
        }
    }

    fn enter_expression(&self) -> Self {
        Self {
            inside_expression: true,
            ..self.clone()
        }
    }
}

pub(crate) fn walk_story<V>(story: &Story, visitor: &mut V)
where
    V: ParsedVisitor + ?Sized,
{
    let context = VisitContext::default();
    visitor.visit_story(story, &context);
    walk_weave(story.root_weave(), visitor, &context);
    for flow in story.flows() {
        walk_flow(flow, visitor, &context);
    }
    for module in story.modules() {
        walk_module(module, visitor, &context);
    }
}

fn walk_module<V>(module: &Module, visitor: &mut V, parent_context: &VisitContext)
where
    V: ParsedVisitor + ?Sized,
{
    let context = parent_context.enter_module(module);
    visitor.visit_module(module, &context);
    for import in module.imports() {
        visitor.visit_import(import, &context);
    }
    walk_weave(module.weave(), visitor, &context);
    for flow in module.flows() {
        walk_flow(flow, visitor, &context);
    }
}

fn walk_flow<V>(flow: &Flow, visitor: &mut V, parent_context: &VisitContext)
where
    V: ParsedVisitor + ?Sized,
{
    let context = parent_context.enter_flow(flow);
    visitor.visit_flow(flow, &context);
    walk_weave(flow.weave(), visitor, &context);
    for child in flow.child_flows() {
        walk_flow(child, visitor, &context);
    }
}

pub(crate) fn walk_weave<V>(weave: &Weave, visitor: &mut V, context: &VisitContext)
where
    V: ParsedVisitor + ?Sized,
{
    visitor.visit_weave(weave, context);
    for object in weave.content() {
        walk_object(object, visitor, context);
    }
}

fn walk_content_list<V>(content: &ContentList, visitor: &mut V, context: &VisitContext)
where
    V: ParsedVisitor + ?Sized,
{
    visitor.visit_content_list(content, context);
    for object in content.objects() {
        walk_object(object, visitor, context);
    }
}

fn walk_object<V>(object: &Object, visitor: &mut V, context: &VisitContext)
where
    V: ParsedVisitor + ?Sized,
{
    visitor.visit_object(object, context);
    match object {
        Object::ContentList(content) => walk_content_list(content, visitor, context),
        Object::Expression(expression) | Object::LogicLine(expression) => {
            walk_expression(expression, visitor, context)
        }
        Object::Conditional(conditional) => walk_conditional(conditional, visitor, context),
        Object::ConstantDeclaration(declaration) => {
            walk_expression(declaration.expression(), visitor, context)
        }
        Object::IncDec(inc_dec) => walk_expression(inc_dec.expression(), visitor, context),
        Object::Choice(choice) => walk_choice(choice, visitor, context),
        Object::Divert(divert) => {
            for argument in divert.arguments() {
                walk_expression(argument, visitor, context);
            }
        }
        Object::Return(ret) => {
            if let Some(expression) = ret.returned_expression() {
                walk_expression(expression, visitor, context);
            }
        }
        Object::TunnelOnwards(tunnel_onwards) => {
            for argument in tunnel_onwards.arguments() {
                walk_expression(argument, visitor, context);
            }
        }
        Object::VariableAssignment(assignment) => {
            if let Some(expression) = assignment.expression() {
                walk_expression(expression, visitor, context);
            }
        }
        Object::Weave(weave) => walk_weave(weave, visitor, context),
        Object::AuthorWarning(_)
        | Object::EnumDeclaration(_)
        | Object::ExternalDeclaration(_)
        | Object::Gather(_)
        | Object::Glue(_)
        | Object::StructDeclaration(_)
        | Object::Tag(_)
        | Object::Text(_) => {}
    }
}

fn walk_choice<V>(choice: &Choice, visitor: &mut V, context: &VisitContext)
where
    V: ParsedVisitor + ?Sized,
{
    if let Some(condition) = choice.condition() {
        walk_expression(condition, visitor, context);
    }
    let choice_content_context = context.enter_choice_content();
    if let Some(content) = choice.start_content() {
        walk_content_list(content, visitor, &choice_content_context);
    }
    walk_content_list(choice.inner_content(), visitor, &choice_content_context);
}

fn walk_conditional<V>(conditional: &Conditional, visitor: &mut V, context: &VisitContext)
where
    V: ParsedVisitor + ?Sized,
{
    if let Some(condition) = conditional.initial_condition() {
        walk_expression(condition, visitor, context);
    }
    for branch in conditional.branches() {
        if let Some(condition) = branch.own_condition() {
            walk_expression(condition, visitor, context);
        }
        walk_weave(branch.content(), visitor, context);
    }
}

fn walk_expression<V>(expression: &Expression, visitor: &mut V, context: &VisitContext)
where
    V: ParsedVisitor + ?Sized,
{
    visitor.visit_expression(expression, context);
    let expression_context = context.enter_expression();
    match expression {
        Expression::StringContent(content) => {
            walk_content_list(content, visitor, &expression_context)
        }
        Expression::FunctionCall { args, .. } | Expression::QualifiedFunctionCall { args, .. } => {
            for argument in args {
                walk_expression(argument, visitor, &expression_context);
            }
        }
        Expression::ArrayLiteral(elements) => {
            for element in elements {
                walk_expression(element, visitor, &expression_context);
            }
        }
        Expression::StructLiteral(fields) => {
            for field in fields {
                walk_expression(field.expression(), visitor, &expression_context);
            }
        }
        Expression::FieldAccess { base, .. } => walk_expression(base, visitor, &expression_context),
        Expression::IndexAccess { base, index } => {
            walk_expression(base, visitor, &expression_context);
            walk_expression(index, visitor, &expression_context);
        }
        Expression::Binary { left, right, .. } => {
            walk_expression(left, visitor, &expression_context);
            walk_expression(right, visitor, &expression_context);
        }
        Expression::Unary { expression, .. } => {
            walk_expression(expression, visitor, &expression_context)
        }
        Expression::MultipleCondition(expressions) => {
            for expression in expressions {
                walk_expression(expression, visitor, &expression_context);
            }
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

#[cfg(test)]
mod tests {
    use crate::{
        parsed::{
            BinaryOperator, ConditionalBranch, FlowLevel, FlowParts, Gather, Text, UnaryOperator,
        },
        source::SourceSpan,
    };

    use super::*;

    type RecordedContext = (Option<String>, Option<String>, Option<String>, bool);

    #[derive(Default)]
    struct RecordingVisitor {
        saw_story: bool,
        modules: Vec<String>,
        imports: Vec<String>,
        flows: Vec<String>,
        module_contexts: Vec<Option<String>>,
        import_contexts: Vec<Option<String>>,
        flow_contexts: Vec<RecordedContext>,
        story_context: Option<RecordedContext>,
        weave_count: usize,
        content_list_count: usize,
        objects: Vec<&'static str>,
        expressions: Vec<&'static str>,
    }

    impl ParsedVisitor for RecordingVisitor {
        fn visit_story(&mut self, _story: &Story, context: &VisitContext) {
            self.saw_story = true;
            self.story_context = Some((
                context.current_module.clone(),
                context.current_flow_path.clone(),
                context.parent_flow_path.clone(),
                context.inside_function,
            ));
        }

        fn visit_module(&mut self, module: &Module, context: &VisitContext) {
            self.modules.push(module.name().to_string());
            self.module_contexts.push(context.current_module.clone());
        }

        fn visit_import(&mut self, import: &ImportDeclaration, context: &VisitContext) {
            self.imports.push(format!(
                "{}::{}",
                import.source_module(),
                import
                    .imported_names()
                    .iter()
                    .map(|name| name.name())
                    .collect::<Vec<_>>()
                    .join(",")
            ));
            self.import_contexts.push(context.current_module.clone());
        }

        fn visit_flow(&mut self, flow: &Flow, context: &VisitContext) {
            self.flows.push(flow.name().to_string());
            self.flow_contexts.push((
                context.current_module.clone(),
                context.current_flow_path.clone(),
                context.parent_flow_path.clone(),
                context.inside_function,
            ));
        }

        fn visit_weave(&mut self, _weave: &Weave, _context: &VisitContext) {
            self.weave_count += 1;
        }

        fn visit_content_list(&mut self, _content: &ContentList, _context: &VisitContext) {
            self.content_list_count += 1;
        }

        fn visit_object(&mut self, object: &Object, _context: &VisitContext) {
            self.objects.push(match object {
                Object::Choice(_) => "choice",
                Object::Conditional(_) => "conditional",
                Object::Gather(_) => "gather",
                Object::Weave(_) => "weave",
                Object::Expression(_) => "expression",
                Object::Text(_) => "text",
                _ => "other",
            });
        }

        fn visit_expression(&mut self, expression: &Expression, _context: &VisitContext) {
            self.expressions.push(match expression {
                Expression::ArrayLiteral(_) => "array",
                Expression::Binary { .. } => "binary",
                Expression::DivertTarget(_) => "divert_target",
                Expression::FieldAccess { .. } => "field_access",
                Expression::IndexAccess { .. } => "index_access",
                Expression::FunctionCall { .. } => "function_call",
                Expression::MultipleCondition(_) => "multiple_condition",
                Expression::StructLiteral(_) => "struct",
                Expression::StringContent(_) => "string_content",
                Expression::Unary { .. } => "unary",
                Expression::VariableReference(_) => "variable",
                _ => "literal",
            });
        }
    }

    #[test]
    fn walks_story_flows_and_nested_parsed_content() {
        let story = Story::new(
            vec![
                Object::Choice(choice_with_all_content()),
                Object::Conditional(conditional_with_branch_content()),
                Object::Expression(Expression::StringContent(ContentList::new(vec![
                    Object::Expression(Expression::Unary {
                        operator: UnaryOperator::Not,
                        expression: Box::new(Expression::VariableReference(
                            "inside_string".to_string(),
                        )),
                    }),
                ]))),
                Object::Expression(Expression::ArrayLiteral(vec![
                    Expression::VariableReference("array_value".to_string()),
                    Expression::FunctionCall {
                        name: "array_call".to_string(),
                        args: Vec::new(),
                    },
                ])),
                Object::Expression(Expression::StructLiteral(vec![
                    crate::parsed::StructLiteralField::new(
                        "struct_value",
                        Expression::VariableReference("field_value".to_string()),
                    ),
                    crate::parsed::StructLiteralField::new(
                        "struct_call",
                        Expression::FunctionCall {
                            name: "field_call".to_string(),
                            args: Vec::new(),
                        },
                    ),
                ])),
                Object::Expression(Expression::FieldAccess {
                    base: Box::new(Expression::VariableReference("field_base".to_string())),
                    field: "hp".to_string(),
                }),
                Object::Expression(Expression::IndexAccess {
                    base: Box::new(Expression::VariableReference("items".to_string())),
                    index: Box::new(Expression::NumberInt(0)),
                }),
                Object::Weave(Weave::new(vec![Object::Gather(Gather::new(span(), 1))], 1)),
            ],
            vec![Flow::from_parts(
                FlowParts::new(
                    FlowLevel::Knot,
                    "knot",
                    vec![Object::Expression(Expression::DivertTarget(
                        "target".to_string(),
                    ))],
                )
                .child_flows(vec![Flow::from_parts(
                    FlowParts::new(
                        FlowLevel::Stitch,
                        "stitch",
                        vec![Object::Text(Text::new("nested", span()))],
                    )
                    .function(true),
                )]),
            )],
        );

        let mut visitor = RecordingVisitor::default();
        walk_story(&story, &mut visitor);

        assert!(visitor.saw_story);
        assert_eq!(visitor.story_context, Some((None, None, None, false)));
        assert_eq!(visitor.flows, vec!["knot", "stitch"]);
        assert_eq!(
            visitor.flow_contexts,
            vec![
                (None, Some("knot".to_string()), None, false),
                (
                    None,
                    Some("knot.stitch".to_string()),
                    Some("knot".to_string()),
                    true
                ),
            ]
        );
        assert!(
            visitor.weave_count >= 5,
            "root, conditional branch, nested object weave, knot, and stitch weaves should be visited"
        );
        assert!(
            visitor.content_list_count >= 3,
            "choice content and string expression content should be visited"
        );

        for expected in ["choice", "conditional", "gather", "weave"] {
            assert!(
                visitor.objects.contains(&expected),
                "missing visited object kind: {expected}"
            );
        }

        for expected in [
            "array",
            "binary",
            "divert_target",
            "field_access",
            "index_access",
            "function_call",
            "multiple_condition",
            "struct",
            "string_content",
            "unary",
            "variable",
        ] {
            assert!(
                visitor.expressions.contains(&expected),
                "missing visited expression kind: {expected}"
            );
        }
    }

    #[test]
    fn walks_modules_imports_and_module_owned_flows() {
        let story = Story::new_with_modules(
            Vec::new(),
            Vec::new(),
            vec![Module::new(
                "game",
                vec![ImportDeclaration::new(
                    vec![
                        crate::parsed::ImportedName::new("sword", span()),
                        crate::parsed::ImportedName::new("heal", span()),
                    ],
                    "items",
                    span(),
                    span(),
                )],
                vec![Object::Text(Text::new("module text", span()))],
                vec![Flow::from_parts(FlowParts::new(
                    FlowLevel::Knot,
                    "main",
                    Vec::new(),
                ))],
                span(),
                span(),
            )],
        );

        let mut visitor = RecordingVisitor::default();
        walk_story(&story, &mut visitor);

        assert_eq!(visitor.modules, vec!["game"]);
        assert_eq!(visitor.module_contexts, vec![Some("game".to_string())]);
        assert_eq!(visitor.imports, vec!["items::sword,heal"]);
        assert_eq!(visitor.import_contexts, vec![Some("game".to_string())]);
        assert_eq!(visitor.flows, vec!["main"]);
        assert_eq!(
            visitor.flow_contexts,
            vec![(
                Some("game".to_string()),
                Some("main".to_string()),
                None,
                false
            )]
        );
        assert!(visitor.objects.contains(&"text"));
    }

    fn choice_with_all_content() -> Choice {
        let mut choice = Choice::new(
            Some(ContentList::from_text("start", span())),
            ContentList::new(vec![Object::Expression(Expression::MultipleCondition(
                vec![
                    Expression::VariableReference("a".to_string()),
                    Expression::VariableReference("b".to_string()),
                ],
            ))]),
            span(),
        );
        choice.set_condition(Some(Expression::Binary {
            operator: BinaryOperator::Equals,
            left: Box::new(Expression::VariableReference("flag".to_string())),
            right: Box::new(Expression::NumberBool(true)),
        }));
        choice
    }

    fn conditional_with_branch_content() -> Conditional {
        Conditional::if_block(
            Some(Expression::VariableReference("outer_condition".to_string())),
            vec![ConditionalBranch::new(
                true,
                false,
                false,
                Weave::new(vec![Object::Expression(Expression::NumberInt(7))], 0),
                Some(Expression::VariableReference(
                    "branch_condition".to_string(),
                )),
            )],
        )
    }

    fn span() -> SourceSpan {
        SourceSpan::new(None, 1, 1)
    }
}
