use crate::parsed::Story;

use super::{
    context::{EnumTypeIndex, StructTypeIndex, TargetSymbolIndex, VariableScopeIndex},
    enums::build_enum_type_index,
    interface_values::{build_module_implementation_index, ModuleImplementationIndex},
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
}

impl<'a> AnalysisIndexes<'a> {
    pub(super) fn build(story: &Story, module_analysis: &'a ModuleAnalysis) -> Self {
        Self {
            variable_scopes: build_variable_scope_index(story),
            struct_types: build_struct_type_index(story),
            enum_types: build_enum_type_index(story),
            target_symbols: build_target_symbol_index(story),
            interface_members: build_interface_member_index(story),
            module_imports: &module_analysis.imports,
            module_implementations: build_module_implementation_index(story),
        }
    }

    pub(super) fn mark_ready_for_incremental_migration(&self) {
        let _ = (
            &self.variable_scopes,
            &self.struct_types,
            &self.enum_types,
            &self.target_symbols,
            &self.interface_members,
            self.module_imports,
            &self.module_implementations,
        );
    }
}
