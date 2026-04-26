use std::collections::{BTreeMap, HashMap};

use crate::parsed::{FlowArgument, TypeName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FlowSymbol {
    is_function: bool,
    arguments: Vec<ParameterSymbol>,
    return_type: TypeName,
    has_typed_signature: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParameterSymbol {
    name: String,
    declared_type: Option<TypeName>,
}

impl FlowSymbol {
    pub(super) fn new(
        is_function: bool,
        arguments: Vec<ParameterSymbol>,
        return_type: TypeName,
        has_typed_signature: bool,
    ) -> Self {
        Self {
            is_function,
            arguments,
            return_type,
            has_typed_signature,
        }
    }

    pub(super) fn label() -> Self {
        Self::new(false, Vec::new(), TypeName::void(), false)
    }

    pub(super) fn is_function(&self) -> bool {
        self.is_function
    }

    pub(super) fn arguments(&self) -> &[ParameterSymbol] {
        &self.arguments
    }

    pub(super) fn return_type(&self) -> &TypeName {
        &self.return_type
    }

    pub(super) fn has_typed_signature(&self) -> bool {
        self.has_typed_signature
    }
}

impl ParameterSymbol {
    pub(super) fn new(name: impl Into<String>, declared_type: Option<TypeName>) -> Self {
        Self {
            name: name.into(),
            declared_type,
        }
    }

    pub(super) fn from_flow_argument(argument: &FlowArgument) -> Self {
        Self::new(argument.name(), argument.declared_type().cloned())
    }

    pub(super) fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn declared_type(&self) -> Option<&TypeName> {
        self.declared_type.as_ref()
    }
}

pub(super) type TargetSymbolIndex = HashMap<String, FlowSymbol>;
pub(super) type StructTypeIndex = BTreeMap<String, StructTypeSymbol>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StructTypeSymbol {
    fields: BTreeMap<String, TypeName>,
}

impl StructTypeSymbol {
    pub(super) fn new(fields: BTreeMap<String, TypeName>) -> Self {
        Self { fields }
    }

    pub(super) fn fields(&self) -> &BTreeMap<String, TypeName> {
        &self.fields
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct VariableSymbol {
    declared_type: Option<TypeName>,
    kind: VariableSymbolKind,
}

impl VariableSymbol {
    pub(super) fn new(declared_type: Option<TypeName>, kind: VariableSymbolKind) -> Self {
        Self {
            declared_type,
            kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VariableSymbolKind {
    Constant,
    GlobalVariable,
    Local,
}

#[derive(Debug, Default)]
pub(super) struct VariableScopeIndex {
    globals_by_module: HashMap<Option<String>, HashMap<String, VariableSymbol>>,
    locals_by_scope: HashMap<(Option<String>, String), HashMap<String, VariableSymbol>>,
}

impl VariableScopeIndex {
    pub(super) fn insert_global(
        &mut self,
        module: Option<&str>,
        name: impl Into<String>,
        declared_type: Option<TypeName>,
        kind: VariableSymbolKind,
    ) {
        self.globals_by_module
            .entry(module.map(str::to_string))
            .or_default()
            .insert(name.into(), VariableSymbol::new(declared_type, kind));
    }

    pub(super) fn insert_local(
        &mut self,
        module: Option<&str>,
        flow_path: impl Into<String>,
        name: impl Into<String>,
        declared_type: Option<TypeName>,
    ) {
        self.locals_by_scope
            .entry((module.map(str::to_string), flow_path.into()))
            .or_default()
            .insert(
                name.into(),
                VariableSymbol::new(declared_type, VariableSymbolKind::Local),
            );
    }

    pub(super) fn contains_visible_variable(
        &self,
        name: &str,
        current_module: Option<&str>,
        current_flow_path: Option<&str>,
    ) -> bool {
        self.visible_variable_declared_type(name, current_module, current_flow_path)
            .is_some()
    }

    pub(super) fn visible_variable_declared_type(
        &self,
        name: &str,
        current_module: Option<&str>,
        current_flow_path: Option<&str>,
    ) -> Option<Option<&TypeName>> {
        if let Some(flow_path) = current_flow_path {
            if let Some(symbol) = self
                .locals_by_scope
                .get(&(current_module.map(str::to_string), flow_path.to_string()))
                .and_then(|locals| locals.get(name))
            {
                return Some(symbol.declared_type.as_ref());
            }
        }

        self.globals_by_module
            .get(&current_module.map(str::to_string))
            .and_then(|globals| globals.get(name))
            .or_else(|| {
                if current_module.is_some() {
                    None
                } else {
                    self.globals_by_module
                        .get(&None)
                        .and_then(|globals| globals.get(name))
                }
            })
            .map(|symbol| symbol.declared_type.as_ref())
    }

    pub(super) fn qualified_constant_declared_type(
        &self,
        qualified_name: &str,
    ) -> Option<Option<&TypeName>> {
        self.qualified_declared_type(qualified_name, VariableSymbolKind::Constant)
    }

    pub(super) fn qualified_global_variable_declared_type(
        &self,
        qualified_name: &str,
    ) -> Option<Option<&TypeName>> {
        self.qualified_declared_type(qualified_name, VariableSymbolKind::GlobalVariable)
    }

    fn qualified_declared_type(
        &self,
        qualified_name: &str,
        kind: VariableSymbolKind,
    ) -> Option<Option<&TypeName>> {
        let (module, name) = qualified_name.split_once("::")?;
        if module.is_empty() || name.is_empty() || name.contains("::") || name.contains('.') {
            return None;
        }

        self.globals_by_module
            .get(&Some(module.to_string()))
            .and_then(|globals| globals.get(name))
            .filter(|symbol| symbol.kind == kind)
            .map(|symbol| symbol.declared_type.as_ref())
    }
}

#[derive(Debug, Clone)]
pub(super) struct FlowContext {
    arguments: Vec<FlowArgument>,
    is_function: bool,
}

impl FlowContext {
    pub(super) fn new(arguments: Vec<FlowArgument>, is_function: bool) -> Self {
        Self {
            arguments,
            is_function,
        }
    }

    pub(super) fn arguments(&self) -> &[FlowArgument] {
        &self.arguments
    }

    pub(super) fn is_function(&self) -> bool {
        self.is_function
    }
}
