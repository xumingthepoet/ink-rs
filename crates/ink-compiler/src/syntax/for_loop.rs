use crate::{
    diagnostic::{Diagnostic, DiagnosticCode},
    parsed::{
        AssignmentTarget, Conditional, ConditionalBranch, ContentList, DictLiteralEntry,
        Expression, ForLoop, ForLoopVariable, IncDec, Object, StructLiteralField,
        VariableAssignment,
    },
    source::{SourceLine, SourceSpan},
};

use super::{
    is_identifier, parse_initial_expression, parser::Parser, scan, weave::weave_from_objects,
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

fn alias_for(name: &str, aliases: &[LoopAlias]) -> Option<String> {
    aliases
        .iter()
        .rev()
        .find(|alias| alias.source_name == name)
        .map(|alias| alias.runtime_name.clone())
}

fn rewrite_object(object: Object, aliases: &[LoopAlias]) -> Object {
    match object {
        Object::ContentList(content) => Object::ContentList(rewrite_content_list(content, aliases)),
        Object::Expression(expression) => {
            Object::Expression(rewrite_expression(expression, aliases))
        }
        Object::Conditional(conditional) => {
            Object::Conditional(rewrite_conditional(conditional, aliases))
        }
        Object::ForLoop(for_loop) => Object::ForLoop(for_loop),
        Object::ConstantDeclaration(declaration) => Object::ConstantDeclaration(declaration),
        Object::LogicLine(expression) => Object::LogicLine(rewrite_expression(expression, aliases)),
        Object::IncDec(inc_dec) => Object::IncDec(rewrite_inc_dec(inc_dec, aliases)),
        Object::Return(ret) => Object::Return(ret),
        Object::VariableAssignment(assignment) => {
            Object::VariableAssignment(rewrite_assignment(assignment, aliases))
        }
        Object::Text(_)
        | Object::AuthorWarning(_)
        | Object::Glue(_)
        | Object::Choice(_)
        | Object::Divert(_)
        | Object::EnumDeclaration(_)
        | Object::ExternalDeclaration(_)
        | Object::Gather(_)
        | Object::StructDeclaration(_)
        | Object::Tag(_)
        | Object::TunnelOnwards(_)
        | Object::Weave(_) => object,
    }
}

fn rewrite_content_list(content: ContentList, aliases: &[LoopAlias]) -> ContentList {
    ContentList::new(rewrite_objects(content.into_objects(), aliases))
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
