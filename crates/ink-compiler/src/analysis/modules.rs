use std::collections::{BTreeMap, BTreeSet};

use crate::{
    diagnostic::Diagnostic,
    parsed::{
        AssignmentTarget, ContentList, DivertTarget, Expression, ExternalDeclaration, Flow,
        FlowArgument, Module, Object, Story, StructField, TypeName,
    },
    source::SourceSpan,
};

use super::{span::object_span, ModuleEntryPoint};

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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleDependencyGraph {
    direct_dependencies: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleImportIndex {
    allowed_symbols: BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleReachability {
    entry_module: Option<String>,
    reachable_modules: BTreeSet<String>,
}

pub(super) fn build_module_symbol_index(story: &Story) -> ModuleSymbolIndex {
    let mut index = ModuleSymbolIndex::default();
    for module in story.modules() {
        index.ensure_module(module.name());
        insert_module_weave_symbols(&mut index, module);
        insert_module_flow_symbols(&mut index, module);
    }
    index
}

pub fn build_module_dependency_graph(story: &Story) -> ModuleDependencyGraph {
    let direct_dependencies = collect_import_dependencies(story)
        .into_iter()
        .map(|(module, dependencies)| {
            (
                module,
                dependencies
                    .into_iter()
                    .map(|dependency| dependency.module)
                    .collect(),
            )
        })
        .collect();

    ModuleDependencyGraph {
        direct_dependencies,
    }
}

pub fn build_module_import_index(story: &Story) -> ModuleImportIndex {
    let mut index = ModuleImportIndex::default();

    for module in story.modules() {
        index.ensure_module(module.name());
        for import in module.imports() {
            for imported_name in import.imported_names() {
                index.insert_allowed_symbol(
                    module.name(),
                    import.source_module(),
                    imported_name.name(),
                );
            }
        }
    }

    index
}

pub fn build_module_reachability(
    graph: &ModuleDependencyGraph,
    entry_point: Option<&ModuleEntryPoint>,
) -> ModuleReachability {
    let Some(entry_point) = entry_point else {
        return ModuleReachability::default();
    };

    let mut reachable_modules = BTreeSet::new();
    collect_reachable_modules(&entry_point.module, graph, &mut reachable_modules);

    ModuleReachability {
        entry_module: Some(entry_point.module.clone()),
        reachable_modules,
    }
}

pub(super) fn module_symbol_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let index = build_module_symbol_index(story);
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

pub(super) fn module_dependency_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let dependencies = collect_import_dependencies(story);
    module_dependency_cycle_diagnostics(&dependencies)
}

pub(super) fn mixed_root_module_diagnostics(story: &Story) -> Vec<Diagnostic> {
    if story.modules().is_empty() {
        return Vec::new();
    }

    if let Some(object) = story.root_weave().content().first() {
        return vec![mixed_root_module_diagnostic(object_span(object))];
    }

    if let Some(flow) = story.flows().first() {
        return vec![mixed_root_module_diagnostic(flow.span().clone())];
    }

    Vec::new()
}

pub(super) fn module_import_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let symbol_index = build_module_symbol_index(story);
    let import_index = build_module_import_index(story);
    let qualified_uses = collect_qualified_uses(story);
    let qualified_use_sets = qualified_use_sets_by_module(&qualified_uses);
    let mut diagnostics = Vec::new();

    for module in story.modules() {
        if let Some(uses) = qualified_uses.get(module.name()) {
            diagnostics.extend(qualified_import_use_diagnostics(
                module.name(),
                uses,
                &symbol_index,
                &import_index,
            ));
        }

        for import in module.imports() {
            if symbol_index
                .symbols_for_module(import.source_module())
                .is_none()
            {
                diagnostics.push(Diagnostic::error(
                    import.source_module_span().clone(),
                    format!(
                        "Imported module '{}' does not exist",
                        import.source_module()
                    ),
                ));
                continue;
            }

            for imported_name in import.imported_names() {
                if symbol_index
                    .get(import.source_module(), imported_name.name())
                    .is_some()
                {
                    if !qualified_use_sets.get(module.name()).is_some_and(|uses| {
                        uses.contains(&(
                            import.source_module().to_string(),
                            imported_name.name().to_string(),
                        ))
                    }) {
                        diagnostics.push(Diagnostic::warning(
                            imported_name.span().clone(),
                            format!(
                                "Imported symbol '{}::{}' is never used",
                                import.source_module(),
                                imported_name.name()
                            ),
                        ));
                    }
                    continue;
                }

                if module_contains_stitch_named(story, import.source_module(), imported_name.name())
                {
                    diagnostics.push(Diagnostic::error(
                        imported_name.span().clone(),
                        format!(
                            "Cannot import stitch '{}' from module '{}'; stitches are scoped to their parent knot",
                            imported_name.name(),
                            import.source_module()
                        ),
                    ));
                } else {
                    diagnostics.push(Diagnostic::error(
                        imported_name.span().clone(),
                        format!(
                            "Module '{}' does not define importable symbol '{}'",
                            import.source_module(),
                            imported_name.name()
                        ),
                    ));
                }
            }
        }
    }

    sort_diagnostics(&mut diagnostics);
    diagnostics
}

fn mixed_root_module_diagnostic(span: SourceSpan) -> Diagnostic {
    Diagnostic::error(
        span,
        "Explicit module compilation cannot be mixed with root story content or top-level flows",
    )
}

fn qualified_import_use_diagnostics(
    current_module: &str,
    uses: &[QualifiedUse],
    symbol_index: &ModuleSymbolIndex,
    import_index: &ModuleImportIndex,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut reported = BTreeSet::new();

    for qualified_use in uses {
        let key = (
            qualified_use.source_module.as_str(),
            qualified_use.symbol.as_str(),
        );
        if !reported.insert((
            qualified_use.source_module.clone(),
            qualified_use.symbol.clone(),
        )) {
            continue;
        }

        if qualified_use.source_module == current_module {
            if symbol_index
                .get(current_module, &qualified_use.symbol)
                .is_none()
            {
                diagnostics.push(Diagnostic::error(
                    qualified_use.span.clone(),
                    format!(
                        "Module '{current_module}' does not define importable symbol '{}'",
                        qualified_use.symbol
                    ),
                ));
            }
            continue;
        }

        if symbol_index
            .symbols_for_module(&qualified_use.source_module)
            .is_none()
        {
            diagnostics.push(Diagnostic::error(
                qualified_use.span.clone(),
                format!(
                    "Qualified reference '{}::{}' uses unknown module '{}'",
                    qualified_use.source_module, qualified_use.symbol, qualified_use.source_module
                ),
            ));
            continue;
        }

        if !import_index.allows(current_module, key.0, key.1) {
            diagnostics.push(Diagnostic::error(
                qualified_use.span.clone(),
                format!(
                    "Qualified reference '{}::{}' requires a direct import in module '{}': IMPORT {} FROM {}",
                    qualified_use.source_module,
                    qualified_use.symbol,
                    current_module,
                    qualified_use.symbol,
                    qualified_use.source_module
                ),
            ));
        }
    }

    diagnostics
}

fn qualified_use_sets_by_module(
    qualified_uses: &BTreeMap<String, Vec<QualifiedUse>>,
) -> BTreeMap<String, BTreeSet<(String, String)>> {
    qualified_uses
        .iter()
        .map(|(module, uses)| {
            (
                module.clone(),
                uses.iter()
                    .map(|qualified_use| {
                        (
                            qualified_use.source_module.clone(),
                            qualified_use.symbol.clone(),
                        )
                    })
                    .collect(),
            )
        })
        .collect()
}

pub(super) fn unreachable_module_diagnostics(
    story: &Story,
    reachability: &ModuleReachability,
) -> Vec<Diagnostic> {
    if reachability.entry_module().is_none() {
        return Vec::new();
    }

    let entry = reachability
        .entry_module()
        .expect("entry module checked above");
    let mut diagnostics = story
        .modules()
        .iter()
        .filter(|module| !reachability.is_reachable(module.name()))
        .map(|module| {
            Diagnostic::warning(
                module.name_span().clone(),
                format!(
                    "Module '{}' is not reachable from entry point '{}::main'",
                    module.name(),
                    entry
                ),
            )
        })
        .collect::<Vec<_>>();
    sort_diagnostics(&mut diagnostics);
    diagnostics
}

pub(super) fn module_entry_point(story: &Story) -> Option<ModuleEntryPoint> {
    let mains = module_main_knots(story);
    (mains.len() == 1).then(|| ModuleEntryPoint::new(mains[0].module.clone(), "main"))
}

impl ModuleDependencyGraph {
    pub fn modules(&self) -> impl Iterator<Item = &str> {
        self.direct_dependencies.keys().map(String::as_str)
    }

    pub fn dependencies_for(&self, module: &str) -> &[String] {
        self.direct_dependencies
            .get(module)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn contains_dependency(&self, module: &str, dependency: &str) -> bool {
        self.dependencies_for(module)
            .iter()
            .any(|candidate| candidate == dependency)
    }
}

impl ModuleImportIndex {
    fn ensure_module(&mut self, module: impl Into<String>) {
        self.allowed_symbols.entry(module.into()).or_default();
    }

    fn insert_allowed_symbol(
        &mut self,
        importing_module: impl Into<String>,
        source_module: impl Into<String>,
        symbol: impl Into<String>,
    ) {
        self.allowed_symbols
            .entry(importing_module.into())
            .or_default()
            .entry(source_module.into())
            .or_default()
            .insert(symbol.into());
    }

    pub fn imports_from(
        &self,
        importing_module: &str,
        source_module: &str,
    ) -> Option<&BTreeSet<String>> {
        self.allowed_symbols
            .get(importing_module)
            .and_then(|imports| imports.get(source_module))
    }

    pub fn allows(&self, importing_module: &str, source_module: &str, symbol: &str) -> bool {
        self.imports_from(importing_module, source_module)
            .is_some_and(|symbols| symbols.contains(symbol))
    }
}

impl ModuleReachability {
    pub fn entry_module(&self) -> Option<&str> {
        self.entry_module.as_deref()
    }

    pub fn is_reachable(&self, module: &str) -> bool {
        self.reachable_modules.contains(module)
    }

    pub fn reachable_modules(&self) -> impl Iterator<Item = &str> {
        self.reachable_modules.iter().map(String::as_str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModuleDependency {
    module: String,
    span: SourceSpan,
}

fn collect_import_dependencies(story: &Story) -> BTreeMap<String, Vec<ModuleDependency>> {
    let mut dependencies_by_module = BTreeMap::new();

    for module in story.modules() {
        let mut dependencies = BTreeMap::<String, SourceSpan>::new();
        for import in module.imports() {
            dependencies
                .entry(import.source_module().to_string())
                .or_insert_with(|| import.source_module_span().clone());
        }
        dependencies_by_module.insert(
            module.name().to_string(),
            dependencies
                .into_iter()
                .map(|(module, span)| ModuleDependency { module, span })
                .collect(),
        );
    }

    dependencies_by_module
}

fn collect_reachable_modules(
    module: &str,
    graph: &ModuleDependencyGraph,
    reachable_modules: &mut BTreeSet<String>,
) {
    if !reachable_modules.insert(module.to_string()) {
        return;
    }

    for dependency in graph.dependencies_for(module) {
        collect_reachable_modules(dependency, graph, reachable_modules);
    }
}

fn module_dependency_cycle_diagnostics(
    dependencies_by_module: &BTreeMap<String, Vec<ModuleDependency>>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut reported_cycles = BTreeSet::new();

    for module in dependencies_by_module.keys() {
        let mut path = Vec::new();
        collect_cycles_from_module(
            module,
            dependencies_by_module,
            &mut path,
            &mut reported_cycles,
            &mut diagnostics,
        );
    }

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

fn collect_cycles_from_module(
    current: &str,
    dependencies_by_module: &BTreeMap<String, Vec<ModuleDependency>>,
    path: &mut Vec<String>,
    reported_cycles: &mut BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    path.push(current.to_string());

    if let Some(dependencies) = dependencies_by_module.get(current) {
        for dependency in dependencies {
            if !dependencies_by_module.contains_key(&dependency.module) {
                continue;
            }

            if let Some(cycle_start) = path.iter().position(|module| module == &dependency.module) {
                let mut cycle = path[cycle_start..].to_vec();
                cycle.push(dependency.module.clone());
                let key = canonical_cycle_key(&cycle);
                if reported_cycles.insert(key) {
                    diagnostics.push(Diagnostic::error(
                        dependency.span.clone(),
                        format!(
                            "Cyclic module import detected: {}",
                            format_module_path(&cycle)
                        ),
                    ));
                }
                continue;
            }

            collect_cycles_from_module(
                &dependency.module,
                dependencies_by_module,
                path,
                reported_cycles,
                diagnostics,
            );
        }
    }

    path.pop();
}

fn canonical_cycle_key(cycle: &[String]) -> String {
    debug_assert!(cycle.len() >= 2);
    let ring = &cycle[..cycle.len() - 1];
    let mut rotations = Vec::new();

    for start in 0..ring.len() {
        let mut rotation = Vec::new();
        for offset in 0..ring.len() {
            rotation.push(ring[(start + offset) % ring.len()].as_str());
        }
        rotations.push(rotation.join("\u{0}"));
    }

    rotations.into_iter().min().unwrap_or_default()
}

fn format_module_path(path: &[String]) -> String {
    path.join(" -> ")
}

fn module_contains_stitch_named(story: &Story, module_name: &str, stitch_name: &str) -> bool {
    story
        .modules()
        .iter()
        .find(|module| module.name() == module_name)
        .is_some_and(|module| {
            module.flows().iter().any(|flow| {
                flow.child_flows()
                    .iter()
                    .any(|child| child.name() == stitch_name)
            })
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QualifiedUse {
    source_module: String,
    symbol: String,
    span: SourceSpan,
}

fn collect_qualified_uses(story: &Story) -> BTreeMap<String, Vec<QualifiedUse>> {
    let mut uses = BTreeMap::new();
    for module in story.modules() {
        collect_qualified_uses_in_objects(module.name(), module.weave().content(), &mut uses);
        for flow in module.flows() {
            collect_qualified_uses_in_flow(module.name(), flow, &mut uses);
        }
    }
    uses
}

fn collect_qualified_uses_in_flow(
    current_module: &str,
    flow: &Flow,
    uses: &mut BTreeMap<String, Vec<QualifiedUse>>,
) {
    for argument in flow.arguments() {
        if let Some(type_name) = argument.declared_type() {
            collect_qualified_uses_in_type_name(current_module, type_name, uses);
        }
    }
    collect_qualified_uses_in_type_name(current_module, flow.return_type(), uses);
    collect_qualified_uses_in_objects(current_module, flow.weave().content(), uses);
    for child in flow.child_flows() {
        collect_qualified_uses_in_flow(current_module, child, uses);
    }
}

fn collect_qualified_uses_in_content_list(
    current_module: &str,
    content: &ContentList,
    uses: &mut BTreeMap<String, Vec<QualifiedUse>>,
) {
    collect_qualified_uses_in_objects(current_module, content.objects(), uses);
}

fn collect_qualified_uses_in_objects(
    current_module: &str,
    objects: &[Object],
    uses: &mut BTreeMap<String, Vec<QualifiedUse>>,
) {
    for object in objects {
        collect_qualified_uses_in_object(current_module, object, uses);
    }
}

fn collect_qualified_uses_in_object(
    current_module: &str,
    object: &Object,
    uses: &mut BTreeMap<String, Vec<QualifiedUse>>,
) {
    match object {
        Object::Expression(expression) | Object::LogicLine(expression) => {
            collect_qualified_uses_in_expression(current_module, expression, uses);
        }
        Object::ConstantDeclaration(declaration) => {
            collect_qualified_uses_in_type_name(current_module, declaration.declared_type(), uses);
            collect_qualified_uses_in_expression(current_module, declaration.expression(), uses);
        }
        Object::ContentList(content) => {
            collect_qualified_uses_in_content_list(current_module, content, uses)
        }
        Object::Conditional(conditional) => {
            if let Some(condition) = conditional.initial_condition() {
                collect_qualified_uses_in_expression(current_module, condition, uses);
            }
            for branch in conditional.branches() {
                if let Some(condition) = branch.own_condition() {
                    collect_qualified_uses_in_expression(current_module, condition, uses);
                }
                collect_qualified_uses_in_objects(current_module, branch.content().content(), uses);
            }
        }
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                collect_qualified_uses_in_content_list(current_module, element, uses);
            }
        }
        Object::Choice(choice) => {
            if let Some(condition) = choice.condition() {
                collect_qualified_uses_in_expression(current_module, condition, uses);
            }
            if let Some(content) = choice.start_content() {
                collect_qualified_uses_in_content_list(current_module, content, uses);
            }
            collect_qualified_uses_in_content_list(current_module, choice.inner_content(), uses);
        }
        Object::Divert(divert) => {
            collect_qualified_uses_in_divert_target(
                current_module,
                divert.target(),
                divert.span(),
                uses,
            );
            for argument in divert.arguments() {
                collect_qualified_uses_in_expression(current_module, argument, uses);
            }
        }
        Object::TunnelOnwards(tunnel_onwards) => {
            if let Some(target) = tunnel_onwards.override_target() {
                collect_qualified_uses_in_divert_target(
                    current_module,
                    target,
                    tunnel_onwards.span(),
                    uses,
                );
            }
            for argument in tunnel_onwards.arguments() {
                collect_qualified_uses_in_expression(current_module, argument, uses);
            }
        }
        Object::VariableAssignment(assignment) => {
            if let Some(type_name) = assignment.declared_type() {
                collect_qualified_uses_in_type_name(current_module, type_name, uses);
            }
            collect_qualified_uses_in_assignment_target(current_module, assignment.target(), uses);
            if let Some(expression) = assignment.expression() {
                collect_qualified_uses_in_expression(current_module, expression, uses);
            }
        }
        Object::IncDec(inc_dec) => {
            collect_qualified_uses_in_assignment_target(current_module, inc_dec.target(), uses);
            collect_qualified_uses_in_expression(current_module, inc_dec.expression(), uses);
        }
        Object::Return(ret) => {
            if let Some(expression) = ret.returned_expression() {
                collect_qualified_uses_in_expression(current_module, expression, uses);
            }
        }
        Object::ExternalDeclaration(external) => {
            for type_name in external.argument_types() {
                collect_qualified_uses_in_type_name(current_module, type_name, uses);
            }
            collect_qualified_uses_in_type_name(current_module, external.return_type(), uses);
        }
        Object::StructDeclaration(declaration) => {
            for field in declaration.fields() {
                collect_qualified_uses_in_type_name(current_module, field.type_name(), uses);
            }
        }
        Object::Weave(weave) => {
            collect_qualified_uses_in_objects(current_module, weave.content(), uses)
        }
        Object::AuthorWarning(_)
        | Object::Gather(_)
        | Object::Glue(_)
        | Object::Tag(_)
        | Object::Text(_) => {}
    }
}

fn collect_qualified_uses_in_divert_target(
    current_module: &str,
    target: &DivertTarget,
    fallback_span: &SourceSpan,
    uses: &mut BTreeMap<String, Vec<QualifiedUse>>,
) {
    match target {
        DivertTarget::Path(path) => {
            record_qualified_use(current_module, path, fallback_span.clone(), uses)
        }
        DivertTarget::QualifiedPath(path) => record_qualified_name_use(current_module, path, uses),
        DivertTarget::Dynamic(expression) => {
            collect_qualified_uses_in_expression(current_module, expression, uses)
        }
        DivertTarget::Done | DivertTarget::End | DivertTarget::Empty => {}
    }
}

fn collect_qualified_uses_in_expression(
    current_module: &str,
    expression: &Expression,
    uses: &mut BTreeMap<String, Vec<QualifiedUse>>,
) {
    match expression {
        Expression::VariableReference(name) | Expression::DivertTarget(name) => {
            record_qualified_use(current_module, name, fallback_qualified_use_span(), uses);
        }
        Expression::QualifiedReference(name) => {
            record_qualified_name_use(current_module, name, uses);
        }
        Expression::FunctionCall { name, args } => {
            record_qualified_use(current_module, name, fallback_qualified_use_span(), uses);
            for arg in args {
                collect_qualified_uses_in_expression(current_module, arg, uses);
            }
        }
        Expression::QualifiedFunctionCall { name, args } => {
            record_qualified_name_use(current_module, name, uses);
            for arg in args {
                collect_qualified_uses_in_expression(current_module, arg, uses);
            }
        }
        Expression::StringContent(content) => {
            collect_qualified_uses_in_content_list(current_module, content, uses)
        }
        Expression::ArrayLiteral(elements) | Expression::MultipleCondition(elements) => {
            for element in elements {
                collect_qualified_uses_in_expression(current_module, element, uses);
            }
        }
        Expression::StructLiteral(fields) => {
            for field in fields {
                collect_qualified_uses_in_expression(current_module, field.expression(), uses);
            }
        }
        Expression::FieldAccess { base, .. } => {
            collect_qualified_uses_in_expression(current_module, base, uses);
        }
        Expression::IndexAccess { base, index } => {
            collect_qualified_uses_in_expression(current_module, base, uses);
            collect_qualified_uses_in_expression(current_module, index, uses);
        }
        Expression::Binary { left, right, .. } => {
            collect_qualified_uses_in_expression(current_module, left, uses);
            collect_qualified_uses_in_expression(current_module, right, uses);
        }
        Expression::Unary { expression, .. } => {
            collect_qualified_uses_in_expression(current_module, expression, uses);
        }
        Expression::String(_)
        | Expression::NumberBool(_)
        | Expression::NumberFloat(_)
        | Expression::NumberInt(_) => {}
    }
}

fn collect_qualified_uses_in_assignment_target(
    current_module: &str,
    target: &AssignmentTarget,
    uses: &mut BTreeMap<String, Vec<QualifiedUse>>,
) {
    match target {
        AssignmentTarget::Variable(name) => {
            record_qualified_use(current_module, name, fallback_qualified_use_span(), uses)
        }
        AssignmentTarget::QualifiedVariable(name) => {
            record_qualified_name_use(current_module, name, uses)
        }
        AssignmentTarget::FieldAccess { base, .. } => {
            collect_qualified_uses_in_assignment_target(current_module, base, uses);
        }
        AssignmentTarget::IndexAccess { base, index } => {
            collect_qualified_uses_in_assignment_target(current_module, base, uses);
            collect_qualified_uses_in_expression(current_module, index, uses);
        }
    }
}

fn collect_qualified_uses_in_type_name(
    current_module: &str,
    type_name: &TypeName,
    uses: &mut BTreeMap<String, Vec<QualifiedUse>>,
) {
    match type_name {
        TypeName::QualifiedStruct(name) => record_qualified_name_use(current_module, name, uses),
        TypeName::Array(element_type) => {
            collect_qualified_uses_in_type_name(current_module, element_type, uses)
        }
        TypeName::Primitive(_) | TypeName::Struct(_) | TypeName::Void => {}
    }
}

fn record_qualified_name_use(
    current_module: &str,
    name: &crate::parsed::QualifiedName,
    uses: &mut BTreeMap<String, Vec<QualifiedUse>>,
) {
    record_qualified_parts(
        current_module,
        name.module(),
        name.symbol(),
        name.module_span().clone(),
        uses,
    );
}

fn record_qualified_use(
    current_module: &str,
    source: &str,
    span: SourceSpan,
    uses: &mut BTreeMap<String, Vec<QualifiedUse>>,
) {
    let Some((module, symbol)) = source.split_once("::") else {
        return;
    };
    if module.is_empty() || symbol.is_empty() || symbol.contains("::") {
        return;
    }
    let symbol = symbol.split('.').next().unwrap_or(symbol);
    record_qualified_parts(current_module, module, symbol, span, uses);
}

fn record_qualified_parts(
    current_module: &str,
    source_module: &str,
    symbol: &str,
    span: SourceSpan,
    uses: &mut BTreeMap<String, Vec<QualifiedUse>>,
) {
    if source_module.is_empty() || symbol.is_empty() || symbol.contains("::") {
        return;
    }
    uses.entry(current_module.to_string())
        .or_default()
        .push(QualifiedUse {
            source_module: source_module.to_string(),
            symbol: symbol.to_string(),
            span,
        });
}

fn fallback_qualified_use_span() -> SourceSpan {
    SourceSpan::new(None, 1, 1)
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

pub(super) fn module_entry_point_diagnostics(story: &Story) -> Vec<Diagnostic> {
    if story.modules().is_empty() {
        return Vec::new();
    }

    let mains = module_main_knots(story);
    match mains.len() {
        0 => vec![Diagnostic::error(
            story.modules()[0].span().clone(),
            "Explicit module compilation requires exactly one module to define a knot named 'main'",
        )],
        1 => Vec::new(),
        _ => mains
            .iter()
            .skip(1)
            .map(|main| {
                Diagnostic::error(
                    main.span.clone(),
                    "Multiple 'main' knots are declared; runnable entry point must be unique",
                )
            })
            .collect(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModuleMainKnot {
    module: String,
    span: SourceSpan,
}

fn module_main_knots(story: &Story) -> Vec<ModuleMainKnot> {
    story
        .modules()
        .iter()
        .flat_map(|module| {
            module
                .flows()
                .iter()
                .filter(|flow| !flow.is_function() && flow.name() == "main")
                .map(|flow| ModuleMainKnot {
                    module: module.name().to_string(),
                    span: flow.span().clone(),
                })
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use crate::parsed::TypeName;
    use crate::{Compiler, DiagnosticSeverity, SourceInput};

    use super::{
        super::test_support::{assert_single_diagnostic, parse_story},
        *,
    };

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

        let external = index.get("game", "play").expect("external symbol");
        assert_eq!(external.kind(), ModuleSymbolKind::External);
        assert_eq!(external.span().line, 8);
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
        assert_eq!(function.span().line, 9);
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
        assert_eq!(knot.span().line, 11);
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
             IMPORT sword FROM items\n\
             IMPORT play FROM audio\n\
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
             IMPORT sword FROM items\n\
             IMPORT play FROM audio\n\
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
             IMPORT sword FROM items\n\
             == main ==\n\
             -> END\n\
             === module items ===\n\
             IMPORT main FROM game\n\
             == sword ==\n\
             -> END",
        );

        let diagnostics = module_dependency_diagnostics(&story);

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
             IMPORT sword FROM items\n\
             == main ==\n\
             -> END\n\
             === module items ===\n\
             IMPORT play FROM audio\n\
             == sword ==\n\
             -> END\n\
             === module audio ===\n\
             IMPORT main FROM game\n\
             == play ==\n\
             -> END",
        );

        let diagnostics = module_dependency_diagnostics(&story);

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
             IMPORT sword FROM missing\n\
             == main ==\n\
             -> END",
        );

        let diagnostics = module_import_diagnostics(&story);

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
             IMPORT sword FROM items\n\
             == main ==\n\
             -> END\n\
             === module items ===\n\
             == shield ==\n\
             -> END",
        );

        let diagnostics = module_import_diagnostics(&story);

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
             IMPORT intro FROM scenes\n\
             == main ==\n\
             -> END\n\
             === module scenes ===\n\
             == opening ==\n\
             = intro\n\
             -> END",
        );

        let diagnostics = module_import_diagnostics(&story);

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
             IMPORT sword FROM items\n\
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
    fn treats_existing_qualified_divert_paths_as_import_uses() {
        let story = parse_story(
            "=== module game ===\n\
             IMPORT sword FROM items\n\
             == main ==\n\
             -> items::sword\n\
             === module items ===\n\
             == sword ==\n\
             -> END",
        );

        let diagnostics = module_import_diagnostics(&story);

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn accepts_directly_imported_qualified_references() {
        let story = parse_story(
            "=== module game ===\n\
             IMPORT helper FROM items\n\
             == main ==\n\
             ~ items::helper()\n\
             -> END\n\
             === module items ===\n\
             == function helper() => void ==\n\
             ~ return",
        );

        let diagnostics = module_import_diagnostics(&story);

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

        let diagnostics = module_import_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "Qualified reference 'items::helper' requires a direct import in module 'game': IMPORT helper FROM items"
        );
    }

    #[test]
    fn rejects_transitive_only_qualified_import_access() {
        let story = parse_story(
            "=== module game ===\n\
             IMPORT relay FROM bridge\n\
             == main ==\n\
             ~ bridge::relay()\n\
             ~ items::helper()\n\
             -> END\n\
             === module bridge ===\n\
             IMPORT helper FROM items\n\
             == function relay() => void ==\n\
             ~ items::helper()\n\
             ~ return\n\
             === module items ===\n\
             == function helper() => void ==\n\
             ~ return",
        );

        let diagnostics = module_import_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "Qualified reference 'items::helper' requires a direct import in module 'game': IMPORT helper FROM items"
        );
    }

    #[test]
    fn rejects_imports_from_wrong_module_for_qualified_use() {
        let story = parse_story(
            "=== module game ===\n\
             IMPORT helper FROM audio\n\
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

        let diagnostics = module_import_diagnostics(&story);

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

        let diagnostics = module_import_diagnostics(&story);

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

        let diagnostics = module_import_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(
            diagnostics[0].message,
            "Qualified reference 'items::helper' requires a direct import in module 'game': IMPORT helper FROM items"
        );
    }

    #[test]
    fn qualified_struct_type_references_count_as_import_uses() {
        let story = parse_story(
            "=== module game ===\n\
             IMPORT Item FROM items\n\
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

        let diagnostics = module_import_diagnostics(&story);

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

        let diagnostics = module_import_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Qualified reference 'items::Item' requires a direct import in module 'game': IMPORT Item FROM items",
        );
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
        let entry_point = module_entry_point(&story);
        let reachability = build_module_reachability(&graph, entry_point.as_ref());

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

        let diagnostics = module_symbol_diagnostics(&story);

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

        let diagnostics = module_symbol_diagnostics(&story);

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
             EXTERNAL Player() => void\n\
             == function util() => void ==\n\
             ~ return\n\
             == util ==\n\
             -> END",
        );

        let diagnostics = module_symbol_diagnostics(&story);

        assert_eq!(diagnostics.len(), 3, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].line, 3);
        assert_eq!(
            diagnostics[0].message,
            "Module 'game' already contains a constant named 'shared'; global variable declarations cannot reuse that name"
        );
        assert_eq!(diagnostics[1].line, 7);
        assert_eq!(
            diagnostics[1].message,
            "Module 'game' already contains a struct named 'Player'; external declarations cannot reuse that name"
        );
        assert_eq!(diagnostics[2].line, 10);
        assert_eq!(
            diagnostics[2].message,
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

        let diagnostics = module_symbol_diagnostics(&story);

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

        let diagnostics = module_symbol_diagnostics(&story);

        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn reports_missing_main_for_explicit_module_compilation() {
        let story = parse_story(
            "=== module library ===\n\
             == helper ==\n\
             -> END",
        );

        let diagnostics = module_entry_point_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].line, 1);
        assert_eq!(
            diagnostics[0].message,
            "Explicit module compilation requires exactly one module to define a knot named 'main'"
        );
        assert!(module_entry_point(&story).is_none());
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
             IMPORT play FROM audio\n\
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

        let diagnostics = module_entry_point_diagnostics(&story);

        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].line, 5);
        assert_eq!(
            diagnostics[0].message,
            "Multiple 'main' knots are declared; runnable entry point must be unique"
        );
        assert!(module_entry_point(&story).is_none());
    }

    #[test]
    fn legacy_sources_do_not_require_module_main_until_root_migration() {
        let story = parse_story("Line.");

        assert!(module_entry_point_diagnostics(&story).is_empty());
        assert!(module_entry_point(&story).is_none());
    }
}
