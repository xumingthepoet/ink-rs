use std::collections::BTreeMap;

use crate::{
    diagnostic::Diagnostic,
    parsed::{
        ExternalDeclaration, Flow, FlowArgument, Module, Object, Story, StructField, TypeName,
    },
    source::SourceSpan,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ModuleSymbolKind {
    Knot,
    Function,
    Constant,
    GlobalVariable,
    Struct,
    External,
}

impl ModuleSymbolKind {
    fn display_name(self) -> &'static str {
        match self {
            Self::Knot => "knot",
            Self::Function => "function",
            Self::Constant => "constant",
            Self::GlobalVariable => "global variable",
            Self::Struct => "struct",
            Self::External => "external",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ModuleSymbol {
    module: String,
    name: String,
    kind: ModuleSymbolKind,
    span: SourceSpan,
    declared_type: Option<TypeName>,
    signature: Option<ModuleSignature>,
    fields: Vec<ModuleStructField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ModuleSignature {
    parameters: Vec<ModuleParameter>,
    return_type: TypeName,
    has_typed_signature: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ModuleParameter {
    name: String,
    declared_type: Option<TypeName>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ModuleStructField {
    name: String,
    type_name: TypeName,
    span: SourceSpan,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleSymbolIndex {
    symbols_by_module: BTreeMap<String, BTreeMap<String, Vec<ModuleSymbol>>>,
}

pub(in crate::analysis) fn build_module_symbol_index(story: &Story) -> ModuleSymbolIndex {
    let mut index = ModuleSymbolIndex::default();
    for module in story.modules() {
        index.ensure_module(module.name());
        insert_module_weave_symbols(&mut index, module);
        insert_module_flow_symbols(&mut index, module);
    }
    index
}

pub(in crate::analysis) fn module_symbol_diagnostics(
    story: &Story,
    index: &ModuleSymbolIndex,
) -> Vec<Diagnostic> {
    index.validate_internal_consistency();

    let mut diagnostics = duplicate_module_diagnostics(story);
    diagnostics.extend(index.namespace_collision_diagnostics());
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
    diagnostics
}

impl ModuleSymbol {
    fn new(
        module: impl Into<String>,
        name: impl Into<String>,
        kind: ModuleSymbolKind,
        span: SourceSpan,
    ) -> Self {
        Self {
            module: module.into(),
            name: name.into(),
            kind,
            span,
            declared_type: None,
            signature: None,
            fields: Vec::new(),
        }
    }

    fn with_declared_type(mut self, declared_type: TypeName) -> Self {
        self.declared_type = Some(declared_type);
        self
    }

    fn with_optional_declared_type(mut self, declared_type: Option<TypeName>) -> Self {
        self.declared_type = declared_type;
        self
    }

    fn with_signature(mut self, signature: ModuleSignature) -> Self {
        self.signature = Some(signature);
        self
    }

    fn with_fields(mut self, fields: Vec<ModuleStructField>) -> Self {
        self.fields = fields;
        self
    }

    pub(super) fn module(&self) -> &str {
        &self.module
    }

    pub(super) fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn kind(&self) -> ModuleSymbolKind {
        self.kind
    }

    pub(super) fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(super) fn declared_type(&self) -> Option<&TypeName> {
        self.declared_type.as_ref()
    }

    pub(super) fn signature(&self) -> Option<&ModuleSignature> {
        self.signature.as_ref()
    }

    pub(super) fn fields(&self) -> &[ModuleStructField] {
        &self.fields
    }
}

impl ModuleSignature {
    fn from_flow(flow: &Flow) -> Self {
        Self {
            parameters: flow
                .arguments()
                .iter()
                .map(ModuleParameter::from_flow_argument)
                .collect(),
            return_type: flow.return_type().clone(),
            has_typed_signature: flow.has_typed_signature(),
        }
    }

    fn from_external(external: &ExternalDeclaration) -> Self {
        Self {
            parameters: external
                .argument_names()
                .iter()
                .zip(external.argument_types())
                .map(|(name, declared_type)| {
                    ModuleParameter::new(name.clone(), Some(declared_type.clone()))
                })
                .collect(),
            return_type: external.return_type().clone(),
            has_typed_signature: true,
        }
    }

    pub(super) fn parameters(&self) -> &[ModuleParameter] {
        &self.parameters
    }

    pub(super) fn return_type(&self) -> &TypeName {
        &self.return_type
    }

    pub(super) fn has_typed_signature(&self) -> bool {
        self.has_typed_signature
    }
}

impl ModuleParameter {
    fn new(name: impl Into<String>, declared_type: Option<TypeName>) -> Self {
        Self {
            name: name.into(),
            declared_type,
        }
    }

    fn from_flow_argument(argument: &FlowArgument) -> Self {
        Self::new(argument.name(), argument.declared_type().cloned())
    }

    pub(super) fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn declared_type(&self) -> Option<&TypeName> {
        self.declared_type.as_ref()
    }
}

impl ModuleStructField {
    fn from_struct_field(field: &StructField) -> Self {
        Self {
            name: field.name().to_string(),
            type_name: field.type_name().clone(),
            span: field.span().clone(),
        }
    }

    pub(super) fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn type_name(&self) -> &TypeName {
        &self.type_name
    }

    pub(super) fn span(&self) -> &SourceSpan {
        &self.span
    }
}

impl ModuleSymbolIndex {
    fn ensure_module(&mut self, module: impl Into<String>) {
        self.symbols_by_module.entry(module.into()).or_default();
    }

    fn insert(&mut self, symbol: ModuleSymbol) {
        self.symbols_by_module
            .entry(symbol.module.clone())
            .or_default()
            .entry(symbol.name.clone())
            .or_default()
            .push(symbol);
    }

    pub(super) fn get(&self, module: &str, name: &str) -> Option<&ModuleSymbol> {
        self.symbols_named(module, name).first()
    }

    pub(super) fn symbols_named(&self, module: &str, name: &str) -> &[ModuleSymbol] {
        self.symbols_by_module
            .get(module)
            .and_then(|symbols| symbols.get(name))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub(super) fn symbols_for_module(
        &self,
        module: &str,
    ) -> Option<&BTreeMap<String, Vec<ModuleSymbol>>> {
        self.symbols_by_module.get(module)
    }

    fn validate_internal_consistency(&self) {
        for (module_name, symbols) in &self.symbols_by_module {
            debug_assert!(self.symbols_for_module(module_name).is_some());
            for (symbol_name, entries) in symbols {
                debug_assert_eq!(
                    self.symbols_named(module_name, symbol_name).len(),
                    entries.len()
                );
                debug_assert!(self.get(module_name, symbol_name).is_some());

                for symbol in entries {
                    debug_assert_eq!(symbol.module(), module_name);
                    debug_assert_eq!(symbol.name(), symbol_name);
                    let _ = symbol.kind();
                    let _ = symbol.span();
                    let _ = symbol.declared_type();
                    if let Some(signature) = symbol.signature() {
                        let _ = signature.return_type();
                        let _ = signature.has_typed_signature();
                        for parameter in signature.parameters() {
                            let _ = parameter.name();
                            let _ = parameter.declared_type();
                        }
                    }
                    for field in symbol.fields() {
                        let _ = field.name();
                        let _ = field.type_name();
                        let _ = field.span();
                    }
                }
            }
        }
    }

    fn namespace_collision_diagnostics(&self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (module_name, symbols) in &self.symbols_by_module {
            for entries in symbols.values() {
                let Some(first) = entries.first() else {
                    continue;
                };
                for duplicate in entries.iter().skip(1) {
                    diagnostics.push(Diagnostic::error(
                        duplicate.span().clone(),
                        format!(
                            "Module '{module_name}' already contains a {} named '{}'; {} declarations cannot reuse that name",
                            first.kind().display_name(),
                            duplicate.name(),
                            duplicate.kind().display_name(),
                        ),
                    ));
                }
            }
        }

        diagnostics
    }
}

fn duplicate_module_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let mut first_seen = BTreeMap::new();
    let mut diagnostics = Vec::new();

    for module in story.modules() {
        if first_seen
            .insert(module.name().to_string(), module.name_span().clone())
            .is_some()
        {
            diagnostics.push(Diagnostic::error(
                module.name_span().clone(),
                format!(
                    "Module '{}' is already declared in this compilation",
                    module.name()
                ),
            ));
        }
    }

    diagnostics
}

fn insert_module_weave_symbols(index: &mut ModuleSymbolIndex, module: &Module) {
    for object in module.weave().content() {
        match object {
            Object::ConstantDeclaration(declaration) => {
                index.insert(
                    ModuleSymbol::new(
                        module.name(),
                        declaration.name(),
                        ModuleSymbolKind::Constant,
                        declaration.span().clone(),
                    )
                    .with_declared_type(declaration.declared_type().clone()),
                );
            }
            Object::VariableAssignment(assignment) if assignment.is_global() => {
                index.insert(
                    ModuleSymbol::new(
                        module.name(),
                        assignment.name(),
                        ModuleSymbolKind::GlobalVariable,
                        assignment.span().clone(),
                    )
                    .with_optional_declared_type(assignment.declared_type().cloned()),
                );
            }
            Object::StructDeclaration(declaration) => {
                index.insert(
                    ModuleSymbol::new(
                        module.name(),
                        declaration.name(),
                        ModuleSymbolKind::Struct,
                        declaration.span().clone(),
                    )
                    .with_fields(
                        declaration
                            .fields()
                            .iter()
                            .map(ModuleStructField::from_struct_field)
                            .collect(),
                    ),
                );
            }
            Object::ExternalDeclaration(external) => {
                index.insert(
                    ModuleSymbol::new(
                        module.name(),
                        external.name(),
                        ModuleSymbolKind::External,
                        external.span().clone(),
                    )
                    .with_signature(ModuleSignature::from_external(external)),
                );
            }
            _ => {}
        }
    }
}

fn insert_module_flow_symbols(index: &mut ModuleSymbolIndex, module: &Module) {
    for flow in module.flows() {
        let kind = if flow.is_function() {
            ModuleSymbolKind::Function
        } else {
            ModuleSymbolKind::Knot
        };
        index.insert(
            ModuleSymbol::new(module.name(), flow.name(), kind, flow.span().clone())
                .with_signature(ModuleSignature::from_flow(flow)),
        );
    }
}
