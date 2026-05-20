use crate::parsed::{DivertTarget, TypeName};

use super::{
    context::{FlowSymbol, TargetSymbolIndex},
    target_symbols::resolve_target_symbol,
    type_names::qualify_type_name_for_module,
};

pub(super) struct ResolvedExpectedArguments {
    target_name: String,
    arguments: Vec<ResolvedExpectedArgument>,
}

pub(super) struct ResolvedExpectedArgument {
    name: String,
    declared_type: Option<TypeName>,
}

pub(super) enum FunctionCallArgumentResolution {
    Function(ResolvedExpectedArguments),
    NonFunction,
    Missing,
}

impl ResolvedExpectedArguments {
    pub(super) fn target_name(&self) -> &str {
        &self.target_name
    }

    pub(super) fn arguments(&self) -> &[ResolvedExpectedArgument] {
        &self.arguments
    }
}

impl ResolvedExpectedArgument {
    pub(super) fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn declared_type(&self) -> Option<&TypeName> {
        self.declared_type.as_ref()
    }
}

pub(super) fn resolve_static_target_expected_arguments(
    target: &DivertTarget,
    current_module: Option<&str>,
    current_flow_path: Option<&str>,
    target_symbols: &TargetSymbolIndex,
) -> Option<ResolvedExpectedArguments> {
    let target_name = match target {
        DivertTarget::Path(target) => target.as_str(),
        DivertTarget::QualifiedPath(target) => target.as_str(),
        DivertTarget::Dynamic(_) | DivertTarget::Done | DivertTarget::End | DivertTarget::Empty => {
            return None;
        }
    };

    let symbol = resolve_target_symbol(
        target_name,
        current_module,
        current_flow_path,
        target_symbols,
    )?;

    Some(resolve_flow_symbol_expected_arguments(target_name, symbol))
}

pub(super) fn resolve_function_call_expected_arguments(
    name: &str,
    current_module: Option<&str>,
    current_flow_path: Option<&str>,
    target_symbols: &TargetSymbolIndex,
) -> FunctionCallArgumentResolution {
    let Some(symbol) =
        resolve_target_symbol(name, current_module, current_flow_path, target_symbols)
    else {
        return FunctionCallArgumentResolution::Missing;
    };

    if !symbol.is_function() {
        return FunctionCallArgumentResolution::NonFunction;
    }

    FunctionCallArgumentResolution::Function(resolve_flow_symbol_expected_arguments(name, symbol))
}

pub(super) fn resolve_flow_symbol_expected_arguments(
    target_name: &str,
    symbol: &FlowSymbol,
) -> ResolvedExpectedArguments {
    let qualified_module = target_name.split_once("::").map(|(module, _)| module);
    let arguments = symbol
        .arguments()
        .iter()
        .map(|parameter| ResolvedExpectedArgument {
            name: parameter.name().to_string(),
            declared_type: parameter.declared_type().map(|declared_type| {
                qualified_module
                    .map(|module| qualify_type_name_for_module(declared_type, module))
                    .unwrap_or_else(|| declared_type.clone())
            }),
        })
        .collect();

    ResolvedExpectedArguments {
        target_name: target_name.to_string(),
        arguments,
    }
}
