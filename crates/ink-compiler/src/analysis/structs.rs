use std::collections::{BTreeMap, BTreeSet};

use crate::{
    diagnostic::Diagnostic,
    parsed::{ContentList, Flow, Object, Story, StructDeclaration, TypeName, Weave},
};

use super::{
    context::{EnumTypeIndex, StructTypeIndex, StructTypeSymbol},
    enums::build_enum_type_index,
};

#[derive(Debug, Clone, Copy)]
struct StructDeclarationRecord<'a> {
    module: Option<&'a str>,
    declaration: &'a StructDeclaration,
}

pub(super) fn struct_type_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let declarations = collect_struct_declarations(story);
    let (index, mut diagnostics) = build_struct_type_index_from_declarations(&declarations);
    let enum_index = build_enum_type_index(story);
    diagnostics.extend(unknown_field_type_diagnostics(
        &declarations,
        &index,
        &enum_index,
    ));
    diagnostics.extend(recursive_struct_diagnostics(&declarations, &index));
    diagnostics
}

pub(super) fn build_struct_type_index(story: &Story) -> StructTypeIndex {
    let declarations = collect_struct_declarations(story);
    build_struct_type_index_from_declarations(&declarations).0
}

fn build_struct_type_index_from_declarations(
    declarations: &[StructDeclarationRecord<'_>],
) -> (StructTypeIndex, Vec<Diagnostic>) {
    let mut index = StructTypeIndex::new();
    let mut diagnostics = Vec::new();

    for record in declarations {
        let declaration = record.declaration;
        let key = scoped_struct_name(record.module, declaration.name());
        if index.contains_key(&key) {
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

        index.insert(key, StructTypeSymbol::new(fields));
    }

    (index, diagnostics)
}

fn unknown_field_type_diagnostics(
    declarations: &[StructDeclarationRecord<'_>],
    index: &StructTypeIndex,
    enum_index: &EnumTypeIndex,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for record in declarations {
        let declaration = record.declaration;
        for field in declaration.fields() {
            for (type_name, key) in referenced_named_type_names(field.type_name(), record.module) {
                if !index.contains_key(&key) && !enum_index.contains_key(&key) {
                    diagnostics.push(Diagnostic::error(
                        field.span().clone(),
                        format!(
                            "Unknown named type '{}' for field '{}' in struct '{}'",
                            type_name,
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
    declarations: &[StructDeclarationRecord<'_>],
    index: &StructTypeIndex,
) -> Vec<Diagnostic> {
    let adjacency = struct_dependency_graph(index);
    let mut diagnostics = Vec::new();
    let mut reported = BTreeSet::new();

    for record in declarations {
        let declaration = record.declaration;
        let key = scoped_struct_name(record.module, declaration.name());
        if !index.contains_key(&key) || reported.contains(&key) {
            continue;
        }
        if reaches_struct(&key, &key, &adjacency, &mut BTreeSet::new()) {
            reported.insert(key);
            diagnostics.push(Diagnostic::error(
                declaration.span().clone(),
                format!(
                    "Recursive struct type '{}' is not supported",
                    declaration.name()
                ),
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
                .flat_map(|type_name| referenced_named_type_names(type_name, module_name(name)))
                .map(|(_, referenced)| referenced)
                .filter(|referenced| index.contains_key(referenced))
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

pub(super) fn scoped_struct_name(module: Option<&str>, name: &str) -> String {
    module
        .map(|module| format!("{module}::{name}"))
        .unwrap_or_else(|| name.to_string())
}

pub(super) fn resolve_struct_symbol<'a>(
    index: &'a StructTypeIndex,
    struct_name: &str,
    current_module: Option<&str>,
) -> Option<&'a StructTypeSymbol> {
    let key = if struct_name.contains("::") {
        struct_name.to_string()
    } else {
        scoped_struct_name(current_module, struct_name)
    };
    index.get(&key)
}

fn module_name(scoped_name: &str) -> Option<&str> {
    scoped_name.split_once("::").map(|(module, _)| module)
}

fn referenced_named_type_names(
    type_name: &TypeName,
    current_module: Option<&str>,
) -> Vec<(String, String)> {
    match type_name {
        TypeName::Struct(name) => {
            vec![(name.clone(), scoped_struct_name(current_module, name))]
        }
        TypeName::QualifiedStruct(name) => {
            vec![(name.as_str().to_string(), name.as_str().to_string())]
        }
        TypeName::Array(element_type) => referenced_named_type_names(element_type, current_module),
        TypeName::Dict { value_type, .. } => {
            referenced_named_type_names(value_type, current_module)
        }
        TypeName::Primitive(_) | TypeName::Interface { .. } | TypeName::Void => Vec::new(),
    }
}

fn collect_struct_declarations(story: &Story) -> Vec<StructDeclarationRecord<'_>> {
    let mut declarations = Vec::new();
    collect_struct_declarations_in_weave(None, story.root_weave(), &mut declarations);
    for flow in story.flows() {
        collect_struct_declarations_in_flow(None, flow, &mut declarations);
    }
    for module in story.modules() {
        collect_struct_declarations_in_weave(
            Some(module.name()),
            module.weave(),
            &mut declarations,
        );
        for flow in module.flows() {
            collect_struct_declarations_in_flow(Some(module.name()), flow, &mut declarations);
        }
    }
    declarations
}

fn collect_struct_declarations_in_flow<'a>(
    module: Option<&'a str>,
    flow: &'a Flow,
    declarations: &mut Vec<StructDeclarationRecord<'a>>,
) {
    collect_struct_declarations_in_weave(module, flow.weave(), declarations);
    for child in flow.child_flows() {
        collect_struct_declarations_in_flow(module, child, declarations);
    }
}

fn collect_struct_declarations_in_content_list<'a>(
    module: Option<&'a str>,
    content: &'a ContentList,
    declarations: &mut Vec<StructDeclarationRecord<'a>>,
) {
    collect_struct_declarations_in_objects(module, content.objects(), declarations);
}

fn collect_struct_declarations_in_weave<'a>(
    module: Option<&'a str>,
    weave: &'a Weave,
    declarations: &mut Vec<StructDeclarationRecord<'a>>,
) {
    collect_struct_declarations_in_objects(module, weave.content(), declarations);
}

fn collect_struct_declarations_in_objects<'a>(
    module: Option<&'a str>,
    objects: &'a [Object],
    declarations: &mut Vec<StructDeclarationRecord<'a>>,
) {
    for object in objects {
        collect_struct_declarations_in_object(module, object, declarations);
    }
}

fn collect_struct_declarations_in_object<'a>(
    module: Option<&'a str>,
    object: &'a Object,
    declarations: &mut Vec<StructDeclarationRecord<'a>>,
) {
    match object {
        Object::StructDeclaration(declaration) => declarations.push(StructDeclarationRecord {
            module,
            declaration,
        }),
        Object::ContentList(content) => {
            collect_struct_declarations_in_content_list(module, content, declarations)
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_struct_declarations_in_weave(module, branch.content(), declarations);
            }
        }
        Object::Weave(weave) => collect_struct_declarations_in_weave(module, weave, declarations),
        Object::AuthorWarning(_)
        | Object::Choice(_)
        | Object::ConstantDeclaration(_)
        | Object::Divert(_)
        | Object::EnumDeclaration(_)
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
    fn indexes_module_structs_by_module_scope() {
        let story = parse_story(
            "=== module game ===\n\
             STRUCT Item {\n\
             hp: int\n\
             }\n\
             == main ==\n\
             -> DONE\n\
             === module items ===\n\
             STRUCT Item {\n\
             label: string\n\
             }\n\
             == helper ==\n\
             -> DONE",
        );

        let index = build_struct_type_index(&story);

        assert_eq!(
            index
                .get("game::Item")
                .and_then(|symbol| symbol.fields().get("hp")),
            Some(&TypeName::int())
        );
        assert_eq!(
            index
                .get("items::Item")
                .and_then(|symbol| symbol.fields().get("label")),
            Some(&TypeName::string())
        );
        assert!(struct_type_diagnostics(&story).is_empty());
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
    fn reports_duplicate_struct_names_only_inside_one_module() {
        let story = parse_story(
            "=== module game ===\n\
             STRUCT Item {\n\
             hp: int\n\
             }\n\
             STRUCT Item {\n\
             label: string\n\
             }\n\
             == main ==\n\
             -> DONE\n\
             === module items ===\n\
             STRUCT Item {\n\
             label: string\n\
             }\n\
             == helper ==\n\
             -> DONE",
        );

        let diagnostics = struct_type_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Duplicate struct declaration 'Item'",
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
            "Unknown named type 'Item' for field 'inventory' in struct 'Player'",
        );
    }

    #[test]
    fn resolves_unqualified_field_types_only_inside_current_module() {
        let story = parse_story(
            "=== module game ===\n\
             STRUCT Box {\n\
             item: Item\n\
             }\n\
             == main ==\n\
             -> DONE\n\
             === module items ===\n\
             STRUCT Item {\n\
             label: string\n\
             }\n\
             == helper ==\n\
             -> DONE",
        );

        let diagnostics = struct_type_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Unknown named type 'Item' for field 'item' in struct 'Box'",
        );
    }

    #[test]
    fn resolves_unqualified_field_types_inside_same_module() {
        let story = parse_story(
            "=== module game ===\n\
             STRUCT Item {\n\
             hp: int\n\
             }\n\
             STRUCT Box {\n\
             item: Item\n\
             }\n\
             == main ==\n\
             -> DONE",
        );

        assert!(struct_type_diagnostics(&story).is_empty());
    }

    #[test]
    fn accepts_enum_field_types_in_same_scope() {
        let story = parse_story(
            "ENUM State { Idle Busy }\n\
             STRUCT Actor {\n\
             state: State\n\
             }\n\
             -> DONE",
        );

        assert!(struct_type_diagnostics(&story).is_empty());
    }

    #[test]
    fn accepts_qualified_enum_field_types() {
        let story = parse_story(
            "=== module game ===\n\
             STRUCT Actor {\n\
             state: items::State\n\
             }\n\
             == main ==\n\
             -> END\n\
             === module items ===\n\
             ENUM State { Idle Busy }\n\
             == helper ==\n\
             -> END",
        );

        assert!(struct_type_diagnostics(&story).is_empty());
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
