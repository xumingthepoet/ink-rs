use crate::parsed::{
    DivertTarget, Expression, InterfaceMemberKind, InterfaceMemberSignature, TypeName,
};

use super::{
    context::{EnumTypeIndex, FlowSymbol, StructTypeIndex, TargetSymbolIndex, VariableScopeIndex},
    expression_types::{infer_expression_type, TypeInferenceError},
    interfaces::InterfaceMemberIndex,
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

pub(super) enum DynamicInterfaceSignatureError {
    Inference(TypeInferenceError),
    NonInterface(TypeName),
    MissingMember { interface_name: String },
    WrongKind { interface_name: String },
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

pub(super) fn resolve_dynamic_interface_signature(
    target: &Expression,
    member: &str,
    expected_kind: InterfaceMemberKind,
    inputs: DynamicInterfaceSignatureInputs<'_>,
    current_module: Option<&str>,
    current_flow_path: Option<&str>,
) -> Result<InterfaceMemberSignature, DynamicInterfaceSignatureError> {
    let target_type = infer_expression_type(
        target,
        inputs.variable_scopes,
        inputs.struct_types,
        inputs.enum_types,
        inputs.target_symbols,
        inputs.interface_members,
        current_module,
        current_flow_path,
    )
    .map_err(DynamicInterfaceSignatureError::Inference)?;

    let Some(interface_name) = target_type.as_interface_name() else {
        return Err(DynamicInterfaceSignatureError::NonInterface(target_type));
    };

    let Some(signature) = inputs.interface_members.member(interface_name, member) else {
        return Err(DynamicInterfaceSignatureError::MissingMember {
            interface_name: interface_name.to_string(),
        });
    };

    if signature.kind() != &expected_kind {
        return Err(DynamicInterfaceSignatureError::WrongKind {
            interface_name: interface_name.to_string(),
        });
    }

    Ok(signature.clone())
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DynamicInterfaceSignatureInputs<'a> {
    pub(super) variable_scopes: &'a VariableScopeIndex,
    pub(super) struct_types: &'a StructTypeIndex,
    pub(super) enum_types: &'a EnumTypeIndex,
    pub(super) target_symbols: &'a TargetSymbolIndex,
    pub(super) interface_members: &'a InterfaceMemberIndex,
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
