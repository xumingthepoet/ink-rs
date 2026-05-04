use std::collections::HashSet;

use crate::{
    diagnostic::Diagnostic,
    parsed::{EnumDeclaration, Flow, Object, Story, Weave},
};

use super::context::{EnumTypeIndex, EnumTypeSymbol};

#[derive(Debug, Clone, Copy)]
struct EnumDeclarationRecord<'a> {
    module: Option<&'a str>,
    declaration: &'a EnumDeclaration,
}

pub(super) fn enum_type_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let declarations = collect_enum_declarations(story);
    build_enum_type_index_from_declarations(&declarations).1
}

pub(super) fn build_enum_type_index(story: &Story) -> EnumTypeIndex {
    let declarations = collect_enum_declarations(story);
    build_enum_type_index_from_declarations(&declarations).0
}

fn build_enum_type_index_from_declarations(
    declarations: &[EnumDeclarationRecord<'_>],
) -> (EnumTypeIndex, Vec<Diagnostic>) {
    let mut index = EnumTypeIndex::new();
    let mut diagnostics = Vec::new();

    for record in declarations {
        let declaration = record.declaration;
        let key = scoped_enum_name(record.module, declaration.name());
        if index.contains_key(&key) {
            diagnostics.push(Diagnostic::error(
                declaration.span().clone(),
                format!("Duplicate enum declaration '{}'", declaration.name()),
            ));
            continue;
        }

        if declaration.members().is_empty() {
            diagnostics.push(Diagnostic::error(
                declaration.span().clone(),
                format!(
                    "Enum '{}' must declare at least one member",
                    declaration.name()
                ),
            ));
        }

        let mut seen_members = HashSet::new();
        let mut unique_members = Vec::new();
        for member in declaration.members() {
            if !seen_members.insert(member.name().to_string()) {
                diagnostics.push(Diagnostic::error(
                    member.span().clone(),
                    format!(
                        "Duplicate member '{}' in enum '{}'",
                        member.name(),
                        declaration.name()
                    ),
                ));
                continue;
            }
            unique_members.push(member.name().to_string());
        }

        index.insert(key, EnumTypeSymbol::new(unique_members));
    }

    (index, diagnostics)
}

pub(super) fn scoped_enum_name(module: Option<&str>, name: &str) -> String {
    module
        .map(|module| format!("{module}::{name}"))
        .unwrap_or_else(|| name.to_string())
}

#[cfg(test)]
pub(super) fn resolve_enum_symbol<'a>(
    index: &'a EnumTypeIndex,
    enum_name: &str,
    current_module: Option<&str>,
) -> Option<&'a EnumTypeSymbol> {
    let key = if enum_name.contains("::") {
        enum_name.to_string()
    } else {
        scoped_enum_name(current_module, enum_name)
    };
    index.get(&key)
}

fn collect_enum_declarations(story: &Story) -> Vec<EnumDeclarationRecord<'_>> {
    let mut declarations = Vec::new();
    collect_enum_declarations_in_weave(None, story.root_weave(), &mut declarations);
    for flow in story.flows() {
        collect_enum_declarations_in_flow(None, flow, &mut declarations);
    }
    for module in story.modules() {
        collect_enum_declarations_in_weave(Some(module.name()), module.weave(), &mut declarations);
        for flow in module.flows() {
            collect_enum_declarations_in_flow(Some(module.name()), flow, &mut declarations);
        }
    }
    declarations
}

fn collect_enum_declarations_in_flow<'a>(
    module: Option<&'a str>,
    flow: &'a Flow,
    declarations: &mut Vec<EnumDeclarationRecord<'a>>,
) {
    collect_enum_declarations_in_weave(module, flow.weave(), declarations);
    for child in flow.child_flows() {
        collect_enum_declarations_in_flow(module, child, declarations);
    }
}

fn collect_enum_declarations_in_weave<'a>(
    module: Option<&'a str>,
    weave: &'a Weave,
    declarations: &mut Vec<EnumDeclarationRecord<'a>>,
) {
    collect_enum_declarations_in_objects(module, weave.content(), declarations);
}

fn collect_enum_declarations_in_objects<'a>(
    module: Option<&'a str>,
    objects: &'a [Object],
    declarations: &mut Vec<EnumDeclarationRecord<'a>>,
) {
    for object in objects {
        collect_enum_declarations_in_object(module, object, declarations);
    }
}

fn collect_enum_declarations_in_object<'a>(
    module: Option<&'a str>,
    object: &'a Object,
    declarations: &mut Vec<EnumDeclarationRecord<'a>>,
) {
    match object {
        Object::EnumDeclaration(declaration) => declarations.push(EnumDeclarationRecord {
            module,
            declaration,
        }),
        Object::ContentList(content) => {
            collect_enum_declarations_in_objects(module, content.objects(), declarations)
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_enum_declarations_in_weave(module, branch.content(), declarations);
            }
        }
        Object::Weave(weave) => collect_enum_declarations_in_weave(module, weave, declarations),
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
        | Object::StructDeclaration(_)
        | Object::Tag(_)
        | Object::Text(_)
        | Object::TunnelOnwards(_)
        | Object::VariableAssignment(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use crate::{analysis::test_support::assert_single_diagnostic, DiagnosticSeverity};

    use super::{super::test_support::parse_story, *};

    #[test]
    fn indexes_root_and_module_enums() {
        let root_story = parse_story("ENUM RootState { Idle Busy }\n-> DONE");
        let root_index = build_enum_type_index(&root_story);

        assert_eq!(
            root_index.get("RootState").map(EnumTypeSymbol::members),
            Some(&["Idle".to_string(), "Busy".to_string()][..])
        );

        let module_story = parse_story(
            "=== module game ===\n\
             ENUM State {\n\
             Ready\n\
             Done\n\
             }\n\
             == main ==\n\
             -> END",
        );
        let module_index = build_enum_type_index(&module_story);

        assert!(module_index
            .get("game::State")
            .is_some_and(|symbol| symbol.contains_member("Ready")));
    }

    #[test]
    fn reports_empty_enums() {
        let story = parse_story("ENUM State {}");

        let diagnostics = enum_type_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Enum 'State' must declare at least one member",
        );
    }

    #[test]
    fn reports_duplicate_enum_members() {
        let story = parse_story("ENUM State { Idle Idle }");

        let diagnostics = enum_type_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Duplicate member 'Idle' in enum 'State'",
        );
    }

    #[test]
    fn reports_duplicate_enum_names_inside_same_scope() {
        let story = parse_story("ENUM State { Idle }\nENUM State { Busy }");

        let diagnostics = enum_type_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Duplicate enum declaration 'State'",
        );
    }

    #[test]
    fn resolves_enum_symbols_by_current_module() {
        let story = parse_story(
            "=== module game ===\n\
             ENUM State { Ready }\n\
             == main ==\n\
             -> END",
        );
        let index = build_enum_type_index(&story);

        assert!(resolve_enum_symbol(&index, "State", Some("game")).is_some());
        assert!(resolve_enum_symbol(&index, "game::State", None).is_some());
        assert!(resolve_enum_symbol(&index, "State", None).is_none());
    }
}
