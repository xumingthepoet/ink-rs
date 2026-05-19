use std::collections::{BTreeMap, BTreeSet};

use crate::{
    diagnostic::Diagnostic,
    parsed::{
        ContentList, ExternalDeclaration, Flow, InterfaceDeclaration, Object, Story, TypeName,
        Weave,
    },
    source::SourceSpan,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InterfaceSymbol {
    name: String,
    span: SourceSpan,
}

#[derive(Debug, Clone)]
struct NamedDeclaration {
    kind: &'static str,
    name: String,
}

pub(super) type InterfaceIndex = BTreeMap<String, InterfaceSymbol>;

pub(super) fn interface_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let (index, mut diagnostics) = build_interface_index_with_diagnostics(story);
    diagnostics.extend(interface_name_conflict_diagnostics(
        story.interfaces(),
        &collect_non_interface_declarations(story),
    ));
    diagnostics.extend(unknown_interface_type_diagnostics(story, &index));
    sort_diagnostics(&mut diagnostics);
    diagnostics
}

#[cfg(test)]
fn build_interface_index(story: &Story) -> InterfaceIndex {
    build_interface_index_with_diagnostics(story).0
}

fn build_interface_index_with_diagnostics(story: &Story) -> (InterfaceIndex, Vec<Diagnostic>) {
    let mut index = InterfaceIndex::new();
    let mut diagnostics = Vec::new();

    for declaration in story.interfaces() {
        if index.contains_key(declaration.name()) {
            diagnostics.push(Diagnostic::error(
                declaration.name_span().clone(),
                format!(
                    "Interface '{}' is already declared in this compilation",
                    declaration.name()
                ),
            ));
        } else {
            index.insert(
                declaration.name().to_string(),
                InterfaceSymbol {
                    name: declaration.name().to_string(),
                    span: declaration.name_span().clone(),
                },
            );
        }
    }

    (index, diagnostics)
}

fn interface_name_conflict_diagnostics(
    interfaces: &[InterfaceDeclaration],
    declarations: &[NamedDeclaration],
) -> Vec<Diagnostic> {
    let mut declaration_names = BTreeMap::<&str, Vec<&NamedDeclaration>>::new();
    for declaration in declarations {
        declaration_names
            .entry(declaration.name.as_str())
            .or_default()
            .push(declaration);
    }
    let mut diagnostics = Vec::new();
    let mut reported = BTreeSet::new();

    for interface in interfaces {
        let Some(conflicting_declarations) = declaration_names.get(interface.name()) else {
            continue;
        };
        for declaration in conflicting_declarations {
            if !reported.insert((interface.name().to_string(), declaration.kind)) {
                continue;
            }
            diagnostics.push(Diagnostic::error(
                interface.name_span().clone(),
                format!(
                    "Interface '{}' conflicts with {} '{}'",
                    interface.name(),
                    declaration.kind,
                    declaration.name
                ),
            ));
        }
    }

    diagnostics
}

fn unknown_interface_type_diagnostics(story: &Story, index: &InterfaceIndex) -> Vec<Diagnostic> {
    let mut references = Vec::new();
    collect_interface_type_references(story, &mut references);
    references
        .into_iter()
        .filter(|reference| !index.contains_key(reference.name.as_str()))
        .map(|reference| {
            Diagnostic::error(
                reference.span,
                format!("Unknown interface type '{}'", reference.name),
            )
        })
        .collect()
}

#[derive(Debug, Clone)]
struct InterfaceTypeReference {
    name: String,
    span: SourceSpan,
}

fn collect_interface_type_references(story: &Story, references: &mut Vec<InterfaceTypeReference>) {
    collect_interface_type_references_in_weave(story.root_weave(), references);
    for flow in story.flows() {
        collect_interface_type_references_in_flow(flow, references);
    }
    for interface in story.interfaces() {
        for member in interface.members() {
            for argument in member.arguments() {
                if let Some(type_name) = argument.declared_type() {
                    collect_interface_type_references_in_type_name(type_name, references);
                }
            }
            if let Some(return_type) = member.return_type() {
                collect_interface_type_references_in_type_name(return_type, references);
            }
        }
    }
    for module in story.modules() {
        collect_interface_type_references_in_weave(module.weave(), references);
        for flow in module.flows() {
            collect_interface_type_references_in_flow(flow, references);
        }
    }
}

fn collect_interface_type_references_in_flow(
    flow: &Flow,
    references: &mut Vec<InterfaceTypeReference>,
) {
    for argument in flow.arguments() {
        if let Some(type_name) = argument.declared_type() {
            collect_interface_type_references_in_type_name(type_name, references);
        }
    }
    collect_interface_type_references_in_type_name(flow.return_type(), references);
    collect_interface_type_references_in_weave(flow.weave(), references);
    for child in flow.child_flows() {
        collect_interface_type_references_in_flow(child, references);
    }
}

fn collect_interface_type_references_in_weave(
    weave: &Weave,
    references: &mut Vec<InterfaceTypeReference>,
) {
    collect_interface_type_references_in_objects(weave.content(), references);
}

fn collect_interface_type_references_in_content_list(
    content: &ContentList,
    references: &mut Vec<InterfaceTypeReference>,
) {
    collect_interface_type_references_in_objects(content.objects(), references);
}

fn collect_interface_type_references_in_objects(
    objects: &[Object],
    references: &mut Vec<InterfaceTypeReference>,
) {
    for object in objects {
        collect_interface_type_references_in_object(object, references);
    }
}

fn collect_interface_type_references_in_object(
    object: &Object,
    references: &mut Vec<InterfaceTypeReference>,
) {
    match object {
        Object::ConstantDeclaration(declaration) => {
            collect_interface_type_references_in_type_name(declaration.declared_type(), references);
        }
        Object::ExternalDeclaration(external) => {
            collect_interface_type_references_in_external(external, references);
        }
        Object::StructDeclaration(declaration) => {
            for field in declaration.fields() {
                collect_interface_type_references_in_type_name(field.type_name(), references);
            }
        }
        Object::VariableAssignment(assignment) => {
            if let Some(type_name) = assignment.declared_type() {
                collect_interface_type_references_in_type_name(type_name, references);
            }
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_interface_type_references_in_content_list(content, references);
            }
            collect_interface_type_references_in_content_list(choice.inner_content(), references);
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_interface_type_references_in_weave(branch.content(), references);
            }
        }
        Object::ContentList(content) => {
            collect_interface_type_references_in_content_list(content, references)
        }
        Object::Weave(weave) => collect_interface_type_references_in_weave(weave, references),
        Object::AuthorWarning(_)
        | Object::Divert(_)
        | Object::EnumDeclaration(_)
        | Object::Expression(_)
        | Object::Gather(_)
        | Object::Glue(_)
        | Object::IncDec(_)
        | Object::LogicLine(_)
        | Object::Return(_)
        | Object::Tag(_)
        | Object::Text(_)
        | Object::TunnelOnwards(_) => {}
    }
}

fn collect_interface_type_references_in_external(
    external: &ExternalDeclaration,
    references: &mut Vec<InterfaceTypeReference>,
) {
    for type_name in external.argument_types() {
        collect_interface_type_references_in_type_name(type_name, references);
    }
    collect_interface_type_references_in_type_name(external.return_type(), references);
}

fn collect_interface_type_references_in_type_name(
    type_name: &TypeName,
    references: &mut Vec<InterfaceTypeReference>,
) {
    match type_name {
        TypeName::Interface {
            name, name_span, ..
        } => references.push(InterfaceTypeReference {
            name: name.clone(),
            span: name_span.clone(),
        }),
        TypeName::Array(element_type) => {
            collect_interface_type_references_in_type_name(element_type, references)
        }
        TypeName::Primitive(_)
        | TypeName::Struct(_)
        | TypeName::QualifiedStruct(_)
        | TypeName::Void => {}
    }
}

fn collect_non_interface_declarations(story: &Story) -> Vec<NamedDeclaration> {
    let mut declarations = Vec::new();
    for module in story.modules() {
        declarations.push(NamedDeclaration {
            kind: "module",
            name: module.name().to_string(),
        });
    }
    collect_named_declarations_in_weave(story.root_weave(), &mut declarations);
    for flow in story.flows() {
        collect_named_declarations_in_flow(flow, &mut declarations);
    }
    for module in story.modules() {
        collect_named_declarations_in_weave(module.weave(), &mut declarations);
        for flow in module.flows() {
            collect_named_declarations_in_flow(flow, &mut declarations);
        }
    }
    declarations
}

fn collect_named_declarations_in_flow(flow: &Flow, declarations: &mut Vec<NamedDeclaration>) {
    declarations.push(NamedDeclaration {
        kind: if flow.is_function() {
            "function"
        } else {
            "flow"
        },
        name: flow.name().to_string(),
    });
    collect_named_declarations_in_weave(flow.weave(), declarations);
    for child in flow.child_flows() {
        collect_named_declarations_in_flow(child, declarations);
    }
}

fn collect_named_declarations_in_weave(weave: &Weave, declarations: &mut Vec<NamedDeclaration>) {
    collect_named_declarations_in_objects(weave.content(), declarations);
}

fn collect_named_declarations_in_content_list(
    content: &ContentList,
    declarations: &mut Vec<NamedDeclaration>,
) {
    collect_named_declarations_in_objects(content.objects(), declarations);
}

fn collect_named_declarations_in_objects(
    objects: &[Object],
    declarations: &mut Vec<NamedDeclaration>,
) {
    for object in objects {
        collect_named_declarations_in_object(object, declarations);
    }
}

fn collect_named_declarations_in_object(object: &Object, declarations: &mut Vec<NamedDeclaration>) {
    match object {
        Object::ConstantDeclaration(declaration) => declarations.push(NamedDeclaration {
            kind: "constant",
            name: declaration.name().to_string(),
        }),
        Object::EnumDeclaration(declaration) => declarations.push(NamedDeclaration {
            kind: "enum",
            name: declaration.name().to_string(),
        }),
        Object::ExternalDeclaration(declaration) => declarations.push(NamedDeclaration {
            kind: "external",
            name: declaration.name().to_string(),
        }),
        Object::StructDeclaration(declaration) => declarations.push(NamedDeclaration {
            kind: "struct",
            name: declaration.name().to_string(),
        }),
        Object::VariableAssignment(assignment) if assignment.is_global() => {
            declarations.push(NamedDeclaration {
                kind: "global variable",
                name: assignment.name().to_string(),
            });
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_named_declarations_in_content_list(content, declarations);
            }
            collect_named_declarations_in_content_list(choice.inner_content(), declarations);
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_named_declarations_in_weave(branch.content(), declarations);
            }
        }
        Object::ContentList(content) => {
            collect_named_declarations_in_content_list(content, declarations)
        }
        Object::Weave(weave) => collect_named_declarations_in_weave(weave, declarations),
        Object::AuthorWarning(_)
        | Object::Divert(_)
        | Object::Expression(_)
        | Object::Gather(_)
        | Object::Glue(_)
        | Object::IncDec(_)
        | Object::LogicLine(_)
        | Object::Return(_)
        | Object::Tag(_)
        | Object::Text(_)
        | Object::TunnelOnwards(_)
        | Object::VariableAssignment(_) => {}
    }
}

fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by(|left, right| {
        (
            left.source_filename.as_deref(),
            left.line,
            left.column,
            left.message.as_str(),
        )
            .cmp(&(
                right.source_filename.as_deref(),
                right.line,
                right.column,
                right.message.as_str(),
            ))
    });
}

#[cfg(test)]
mod tests {
    use crate::diagnostic::DiagnosticSeverity;

    use super::{
        super::test_support::{assert_single_diagnostic, parse_story},
        *,
    };

    #[test]
    fn indexes_interface_declarations() {
        let story = parse_story("=== interface IItem ===\n=== interface IRoute ===");

        let index = build_interface_index(&story);

        assert!(index.contains_key("IItem"));
        assert!(index.contains_key("IRoute"));
    }

    #[test]
    fn reports_duplicate_interface_names() {
        let story = parse_story("=== interface IItem ===\n=== interface IItem ===");

        let diagnostics = interface_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Interface 'IItem' is already declared in this compilation",
        );
    }

    #[test]
    fn reports_unknown_interface_type_references() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == function score(route: interface<IMissing>) => int ==\n\
             === module game ===\n\
             VAR route: interface<IOther> = 0",
        );

        let diagnostics = interface_diagnostics(&story);

        assert_eq!(diagnostics.len(), 2, "{diagnostics:#?}");
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic.message == "Unknown interface type 'IMissing'"
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic.message == "Unknown interface type 'IOther'"
        }));
    }

    #[test]
    fn reports_interface_conflicts_with_module_names() {
        let story = parse_story("=== interface IItem ===\n=== module IItem ===");

        let diagnostics = interface_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Interface 'IItem' conflicts with module 'IItem'",
        );
    }
}
