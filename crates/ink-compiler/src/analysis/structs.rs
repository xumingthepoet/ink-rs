use std::collections::{BTreeMap, BTreeSet};

use crate::{
    diagnostic::Diagnostic,
    parsed::{ContentList, Flow, Object, Sequence, Story, StructDeclaration, TypeName, Weave},
};

use super::context::{StructTypeIndex, StructTypeSymbol};

pub(super) fn struct_type_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let declarations = collect_struct_declarations(story);
    let (index, mut diagnostics) = build_struct_type_index_from_declarations(&declarations);
    diagnostics.extend(unknown_field_type_diagnostics(&declarations, &index));
    diagnostics.extend(recursive_struct_diagnostics(&declarations, &index));
    diagnostics
}

pub(super) fn build_struct_type_index(story: &Story) -> StructTypeIndex {
    let declarations = collect_struct_declarations(story);
    build_struct_type_index_from_declarations(&declarations).0
}

fn build_struct_type_index_from_declarations(
    declarations: &[&StructDeclaration],
) -> (StructTypeIndex, Vec<Diagnostic>) {
    let mut index = StructTypeIndex::new();
    let mut diagnostics = Vec::new();

    for declaration in declarations {
        if index.contains_key(declaration.name()) {
            diagnostics.push(Diagnostic::error(
                declaration.span().clone(),
                format!("Duplicate struct declaration '{}'", declaration.name()),
            ));
            continue;
        }

        let mut seen_fields = BTreeSet::new();
        let mut fields = BTreeMap::new();
        for field in declaration.fields() {
            if !seen_fields.insert(field.name().to_string()) {
                diagnostics.push(Diagnostic::error(
                    field.span().clone(),
                    format!(
                        "Duplicate field '{}' in struct '{}'",
                        field.name(),
                        declaration.name()
                    ),
                ));
                continue;
            }
            fields.insert(field.name().to_string(), field.type_name().clone());
        }

        index.insert(
            declaration.name().to_string(),
            StructTypeSymbol::new(fields),
        );
    }

    (index, diagnostics)
}

fn unknown_field_type_diagnostics(
    declarations: &[&StructDeclaration],
    index: &StructTypeIndex,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for declaration in declarations {
        for field in declaration.fields() {
            for struct_name in referenced_struct_names(field.type_name()) {
                if !index.contains_key(struct_name) {
                    diagnostics.push(Diagnostic::error(
                        field.span().clone(),
                        format!(
                            "Unknown struct type '{}' for field '{}' in struct '{}'",
                            struct_name,
                            field.name(),
                            declaration.name()
                        ),
                    ));
                }
            }
        }
    }

    diagnostics
}

fn recursive_struct_diagnostics(
    declarations: &[&StructDeclaration],
    index: &StructTypeIndex,
) -> Vec<Diagnostic> {
    let adjacency = struct_dependency_graph(index);
    let mut diagnostics = Vec::new();
    let mut reported = BTreeSet::new();

    for declaration in declarations {
        let name = declaration.name();
        if !index.contains_key(name) || reported.contains(name) {
            continue;
        }
        if reaches_struct(name, name, &adjacency, &mut BTreeSet::new()) {
            reported.insert(name.to_string());
            diagnostics.push(Diagnostic::error(
                declaration.span().clone(),
                format!("Recursive struct type '{}' is not supported", name),
            ));
        }
    }

    diagnostics
}

fn struct_dependency_graph(index: &StructTypeIndex) -> BTreeMap<String, BTreeSet<String>> {
    index
        .iter()
        .map(|(name, symbol)| {
            let dependencies = symbol
                .fields()
                .values()
                .flat_map(referenced_struct_names)
                .filter(|referenced| index.contains_key(*referenced))
                .map(ToString::to_string)
                .collect();
            (name.clone(), dependencies)
        })
        .collect()
}

fn reaches_struct(
    target: &str,
    current: &str,
    adjacency: &BTreeMap<String, BTreeSet<String>>,
    visited: &mut BTreeSet<String>,
) -> bool {
    let Some(dependencies) = adjacency.get(current) else {
        return false;
    };
    if !visited.insert(current.to_string()) {
        return false;
    }

    for dependency in dependencies {
        if dependency == target || reaches_struct(target, dependency, adjacency, visited) {
            return true;
        }
    }

    false
}

fn referenced_struct_names(type_name: &TypeName) -> Vec<&str> {
    match type_name {
        TypeName::Struct(name) => vec![name.as_str()],
        TypeName::Array(element_type) => referenced_struct_names(element_type),
        TypeName::Primitive(_) | TypeName::Void => Vec::new(),
    }
}

fn collect_struct_declarations(story: &Story) -> Vec<&StructDeclaration> {
    let mut declarations = Vec::new();
    collect_struct_declarations_in_weave(story.root_weave(), &mut declarations);
    for flow in story.flows() {
        collect_struct_declarations_in_flow(flow, &mut declarations);
    }
    declarations
}

fn collect_struct_declarations_in_flow<'a>(
    flow: &'a Flow,
    declarations: &mut Vec<&'a StructDeclaration>,
) {
    collect_struct_declarations_in_weave(flow.weave(), declarations);
    for child in flow.child_flows() {
        collect_struct_declarations_in_flow(child, declarations);
    }
}

fn collect_struct_declarations_in_content_list<'a>(
    content: &'a ContentList,
    declarations: &mut Vec<&'a StructDeclaration>,
) {
    collect_struct_declarations_in_objects(content.objects(), declarations);
}

fn collect_struct_declarations_in_weave<'a>(
    weave: &'a Weave,
    declarations: &mut Vec<&'a StructDeclaration>,
) {
    collect_struct_declarations_in_objects(weave.content(), declarations);
}

fn collect_struct_declarations_in_objects<'a>(
    objects: &'a [Object],
    declarations: &mut Vec<&'a StructDeclaration>,
) {
    for object in objects {
        collect_struct_declarations_in_object(object, declarations);
    }
}

fn collect_struct_declarations_in_object<'a>(
    object: &'a Object,
    declarations: &mut Vec<&'a StructDeclaration>,
) {
    match object {
        Object::StructDeclaration(declaration) => declarations.push(declaration),
        Object::ContentList(content) => {
            collect_struct_declarations_in_content_list(content, declarations)
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_struct_declarations_in_weave(branch.content(), declarations);
            }
        }
        Object::Sequence(sequence) => {
            collect_struct_declarations_in_sequence(sequence, declarations)
        }
        Object::Weave(weave) => collect_struct_declarations_in_weave(weave, declarations),
        Object::AuthorWarning(_)
        | Object::Choice(_)
        | Object::ConstantDeclaration(_)
        | Object::Divert(_)
        | Object::Expression(_)
        | Object::ExternalDeclaration(_)
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

fn collect_struct_declarations_in_sequence<'a>(
    sequence: &'a Sequence,
    declarations: &mut Vec<&'a StructDeclaration>,
) {
    for content in sequence.elements() {
        collect_struct_declarations_in_content_list(content, declarations);
    }
}

#[cfg(test)]
mod tests {
    use crate::{analysis::test_support::assert_single_diagnostic, diagnostic::DiagnosticSeverity};

    use super::{super::test_support::parse_story, *};

    #[test]
    fn indexes_valid_nested_structures() {
        let story = parse_story(
            "STRUCT Item {\n\
             id: int\n\
             }\n\
             STRUCT Player {\n\
             inventory: Item[][]\n\
             hp: int\n\
             }\n\
             -> DONE",
        );

        let index = build_struct_type_index(&story);

        assert_eq!(
            index
                .get("Player")
                .and_then(|symbol| symbol.fields().get("inventory")),
            Some(&TypeName::array(TypeName::array(TypeName::struct_type(
                "Item"
            ))))
        );
        assert_eq!(
            index
                .get("Player")
                .and_then(|symbol| symbol.fields().get("hp")),
            Some(&TypeName::int())
        );
    }

    #[test]
    fn reports_duplicate_struct_names() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             STRUCT Player {\n\
             name: string\n\
             }\n\
             -> DONE",
        );

        let diagnostics = struct_type_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Duplicate struct declaration 'Player'",
        );
    }

    #[test]
    fn reports_duplicate_fields() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             hp: string\n\
             }\n\
             -> DONE",
        );

        let diagnostics = struct_type_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Duplicate field 'hp' in struct 'Player'",
        );
    }

    #[test]
    fn reports_unknown_field_types_inside_nested_arrays() {
        let story = parse_story(
            "STRUCT Player {\n\
             inventory: Item[][]\n\
             }\n\
             -> DONE",
        );

        let diagnostics = struct_type_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Unknown struct type 'Item' for field 'inventory' in struct 'Player'",
        );
    }

    #[test]
    fn reports_recursive_struct_references() {
        let story = parse_story(
            "STRUCT Node {\n\
             next: Node\n\
             }\n\
             -> DONE",
        );

        let diagnostics = struct_type_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Recursive struct type 'Node' is not supported",
        );
    }
}
