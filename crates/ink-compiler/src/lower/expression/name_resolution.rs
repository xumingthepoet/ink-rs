use std::collections::HashSet;

use super::super::{
    context::ChoicePathMode,
    indexes::{ConstantValues, ExternalSignatures},
};

pub(super) fn resolve_runtime_variable_name(
    name: &str,
    path_mode: &ChoicePathMode,
    global_variables: &HashSet<String>,
) -> String {
    if name.contains("::") || path_mode.is_local_variable(name) {
        return name.to_string();
    }

    path_mode
        .current_module_name()
        .map(|module_name| format!("{module_name}::{name}"))
        .filter(|qualified_name| global_variables.contains(qualified_name))
        .unwrap_or_else(|| name.to_string())
}

pub(super) fn resolve_constant_name(
    name: &str,
    path_mode: &ChoicePathMode,
    constants: &ConstantValues,
) -> Option<String> {
    if constants.contains_key(name) {
        return Some(name.to_string());
    }

    if name.contains("::") {
        return None;
    }

    path_mode
        .current_module_name()
        .map(|module_name| format!("{module_name}::{name}"))
        .filter(|qualified_name| constants.contains_key(qualified_name))
}

pub(super) fn resolve_callable_name(
    name: &str,
    external_signatures: &ExternalSignatures,
    path_mode: &ChoicePathMode,
) -> String {
    if external_signatures.contains_key(name) || name.contains("::") {
        return name.to_string();
    }

    path_mode
        .current_module_name()
        .map(|module_name| format!("{module_name}::{name}"))
        .filter(|qualified_name| external_signatures.contains_key(qualified_name))
        .unwrap_or_else(|| name.to_string())
}
