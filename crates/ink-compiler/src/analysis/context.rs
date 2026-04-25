use std::collections::{HashMap, HashSet};

use crate::parsed::FlowArgument;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct FlowSymbol {
    pub(super) is_function: bool,
}

pub(super) type TargetSymbolIndex = HashMap<String, FlowSymbol>;
pub(super) type VariableTargetIndex = HashSet<String>;

#[derive(Debug, Default)]
pub(super) struct VariableScopeIndex {
    pub(super) globals: HashSet<String>,
    pub(super) locals_by_flow_path: HashMap<String, HashSet<String>>,
}

impl VariableScopeIndex {
    pub(super) fn contains_visible_variable(
        &self,
        name: &str,
        current_flow_path: Option<&str>,
    ) -> bool {
        if let Some(flow_path) = current_flow_path {
            if self
                .locals_by_flow_path
                .get(flow_path)
                .is_some_and(|locals| locals.contains(name))
            {
                return true;
            }
        }

        self.globals.contains(name)
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
