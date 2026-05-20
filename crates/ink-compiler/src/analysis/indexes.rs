use crate::parsed::Story;

use super::{
    context::{EnumTypeIndex, StructTypeIndex, TargetSymbolIndex, VariableScopeIndex},
    enums::build_enum_type_index,
    interface_values::{
        build_module_implementation_index, collect_interface_module_literal_uses,
        InterfaceModuleLiteralUses, ModuleImplementationIndex,
    },
    interfaces::{build_interface_member_index, InterfaceMemberIndex},
    modules::{ModuleAnalysis, ModuleImportIndex},
    structs::build_struct_type_index,
    target_symbols::build_target_symbol_index,
    variables::build_variable_scope_index,
};

pub(super) struct AnalysisIndexes<'a> {
    pub(super) variable_scopes: VariableScopeIndex,
    pub(super) struct_types: StructTypeIndex,
    pub(super) enum_types: EnumTypeIndex,
    pub(super) target_symbols: TargetSymbolIndex,
    pub(super) interface_members: InterfaceMemberIndex,
    pub(super) module_imports: &'a ModuleImportIndex,
    pub(super) module_implementations: ModuleImplementationIndex,
    pub(super) interface_module_literal_uses: InterfaceModuleLiteralUses,
}

impl<'a> AnalysisIndexes<'a> {
    pub(super) fn build(story: &Story, module_analysis: &'a ModuleAnalysis) -> Self {
        let variable_scopes = build_variable_scope_index(story);
        let struct_types = build_struct_type_index(story);
        let enum_types = build_enum_type_index(story);
        let target_symbols = build_target_symbol_index(story);
        let interface_members = build_interface_member_index(story);
        let module_implementations = build_module_implementation_index(story);
        let interface_module_literal_uses = collect_interface_module_literal_uses(
            story,
            &variable_scopes,
            &struct_types,
            &enum_types,
            &target_symbols,
            &module_implementations,
            &interface_members,
        );

        Self {
            variable_scopes,
            struct_types,
            enum_types,
            target_symbols,
            interface_members,
            module_imports: &module_analysis.imports,
            module_implementations,
            interface_module_literal_uses,
        }
    }
}
