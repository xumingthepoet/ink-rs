use std::collections::HashSet;

use crate::{
    diagnostic::{Diagnostic, DiagnosticCode},
    parsed::{
        AssignmentTarget, Choice, Conditional, ConditionalBranch, ContentList, DictLiteralEntry,
        Divert, DivertTarget, DynamicChoiceBinding, Expression, ForLoop, ForLoopVariable, IncDec,
        Object, Return, StructLiteralField, TunnelOnwards, VariableAssignment, Weave,
    },
    source::{SourceLine, SourceSpan},
};

use super::{
    is_identifier, knot, parse_initial_expression, parser::Parser, scan, weave::weave_from_objects,
};

#[derive(Clone)]
pub(super) struct LoopAlias {
    pub(super) source_name: String,
    pub(super) runtime_name: String,
}

pub(super) fn rewrite_objects(objects: Vec<Object>, aliases: &[LoopAlias]) -> Vec<Object> {
    if aliases.is_empty() {
        return objects;
    }

    objects
        .into_iter()
        .map(|object| rewrite_object(object, aliases))
        .collect()
}

pub(super) fn rewrite_expression(expression: Expression, aliases: &[LoopAlias]) -> Expression {
    if aliases.is_empty() {
        return expression;
    }

    match expression {
        Expression::VariableReference(name) => alias_for(&name, aliases)
            .map_or(Expression::VariableReference(name), |alias| {
                Expression::VariableReference(alias)
            }),
        Expression::FunctionCall { name, args } => Expression::FunctionCall {
            name,
            args: args
                .into_iter()
                .map(|arg| rewrite_expression(arg, aliases))
                .collect(),
        },
        Expression::QualifiedFunctionCall { name, args } => Expression::QualifiedFunctionCall {
            name,
            args: args
                .into_iter()
                .map(|arg| rewrite_expression(arg, aliases))
                .collect(),
        },
        Expression::DynamicInterfaceAccess { target, member } => {
            Expression::DynamicInterfaceAccess {
                target: Box::new(rewrite_expression(*target, aliases)),
                member,
            }
        }
        Expression::DynamicInterfaceFunctionCall {
            target,
            member,
            args,
        } => Expression::DynamicInterfaceFunctionCall {
            target: Box::new(rewrite_expression(*target, aliases)),
            member,
            args: args
                .into_iter()
                .map(|arg| rewrite_expression(arg, aliases))
                .collect(),
        },
        Expression::ArrayLiteral(elements) => Expression::ArrayLiteral(
            elements
                .into_iter()
                .map(|element| rewrite_expression(element, aliases))
                .collect(),
        ),
        Expression::StructLiteral { type_name, fields } => Expression::StructLiteral {
            type_name,
            fields: fields
                .into_iter()
                .map(|field| {
                    StructLiteralField::new(
                        field.name().to_string(),
                        rewrite_expression(field.expression().clone(), aliases),
                    )
                })
                .collect(),
        },
        Expression::DictLiteral(entries) => Expression::DictLiteral(
            entries
                .into_iter()
                .map(|entry| {
                    DictLiteralEntry::new(
                        entry.key().clone(),
                        rewrite_expression(entry.value().clone(), aliases),
                    )
                })
                .collect(),
        ),
        Expression::FieldAccess { base, field } => Expression::FieldAccess {
            base: Box::new(rewrite_expression(*base, aliases)),
            field,
        },
        Expression::IndexAccess { base, index } => Expression::IndexAccess {
            base: Box::new(rewrite_expression(*base, aliases)),
            index: Box::new(rewrite_expression(*index, aliases)),
        },
        Expression::Binary {
            operator,
            left,
            right,
        } => Expression::Binary {
            operator,
            left: Box::new(rewrite_expression(*left, aliases)),
            right: Box::new(rewrite_expression(*right, aliases)),
        },
        Expression::Unary {
            operator,
            expression,
        } => Expression::Unary {
            operator,
            expression: Box::new(rewrite_expression(*expression, aliases)),
        },
        Expression::MultipleCondition(expressions) => Expression::MultipleCondition(
            expressions
                .into_iter()
                .map(|expression| rewrite_expression(expression, aliases))
                .collect(),
        ),
        Expression::StringContent(content) => {
            Expression::StringContent(rewrite_content_list(content, aliases))
        }
        Expression::String(_)
        | Expression::NumberInt(_)
        | Expression::NumberFloat(_)
        | Expression::NumberBool(_)
        | Expression::DivertTarget(_)
        | Expression::QualifiedReference(_) => expression,
    }
}

impl Parser {
    pub(super) fn parse_multiline_for_loop(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<Vec<Object>> {
        let line = &lines[*index];
        let trimmed = line.text.trim();
        let after_open = trimmed.strip_prefix('{')?.trim();
        if !is_for_header_start(after_open) {
            return None;
        }

        let id = self.next_for_loop_id();
        let aliases = self.loop_aliases().to_vec();
        let Some(header) = parse_for_header(after_open, &line.span, &aliases, id) else {
            self.diagnostics.push(
                Diagnostic::error(
                    line.span.clone(),
                    "Invalid for loop header. Use `{ for item in items: }`, `{ for index, item in items: }`, or `{ for key, value in dict: }`.",
                )
                .with_code(DiagnosticCode::InvalidInlineSyntax),
            );
            skip_invalid_for_loop(lines, index);
            return Some(Vec::new());
        };

        *index += 1;
        let aliases = header
            .variables
            .iter()
            .map(|variable| LoopAlias {
                source_name: variable.source_name().to_string(),
                runtime_name: variable.runtime_name().to_string(),
            })
            .collect::<Vec<_>>();
        self.push_loop_aliases(aliases);

        let mut body = Vec::new();
        while *index < lines.len() {
            let current_line = &lines[*index];
            let current_trimmed = current_line.text.trim();

            if current_trimmed.starts_with('}') {
                *index += 1;
                self.pop_loop_aliases();
                return Some(vec![Object::ForLoop(ForLoop::new(
                    header.id,
                    header.variables,
                    header.iterable,
                    weave_from_objects(body),
                    line.span.clone(),
                ))]);
            }

            if current_trimmed.starts_with('{') {
                if let Some(nested) = self.parse_compound_statement(lines, index) {
                    body.extend(nested);
                    continue;
                }
            }

            if is_flow_declaration_line(current_trimmed) {
                self.diagnostics.push(
                    Diagnostic::error(
                        current_line.span.clone(),
                        "Flow declarations are not allowed inside for loops.",
                    )
                    .with_code(DiagnosticCode::InvalidInlineSyntax),
                );
                *index += 1;
                continue;
            }

            body.extend(self.parse_statement(current_line));
            *index += 1;
        }

        self.pop_loop_aliases();
        self.diagnostics.push(
            Diagnostic::error(line.span.clone(), "Expected closing `}` for for loop")
                .with_code(DiagnosticCode::InvalidInlineSyntax),
        );
        Some(Vec::new())
    }
}

struct ParsedForHeader {
    id: usize,
    variables: Vec<ForLoopVariable>,
    iterable: Expression,
}

fn is_for_header_start(source: &str) -> bool {
    let Some(after_for) = source.strip_prefix("for") else {
        return false;
    };
    !after_for
        .chars()
        .next()
        .is_some_and(super::is_identifier_continue)
}

fn skip_invalid_for_loop(lines: &[SourceLine], index: &mut usize) {
    *index += 1;
    while *index < lines.len() {
        let trimmed = lines[*index].text.trim();
        *index += 1;
        if trimmed.starts_with('}') {
            break;
        }
    }
}

fn parse_for_header(
    source: &str,
    span: &SourceSpan,
    aliases: &[LoopAlias],
    id: usize,
) -> Option<ParsedForHeader> {
    let header = source.strip_suffix(':')?.trim();
    let after_for = header.strip_prefix("for")?;
    if after_for
        .chars()
        .next()
        .is_some_and(super::is_identifier_continue)
    {
        return None;
    }
    let after_for = after_for.trim_start();
    let (variables_source, iterable_source) = split_for_header(after_for)?;
    let variable_names =
        scan::split_top_level_with_options(variables_source, ',', scan::ScanOptions::expression());
    if variable_names.is_empty() {
        return None;
    }
    if !variable_names.iter().all(|name| is_identifier(name.trim())) {
        return None;
    }
    if has_duplicate_variable_names(&variable_names) {
        return None;
    }

    let iterable = rewrite_expression(parse_initial_expression(iterable_source.trim())?, aliases);
    let variables = variable_names
        .into_iter()
        .enumerate()
        .map(|(index, name)| ForLoopVariable::new(name.trim(), format!("$for{id}_v{index}")))
        .collect();

    let _ = span;
    Some(ParsedForHeader {
        id,
        variables,
        iterable,
    })
}

fn split_for_header(source: &str) -> Option<(&str, &str)> {
    for (index, token) in scan::top_level_token_matches_with_options(
        source,
        &[" in "],
        scan::ScanOptions::expression(),
    ) {
        if token == " in " {
            return Some((&source[..index], &source[index + token.len()..]));
        }
    }
    None
}

fn has_duplicate_variable_names(variable_names: &[&str]) -> bool {
    let mut seen = HashSet::new();
    variable_names.iter().any(|name| !seen.insert(name.trim()))
}

fn is_flow_declaration_line(source: &str) -> bool {
    knot::is_knot_declaration_line(source) || knot::is_stitch_declaration_line(source)
}

fn alias_for(name: &str, aliases: &[LoopAlias]) -> Option<String> {
    aliases
        .iter()
        .rev()
        .find(|alias| alias.source_name == name)
        .map(|alias| alias.runtime_name.clone())
}

pub(super) fn rewrite_object(object: Object, aliases: &[LoopAlias]) -> Object {
    match object {
        Object::ContentList(content) => Object::ContentList(rewrite_content_list(content, aliases)),
        Object::Expression(expression) => {
            Object::Expression(rewrite_expression(expression, aliases))
        }
        Object::Conditional(conditional) => {
            Object::Conditional(rewrite_conditional(conditional, aliases))
        }
        Object::ForLoop(for_loop) => Object::ForLoop(rewrite_for_loop(for_loop, aliases)),
        Object::ConstantDeclaration(declaration) => Object::ConstantDeclaration(declaration),
        Object::LogicLine(expression) => Object::LogicLine(rewrite_expression(expression, aliases)),
        Object::IncDec(inc_dec) => Object::IncDec(rewrite_inc_dec(inc_dec, aliases)),
        Object::Choice(choice) => Object::Choice(rewrite_choice(choice, aliases)),
        Object::Divert(divert) => Object::Divert(rewrite_divert(divert, aliases)),
        Object::Return(ret) => Object::Return(rewrite_return(ret, aliases)),
        Object::VariableAssignment(assignment) => {
            Object::VariableAssignment(rewrite_assignment(assignment, aliases))
        }
        Object::TunnelOnwards(tunnel_onwards) => {
            Object::TunnelOnwards(rewrite_tunnel_onwards(tunnel_onwards, aliases))
        }
        Object::Weave(weave) => Object::Weave(Weave::new(
            rewrite_objects(weave.content().to_vec(), aliases),
            weave.base_indent(),
        )),
        Object::Text(_)
        | Object::AuthorWarning(_)
        | Object::Glue(_)
        | Object::EnumDeclaration(_)
        | Object::ExternalDeclaration(_)
        | Object::Gather(_)
        | Object::StructDeclaration(_)
        | Object::Tag(_) => object,
    }
}

pub(super) fn rewrite_content_list(content: ContentList, aliases: &[LoopAlias]) -> ContentList {
    ContentList::new(rewrite_objects(content.into_objects(), aliases))
}

pub(super) fn dynamic_choice_aliases(binding: &DynamicChoiceBinding) -> Vec<LoopAlias> {
    binding
        .variables()
        .iter()
        .map(|variable| LoopAlias {
            source_name: variable.source_name().to_string(),
            runtime_name: variable.runtime_name().to_string(),
        })
        .collect()
}

pub(super) fn rewrite_choice_own_dynamic_aliases(choice: Choice) -> Choice {
    let Some(binding) = choice.dynamic_binding().cloned() else {
        return choice;
    };
    let aliases = dynamic_choice_aliases(&binding);
    rewrite_choice_parts(choice, &aliases, false)
}

fn rewrite_choice(choice: Choice, aliases: &[LoopAlias]) -> Choice {
    rewrite_choice_parts(choice, aliases, true)
}

fn rewrite_choice_parts(
    choice: Choice,
    aliases: &[LoopAlias],
    rewrite_dynamic_iterable: bool,
) -> Choice {
    let start_content = choice
        .start_content()
        .cloned()
        .map(|content| rewrite_content_list(content, aliases));
    let inner_content = rewrite_content_list(choice.inner_content().clone(), aliases);
    let condition = choice
        .condition()
        .cloned()
        .map(|condition| rewrite_expression(condition, aliases));
    let dynamic_binding = choice.dynamic_binding().cloned().map(|binding| {
        if rewrite_dynamic_iterable {
            let iterable = rewrite_expression(binding.iterable().clone(), aliases);
            binding.with_iterable(iterable)
        } else {
            binding
        }
    });

    let mut rewritten = Choice::new(start_content, inner_content, choice.span().clone());
    rewritten.set_identifier(choice.identifier().map(str::to_string));
    rewritten.set_is_invisible_default(choice.is_invisible_default());
    rewritten.set_indentation_depth(choice.indentation_depth());
    rewritten.set_condition(condition);
    rewritten.set_dynamic_binding(dynamic_binding);
    rewritten
}

fn rewrite_for_loop(for_loop: ForLoop, aliases: &[LoopAlias]) -> ForLoop {
    ForLoop::new(
        for_loop.id(),
        for_loop.variables().to_vec(),
        rewrite_expression(for_loop.iterable().clone(), aliases),
        Weave::new(
            rewrite_objects(for_loop.body().content().to_vec(), aliases),
            for_loop.body().base_indent(),
        ),
        for_loop.span().clone(),
    )
}

fn rewrite_conditional(conditional: Conditional, aliases: &[LoopAlias]) -> Conditional {
    let initial_condition = conditional
        .initial_condition()
        .cloned()
        .map(|condition| rewrite_expression(condition, aliases));
    let branches = conditional
        .branches()
        .iter()
        .map(|branch| {
            ConditionalBranch::new(
                branch.is_true_branch(),
                branch.is_else(),
                branch.is_inline(),
                weave_from_objects(rewrite_objects(
                    branch.content().content().to_vec(),
                    aliases,
                )),
                branch
                    .own_condition()
                    .cloned()
                    .map(|condition| rewrite_expression(condition, aliases)),
            )
        })
        .collect();
    Conditional::new(conditional.kind(), initial_condition, branches)
}

fn rewrite_assignment(assignment: VariableAssignment, aliases: &[LoopAlias]) -> VariableAssignment {
    let target = if assignment.is_temporary() || assignment.is_global() {
        assignment.target().clone()
    } else {
        rewrite_assignment_target(assignment.target().clone(), aliases)
    };
    let expression = assignment
        .expression()
        .cloned()
        .map(|expression| rewrite_expression(expression, aliases));
    VariableAssignment::with_target(
        target,
        expression,
        assignment.declared_type().cloned(),
        assignment.is_global(),
        assignment.is_temporary(),
        assignment.span().clone(),
    )
}

fn rewrite_divert(divert: Divert, aliases: &[LoopAlias]) -> Divert {
    let target = rewrite_divert_target(divert.target().clone(), aliases);
    let arguments = divert
        .arguments()
        .iter()
        .cloned()
        .map(|argument| rewrite_expression(argument, aliases))
        .collect();
    let mut rewritten = if divert.has_argument_list() {
        Divert::with_arguments(target, arguments, divert.span().clone())
    } else {
        Divert::new(target, divert.span().clone())
    };
    if divert.is_tunnel() {
        rewritten = rewritten.with_tunnel();
    }
    if divert.is_thread() {
        rewritten = rewritten.with_thread();
    }
    rewritten
}

fn rewrite_divert_target(target: DivertTarget, aliases: &[LoopAlias]) -> DivertTarget {
    match target {
        DivertTarget::Dynamic(expression) => {
            DivertTarget::Dynamic(rewrite_expression(expression, aliases))
        }
        _ => target,
    }
}

fn rewrite_return(ret: Return, aliases: &[LoopAlias]) -> Return {
    Return::new(
        ret.returned_expression()
            .cloned()
            .map(|expression| rewrite_expression(expression, aliases)),
        ret.span().clone(),
    )
}

fn rewrite_tunnel_onwards(tunnel_onwards: TunnelOnwards, aliases: &[LoopAlias]) -> TunnelOnwards {
    let override_target = tunnel_onwards
        .override_target()
        .cloned()
        .map(|target| rewrite_divert_target(target, aliases));
    let arguments = tunnel_onwards
        .arguments()
        .iter()
        .cloned()
        .map(|argument| rewrite_expression(argument, aliases))
        .collect();
    TunnelOnwards::with_arguments(override_target, arguments, tunnel_onwards.span().clone())
}

fn rewrite_inc_dec(inc_dec: IncDec, aliases: &[LoopAlias]) -> IncDec {
    IncDec::with_target(
        rewrite_assignment_target(inc_dec.target().clone(), aliases),
        rewrite_expression(inc_dec.expression().clone(), aliases),
        inc_dec.is_increment(),
        inc_dec.span().clone(),
    )
}

fn rewrite_assignment_target(target: AssignmentTarget, aliases: &[LoopAlias]) -> AssignmentTarget {
    match target {
        AssignmentTarget::Variable(name) => alias_for(&name, aliases)
            .map(AssignmentTarget::Variable)
            .unwrap_or(AssignmentTarget::Variable(name)),
        AssignmentTarget::QualifiedVariable(_) => target,
        AssignmentTarget::FieldAccess { base, field } => AssignmentTarget::FieldAccess {
            base: Box::new(rewrite_assignment_target(*base, aliases)),
            field,
        },
        AssignmentTarget::IndexAccess { base, index } => AssignmentTarget::IndexAccess {
            base: Box::new(rewrite_assignment_target(*base, aliases)),
            index: rewrite_expression(index, aliases),
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parsed::{ConditionalKind, Object},
        source::SourceInput,
        syntax::parse,
    };

    fn parse_ok(source: &str) -> crate::parsed::Story {
        let output = parse(SourceInput::new(source));
        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        output.artifact.expect("parser should produce a story")
    }

    #[test]
    fn parses_array_item_index_and_dict_for_forms() {
        let story = parse_ok(
            "{ for item in items:\nitem {item}\n}\n\
             { for index, item in items:\n{index}: {item}\n}\n\
             { for key, value in scores:\n{key}: {value}\n}",
        );

        let loops = story
            .root_weave()
            .content()
            .iter()
            .filter_map(|object| match object {
                Object::ForLoop(for_loop) => Some(for_loop),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(loops.len(), 3);
        assert_eq!(loops[0].variables().len(), 1);
        assert_eq!(loops[0].variables()[0].source_name(), "item");
        assert_eq!(loops[1].variables().len(), 2);
        assert_eq!(loops[1].variables()[0].source_name(), "index");
        assert_eq!(loops[2].variables().len(), 2);
        assert_eq!(loops[2].variables()[0].source_name(), "key");
    }

    #[test]
    fn parses_conditionals_switches_and_nested_for_in_for_body() {
        let story = parse_ok(
            "{ for row in rows:\n\
             { if row.ready:\nready\n- else: waiting\n}\n\
             { if:\n- row.count > 1: many\n- else: one\n}\n\
             { switch row.kind:\n- 0: zero\n- else: other\n}\n\
             { for item in row.items:\n{item}\n}\n\
             }",
        );

        let Object::ForLoop(for_loop) = &story.root_weave().content()[0] else {
            panic!("expected for loop");
        };
        assert!(for_loop
            .body()
            .content()
            .iter()
            .any(|object| contains_conditional_kind(object, ConditionalKind::If)));
        assert!(for_loop
            .body()
            .content()
            .iter()
            .any(|object| contains_conditional_kind(object, ConditionalKind::Switch)));
        assert!(for_loop.body().content().iter().any(contains_for_loop));
    }

    #[test]
    fn rejects_malformed_for_header() {
        let output = parse(SourceInput::new("{ for item items\nbad\n}"));

        assert_eq!(output.diagnostics.len(), 1, "{:#?}", output.diagnostics);
        assert_eq!(
            output.diagnostics[0].message,
            "Invalid for loop header. Use `{ for item in items: }`, `{ for index, item in items: }`, or `{ for key, value in dict: }`."
        );
    }

    #[test]
    fn rejects_duplicate_for_header_variables() {
        let output = parse(SourceInput::new(
            "=== module game ===\n\
             VAR scores: Dict<string, int> = %{\"ada\": 1}\n\
             == main ==\n\
             { for key, key in scores:\n\
                 {key}\n\
             }",
        ));

        assert_eq!(output.diagnostics.len(), 1, "{:#?}", output.diagnostics);
        assert_eq!(
            output.diagnostics[0].message,
            "Invalid for loop header. Use `{ for item in items: }`, `{ for index, item in items: }`, or `{ for key, value in dict: }`."
        );
    }

    #[test]
    fn rejects_flow_declarations_in_for_body() {
        let output = parse(SourceInput::new(
            "=== module game ===\n\
             VAR values: int[] = [1]\n\
             == main ==\n\
             { for value in values:\n\
                 == bad ==\n\
                 bad\n\
             }",
        ));

        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message
                    == "Flow declarations are not allowed inside for loops."),
            "{:#?}",
            output.diagnostics
        );
    }

    fn contains_conditional_kind(object: &Object, kind: ConditionalKind) -> bool {
        match object {
            Object::Conditional(conditional) => conditional.kind() == kind,
            Object::ContentList(content) => content
                .objects()
                .iter()
                .any(|object| contains_conditional_kind(object, kind)),
            _ => false,
        }
    }

    fn contains_for_loop(object: &Object) -> bool {
        match object {
            Object::ForLoop(_) => true,
            Object::ContentList(content) => content.objects().iter().any(contains_for_loop),
            _ => false,
        }
    }
}
