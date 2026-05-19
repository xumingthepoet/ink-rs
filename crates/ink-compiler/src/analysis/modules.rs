mod dependencies;
mod entry_point;
mod imports;
mod symbols;

use crate::diagnostic::Diagnostic;
use crate::parsed::Story;

use super::ModuleEntryPoint;
pub use dependencies::{
    build_module_dependency_graph, build_module_reachability, ModuleDependencyGraph,
    ModuleReachability,
};
pub(in crate::analysis) use dependencies::{
    module_dependency_diagnostics, unreachable_module_diagnostics,
};
pub(in crate::analysis) use entry_point::{
    build_module_entry_point_analysis, mixed_root_module_diagnostics,
    module_entry_point_diagnostics, ModuleEntryPointAnalysis,
};
pub(in crate::analysis) use imports::module_import_diagnostics;
pub use imports::{build_module_import_index, ModuleImportIndex};
pub use symbols::ModuleSymbolIndex;
#[cfg(test)]
use symbols::ModuleSymbolKind;
pub(in crate::analysis) use symbols::{build_module_symbol_index, module_symbol_diagnostics};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::analysis) struct ModuleAnalysis {
    pub(in crate::analysis) entry_points: ModuleEntryPointAnalysis,
    pub(in crate::analysis) symbols: ModuleSymbolIndex,
    pub(in crate::analysis) dependencies: ModuleDependencyGraph,
    pub(in crate::analysis) imports: ModuleImportIndex,
    pub(in crate::analysis) reachability: ModuleReachability,
}

impl ModuleAnalysis {
    pub(in crate::analysis) fn build(story: &Story) -> Self {
        let entry_points = build_module_entry_point_analysis(story);
        let symbols = build_module_symbol_index(story);
        let dependencies = build_module_dependency_graph(story);
        let imports = build_module_import_index(story);
        let reachability =
            build_module_reachability(story, &dependencies, entry_points.entry_point());

        Self {
            entry_points,
            symbols,
            dependencies,
            imports,
            reachability,
        }
    }

    pub(in crate::analysis) fn into_checked_parts(
        self,
    ) -> (
        Option<ModuleEntryPoint>,
        ModuleSymbolIndex,
        ModuleDependencyGraph,
        ModuleImportIndex,
        ModuleReachability,
    ) {
        (
            self.entry_points.into_entry_point(),
            self.symbols,
            self.dependencies,
            self.imports,
            self.reachability,
        )
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
    use crate::parsed::{Story, TypeName};
    use crate::{Compiler, DiagnosticSeverity, SourceInput};

    use super::{
        super::test_support::{assert_single_diagnostic, parse_story},
        *,
    };

    fn import_diagnostics(story: &Story) -> Vec<Diagnostic> {
        let symbol_index = build_module_symbol_index(story);
        let import_index = build_module_import_index(story);
        module_import_diagnostics(story, &symbol_index, &import_index)
    }

    fn dependency_diagnostics(story: &Story) -> Vec<Diagnostic> {
        let graph = build_module_dependency_graph(story);
        module_dependency_diagnostics(&graph)
    }

    fn symbol_diagnostics(story: &Story) -> Vec<Diagnostic> {
        let symbol_index = build_module_symbol_index(story);
        module_symbol_diagnostics(story, &symbol_index)
    }

    fn entry_point_diagnostics(story: &Story) -> Vec<Diagnostic> {
        let entry_points = build_module_entry_point_analysis(story);
        module_entry_point_diagnostics(story, &entry_points)
    }

    fn entry_point(story: &Story) -> Option<ModuleEntryPoint> {
        build_module_entry_point_analysis(story).into_entry_point()
    }

    #[test]
    fn indexes_every_first_phase_importable_declaration_kind() {
        let story = parse_story(
            "=== module game ===\n\
             CONST MAX_SCORE: int = 3\n\
             VAR score: int = 0\n\
             STRUCT Player {\n\
             hp: int\n\
             name: string\n\
             }\n\
             ENUM State { Idle Busy }\n\
             EXTERNAL play(name: string) => void\n\
             == function heal(amount: int) => int ==\n\
             ~ return amount\n\
             == main(arg: string) ==\n\
             # tag\n\
             * (choice_id) Choice\n\
             - (gather_id)\n\
             = intro\n\
             ~ temp local: int = 0\n\
             -> END",
        );

        let index = build_module_symbol_index(&story);

        let constant = index.get("game", "MAX_SCORE").expect("constant symbol");
        assert_eq!(constant.kind(), ModuleSymbolKind::Constant);
        assert_eq!(constant.declared_type(), Some(&TypeName::int()));
        assert_eq!(constant.span().line, 2);

        let variable = index.get("game", "score").expect("global variable symbol");
        assert_eq!(variable.kind(), ModuleSymbolKind::GlobalVariable);
        assert_eq!(variable.declared_type(), Some(&TypeName::int()));
        assert_eq!(variable.span().line, 3);

        let struct_symbol = index.get("game", "Player").expect("struct symbol");
        assert_eq!(struct_symbol.kind(), ModuleSymbolKind::Struct);
        assert_eq!(struct_symbol.fields()[0].name(), "hp");
        assert_eq!(struct_symbol.fields()[0].type_name(), &TypeName::int());
        assert_eq!(struct_symbol.fields()[0].span().line, 5);
        assert_eq!(struct_symbol.fields()[1].name(), "name");
        assert_eq!(struct_symbol.fields()[1].type_name(), &TypeName::string());

        let enum_symbol = index.get("game", "State").expect("enum symbol");
        assert_eq!(enum_symbol.kind(), ModuleSymbolKind::Enum);
        assert_eq!(enum_symbol.span().line, 8);

        let external = index.get("game", "play").expect("external symbol");
        assert_eq!(external.kind(), ModuleSymbolKind::External);
        assert_eq!(external.span().line, 9);
        let external_signature = external.signature().expect("external signature");
        assert_eq!(external_signature.parameters()[0].name(), "name");
        assert_eq!(
            external_signature.parameters()[0].declared_type(),
            Some(&TypeName::string())
        );
        assert_eq!(external_signature.return_type(), &TypeName::void());
        assert!(external_signature.has_typed_signature());

        let function = index.get("game", "heal").expect("function symbol");
        assert_eq!(function.kind(), ModuleSymbolKind::Function);
        assert_eq!(function.span().line, 10);
        let function_signature = function.signature().expect("function signature");
        assert_eq!(function_signature.parameters()[0].name(), "amount");
        assert_eq!(
            function_signature.parameters()[0].declared_type(),
            Some(&TypeName::int())
        );
        assert_eq!(function_signature.return_type(), &TypeName::int());
        assert!(function_signature.has_typed_signature());

        let knot = index.get("game", "main").expect("knot symbol");
        assert_eq!(knot.kind(), ModuleSymbolKind::Knot);
        assert_eq!(knot.module(), "game");
        assert_eq!(knot.name(), "main");
        assert_eq!(knot.span().line, 12);
        let knot_signature = knot.signature().expect("knot signature");
        assert_eq!(knot_signature.parameters()[0].name(), "arg");
        assert_eq!(
            knot_signature.parameters()[0].declared_type(),
            Some(&TypeName::string())
        );
        assert_eq!(knot_signature.return_type(), &TypeName::void());
        assert!(knot_signature.has_typed_signature());

        assert!(index.get("game", "intro").is_none());
        assert!(index.get("game", "choice_id").is_none());
        assert!(index.get("game", "gather_id").is_none());
        assert!(index.get("game", "arg").is_none());
        assert!(index.get("game", "local").is_none());
        assert!(index.get("game", "tag").is_none());
    }

    #[test]
    fn keeps_same_symbol_names_separate_by_module() {
        let story = parse_story(
            "=== module game ===\n\
             CONST value: int = 1\n\
             === module ui ===\n\
             CONST value: string = \"ready\"",
        );

        let index = build_module_symbol_index(&story);

        let game_value = index.get("game", "value").expect("game value");
        let ui_value = index.get("ui", "value").expect("ui value");
        assert_eq!(game_value.declared_type(), Some(&TypeName::int()));
        assert_eq!(ui_value.declared_type(), Some(&TypeName::string()));
        assert_eq!(index.symbols_named("game", "value").len(), 1);
        assert_eq!(index.symbols_named("ui", "value").len(), 1);
    }

    #[test]
    fn records_empty_modules_without_symbols() {
        let story = parse_story("=== module empty ===");

        let index = build_module_symbol_index(&story);

        let symbols = index
            .symbols_for_module("empty")
            .expect("empty module should be indexed");
        assert!(symbols.is_empty());
    }

    #[test]
    fn records_acyclic_import_dependency_metadata_deterministically() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT sword\n\
             FROM audio IMPORT play\n\
             == main ==\n\
             -> END\n\
             === module items ===\n\
             == sword ==\n\
             -> END\n\
             === module audio ===\n\
             == play ==\n\
             -> END",
        );

        let checked = super::super::analyze(story);

        assert!(!checked.has_errors(), "{:#?}", checked.diagnostics);
        let graph = checked
            .artifact
            .expect("expected checked story")
            .module_dependencies;
        assert_eq!(
            graph.modules().collect::<Vec<_>>(),
            vec!["audio", "game", "items"]
        );
        assert_eq!(
            graph.dependencies_for("game"),
            &["audio".to_string(), "items".to_string()]
        );
        assert!(graph.contains_dependency("game", "items"));
        assert!(!graph.contains_dependency("items", "game"));
    }

    #[test]
    fn import_dependency_metadata_is_source_order_independent() {
        let game = SourceInput::named(
            "=== module game ===\n\
             FROM items IMPORT sword\n\
             FROM audio IMPORT play\n\
             == main ==\n\
             -> END",
            "game.ink",
        );
        let items = SourceInput::named(
            "=== module items ===\n\
             == sword ==\n\
             -> END",
            "items.ink",
        );
        let audio = SourceInput::named(
            "=== module audio ===\n\
             == play ==\n\
             -> END",
            "audio.ink",
        );
        let compiler = Compiler::default();

        let first = compiler.parse_sources(vec![game.clone(), items.clone(), audio.clone()]);
        let second = compiler.parse_sources(vec![audio, game, items]);
        assert!(!first.has_errors(), "{:#?}", first.diagnostics);
        assert!(!second.has_errors(), "{:#?}", second.diagnostics);

        let first_checked = super::super::analyze(first.artifact.expect("first story"));
        let second_checked = super::super::analyze(second.artifact.expect("second story"));
        assert!(
            !first_checked.has_errors(),
            "{:#?}",
            first_checked.diagnostics
        );
        assert!(
            !second_checked.has_errors(),
            "{:#?}",
            second_checked.diagnostics
        );

        assert_eq!(
            first_checked
                .artifact
                .expect("first checked")
                .module_dependencies,
            second_checked
                .artifact
                .expect("second checked")
                .module_dependencies
        );
    }

    #[test]
    fn reports_direct_import_cycles() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT sword\n\
             == main ==\n\
             -> END\n\
             === module items ===\n\
             FROM game IMPORT main\n\
             == sword ==\n\
             -> END",
        );

        let diagnostics = dependency_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].line, 6);
        assert_eq!(
            diagnostics[0].message,
            "Cyclic module import detected: game -> items -> game"
        );
    }

    #[test]
    fn reports_transitive_import_cycles() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT sword\n\
             == main ==\n\
             -> END\n\
             === module items ===\n\
             FROM audio IMPORT play\n\
             == sword ==\n\
             -> END\n\
             === module audio ===\n\
             FROM game IMPORT main\n\
             == play ==\n\
             -> END",
        );

        let diagnostics = dependency_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].line, 6);
        assert_eq!(
            diagnostics[0].message,
            "Cyclic module import detected: audio -> game -> items -> audio"
        );
    }

    #[test]
    fn validates_missing_import_source_modules() {
        let story = parse_story(
            "=== module game ===\n\
             FROM missing IMPORT sword\n\
             == main ==\n\
             -> END",
        );

        let diagnostics = import_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].line, 2);
        assert_eq!(
            diagnostics[0].message,
            "Imported module 'missing' does not exist"
        );
    }

    #[test]
    fn validates_missing_imported_symbols() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT sword\n\
             == main ==\n\
             -> END\n\
             === module items ===\n\
             == shield ==\n\
             -> END",
        );

        let diagnostics = import_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].line, 2);
        assert_eq!(
            diagnostics[0].message,
            "Module 'items' does not define importable symbol 'sword'"
        );
    }

    #[test]
    fn rejects_stitch_imports() {
        let story = parse_story(
            "=== module game ===\n\
             FROM scenes IMPORT intro\n\
             == main ==\n\
             -> END\n\
             === module scenes ===\n\
             == opening ==\n\
             = intro\n\
             -> END",
        );

        let diagnostics = import_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "Cannot import stitch 'intro' from module 'scenes'; stitches are scoped to their parent knot"
        );
    }

    #[test]
    fn records_import_allow_lists_and_warns_for_unused_imports() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT sword\n\
             == main ==\n\
             -> END\n\
             === module items ===\n\
             == sword ==\n\
             -> END",
        );

        let checked = super::super::analyze(story);

        assert!(!checked.has_errors(), "{:#?}", checked.diagnostics);
        assert_eq!(checked.diagnostics.len(), 1, "{:#?}", checked.diagnostics);
        assert_eq!(checked.diagnostics[0].severity, DiagnosticSeverity::Warning);
        assert_eq!(
            checked.diagnostics[0].message,
            "Imported symbol 'items::sword' is never used"
        );
        let checked = checked.artifact.expect("expected checked story");
        assert!(checked.module_imports.allows("game", "items", "sword"));
        assert!(!checked.module_imports.allows("game", "items", "shield"));
    }

    #[test]
    fn bare_module_imports_create_dependencies_without_symbol_access() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items\n\
             == main ==\n\
             -> END\n\
             === module items ===\n\
             == sword ==\n\
             -> END",
        );

        let checked = super::super::analyze(story);

        assert!(!checked.has_errors(), "{:#?}", checked.diagnostics);
        assert!(
            checked.diagnostics.is_empty(),
            "bare module imports should not create symbol-use warnings: {:#?}",
            checked.diagnostics
        );
        let checked = checked.artifact.expect("expected checked story");
        assert!(checked
            .module_dependencies
            .contains_dependency("game", "items"));
        assert!(!checked.module_imports.allows("game", "items", "sword"));
    }

    #[test]
    fn treats_existing_qualified_divert_paths_as_import_uses() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT sword\n\
             == main ==\n\
             -> items::sword\n\
             === module items ===\n\
             == sword ==\n\
             -> END",
        );

        let diagnostics = import_diagnostics(&story);

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn accepts_directly_imported_qualified_references() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT helper\n\
             == main ==\n\
             ~ items::helper()\n\
             -> END\n\
             === module items ===\n\
             == function helper() => void ==\n\
             ~ return",
        );

        let diagnostics = import_diagnostics(&story);

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn rejects_cross_module_qualified_references_without_direct_import() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             ~ items::helper()\n\
             -> END\n\
             === module items ===\n\
             == function helper() => void ==\n\
             ~ return",
        );

        let diagnostics = import_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "Qualified reference 'items::helper' requires a direct import in module 'game': FROM items IMPORT helper"
        );
    }

    #[test]
    fn rejects_transitive_only_qualified_import_access() {
        let story = parse_story(
            "=== module game ===\n\
             FROM bridge IMPORT relay\n\
             == main ==\n\
             ~ bridge::relay()\n\
             ~ items::helper()\n\
             -> END\n\
             === module bridge ===\n\
             FROM items IMPORT helper\n\
             == function relay() => void ==\n\
             ~ items::helper()\n\
             ~ return\n\
             === module items ===\n\
             == function helper() => void ==\n\
             ~ return",
        );

        let diagnostics = import_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "Qualified reference 'items::helper' requires a direct import in module 'game': FROM items IMPORT helper"
        );
    }

    #[test]
    fn rejects_imports_from_wrong_module_for_qualified_use() {
        let story = parse_story(
            "=== module game ===\n\
             FROM audio IMPORT helper\n\
             == main ==\n\
             ~ audio::helper()\n\
             -> END\n\
             === module items ===\n\
             == function helper() => void ==\n\
             ~ return\n\
             === module audio ===\n\
             == function play() => void ==\n\
             ~ return",
        );

        let diagnostics = import_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "Module 'audio' does not define importable symbol 'helper'"
        );
    }

    #[test]
    fn accepts_same_module_self_qualified_references_without_import() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             ~ game::helper()\n\
             -> END\n\
             == function helper() => void ==\n\
             ~ return",
        );

        let diagnostics = import_diagnostics(&story);

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn main_module_has_no_special_cross_module_visibility() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             ~ items::helper()\n\
             -> END\n\
             === module items ===\n\
             == function helper() => void ==\n\
             ~ return",
        );

        let diagnostics = import_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "Qualified reference 'items::helper' requires a direct import in module 'game': FROM items IMPORT helper"
        );
    }

    #[test]
    fn qualified_struct_type_references_count_as_import_uses() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT Item\n\
             VAR item: items::Item\n\
             == main ==\n\
             -> END\n\
             === module items ===\n\
             STRUCT Item {\n\
             hp: int\n\
             }\n\
             == helper ==\n\
             -> END",
        );

        let diagnostics = import_diagnostics(&story);

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn qualified_struct_type_references_require_direct_imports() {
        let story = parse_story(
            "=== module game ===\n\
             VAR item: items::Item\n\
             == main ==\n\
             -> END\n\
             === module items ===\n\
             STRUCT Item {\n\
             hp: int\n\
             }\n\
             == helper ==\n\
             -> END",
        );

        let diagnostics = import_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Qualified reference 'items::Item' requires a direct import in module 'game': FROM items IMPORT Item",
        );
    }

    #[test]
    fn qualified_enum_type_references_count_as_import_uses() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT State\n\
             VAR state: items::State\n\
             == main ==\n\
             -> END\n\
             === module items ===\n\
             ENUM State { Idle Busy }\n\
             == helper ==\n\
             -> END",
        );

        let diagnostics = import_diagnostics(&story);

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn warns_for_modules_unreachable_from_main_import_path() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             -> END\n\
             === module unused ===\n\
             == helper ==\n\
             -> END",
        );
        let graph = build_module_dependency_graph(&story);
        let entry_points = build_module_entry_point_analysis(&story);
        let reachability = build_module_reachability(&story, &graph, entry_points.entry_point());

        let diagnostics = unreachable_module_diagnostics(&story, &reachability);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Warning);
        assert_eq!(
            diagnostics[0].message,
            "Module 'unused' is not reachable from entry point 'game::main'"
        );
        assert!(reachability.is_reachable("game"));
        assert!(!reachability.is_reachable("unused"));
        assert_eq!(
            reachability.reachable_modules().collect::<Vec<_>>(),
            vec!["game"]
        );
    }

    #[test]
    fn internal_modules_and_import_dependencies_are_reachable_roots() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             -> END\n\
             === module host_api ===\n\
             FROM config IMPORT value\n\
             == INTERNAL read() => string ==\n\
             ~ return config::value()\n\
             === module config ===\n\
             == function value() => string ==\n\
             ~ return \"ok\"\n\
             === module unused ===\n\
             == helper ==\n\
             -> END",
        );
        let graph = build_module_dependency_graph(&story);
        let entry_points = build_module_entry_point_analysis(&story);
        let reachability = build_module_reachability(&story, &graph, entry_points.entry_point());

        let diagnostics = unreachable_module_diagnostics(&story, &reachability);

        assert!(reachability.is_reachable("game"));
        assert!(reachability.is_reachable("host_api"));
        assert!(reachability.is_reachable("config"));
        assert!(!reachability.is_reachable("unused"));
        assert_eq!(
            reachability.reachable_modules().collect::<Vec<_>>(),
            vec!["config", "game", "host_api"]
        );
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Warning);
        assert_eq!(
            diagnostics[0].message,
            "Module 'unused' is not reachable from entry point 'game::main'"
        );
    }

    #[test]
    fn errors_in_unreachable_modules_still_fail_analysis() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             -> END\n\
             === module unused ===\n\
             CONST value: int = 1\n\
             VAR value: int = 0",
        );

        let checked = super::super::analyze(story);

        assert!(checked.has_errors(), "{:#?}", checked.diagnostics);
        assert!(checked.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic.message.contains("already contains a constant")
        }));
        assert!(checked.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Warning
                && diagnostic.message
                    == "Module 'unused' is not reachable from entry point 'game::main'"
        }));
    }

    #[test]
    fn reports_duplicate_modules_in_one_source() {
        let story = parse_story(
            "=== module game ===\n\
             === module game ===",
        );

        let diagnostics = symbol_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].line, 2);
        assert_eq!(
            diagnostics[0].message,
            "Module 'game' is already declared in this compilation"
        );
    }

    #[test]
    fn reports_duplicate_modules_across_source_inputs() {
        let parsed = Compiler::default().parse_sources(vec![
            SourceInput::named("=== module game ===", "first.ink"),
            SourceInput::named("=== module game ===", "second.ink"),
        ]);
        assert!(!parsed.has_errors(), "{:#?}", parsed.diagnostics);
        let story = parsed.artifact.expect("expected parsed story");

        let diagnostics = symbol_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(
            diagnostics[0].source_filename.as_deref(),
            Some("second.ink")
        );
        assert_eq!(
            diagnostics[0].message,
            "Module 'game' is already declared in this compilation"
        );
    }

    #[test]
    fn reports_same_module_namespace_collisions_across_importable_kinds() {
        let story = parse_story(
            "=== module game ===\n\
             CONST shared: int = 1\n\
             VAR shared: int = 0\n\
             STRUCT Player {\n\
             hp: int\n\
             }\n\
             ENUM Player { Ready }\n\
             EXTERNAL Player() => void\n\
             == function util() => void ==\n\
             ~ return\n\
             == util ==\n\
             -> END",
        );

        let diagnostics = symbol_diagnostics(&story);

        assert_eq!(diagnostics.len(), 4, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].line, 3);
        assert_eq!(
            diagnostics[0].message,
            "Module 'game' already contains a constant named 'shared'; global variable declarations cannot reuse that name"
        );
        assert_eq!(diagnostics[1].line, 7);
        assert_eq!(
            diagnostics[1].message,
            "Module 'game' already contains a struct named 'Player'; enum declarations cannot reuse that name"
        );
        assert_eq!(diagnostics[2].line, 8);
        assert_eq!(
            diagnostics[2].message,
            "Module 'game' already contains a struct named 'Player'; external declarations cannot reuse that name"
        );
        assert_eq!(diagnostics[3].line, 11);
        assert_eq!(
            diagnostics[3].message,
            "Module 'game' already contains a function named 'util'; knot declarations cannot reuse that name"
        );
    }

    #[test]
    fn allows_same_symbol_names_in_different_modules() {
        let story = parse_story(
            "=== module game ===\n\
             CONST value: int = 1\n\
             == main ==\n\
             -> END\n\
             === module ui ===\n\
             CONST value: string = \"ready\"\n\
             == helper ==\n\
             -> END",
        );

        let diagnostics = symbol_diagnostics(&story);

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn allows_stitch_names_to_repeat_inside_different_knots() {
        let story = parse_story(
            "=== module game ===\n\
             == first ==\n\
             = shared\n\
             -> END\n\
             == second ==\n\
             = shared\n\
             -> END",
        );

        let diagnostics = symbol_diagnostics(&story);

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn reports_missing_main_for_explicit_module_compilation() {
        let story = parse_story(
            "=== module library ===\n\
             == helper ==\n\
             -> END",
        );

        let diagnostics = entry_point_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].line, 1);
        assert_eq!(
            diagnostics[0].message,
            "Explicit module compilation requires exactly one module to define a knot named 'main'"
        );
        assert!(entry_point(&story).is_none());
    }

    #[test]
    fn records_unique_module_qualified_main_entry_point() {
        let story = parse_story(
            "=== module helpers ===\n\
             == setup ==\n\
             -> END\n\
             === module game ===\n\
             == main ==\n\
             -> END",
        );

        let checked = super::super::analyze(story);

        assert!(!checked.has_errors(), "{:#?}", checked.diagnostics);
        let entry_point = checked
            .artifact
            .expect("expected checked story")
            .entry_point
            .expect("expected module entry point");
        assert_eq!(entry_point.module, "game");
        assert_eq!(entry_point.knot, "main");
        assert_eq!(entry_point.qualified_name(), "game::main");
    }

    #[test]
    fn checked_story_records_module_symbol_metadata() {
        let story = parse_story(
            "=== module game ===\n\
             FROM audio IMPORT play\n\
             VAR score: int = 0\n\
             == main ==\n\
             -> END\n\
             === module audio ===\n\
             EXTERNAL play(name: string) => int\n\
             == helper ==\n\
             -> END",
        );

        let checked = super::super::analyze(story)
            .artifact
            .expect("analysis should produce checked story");

        assert_eq!(
            checked
                .module_symbols
                .get("game", "score")
                .expect("score symbol")
                .kind(),
            ModuleSymbolKind::GlobalVariable
        );
        assert_eq!(
            checked
                .module_symbols
                .get("audio", "play")
                .expect("external symbol")
                .kind(),
            ModuleSymbolKind::External
        );
    }

    #[test]
    fn reports_multiple_main_knots_across_modules() {
        let story = parse_story(
            "=== module first ===\n\
             == main ==\n\
             -> END\n\
             === module second ===\n\
             == main ==\n\
             -> END",
        );

        let diagnostics = entry_point_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].line, 5);
        assert_eq!(
            diagnostics[0].message,
            "Multiple 'main' knots are declared; runnable entry point must be unique"
        );
        assert!(entry_point(&story).is_none());
    }

    #[test]
    fn root_weave_sources_have_no_module_entry_point() {
        let story = parse_story("Line.");

        assert!(entry_point_diagnostics(&story).is_empty());
        assert!(entry_point(&story).is_none());
    }
}
