use std::collections::{BTreeMap, BTreeSet};

use crate::{
    diagnostic::Diagnostic,
    parsed::{
        AssignmentTarget, ContentList, DivertTarget, Expression, Flow, Object, QualifiedName,
        Story, TypeName,
    },
    source::SourceSpan,
};

use super::{sort_diagnostics, ModuleSymbolIndex};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleImportIndex {
    allowed_symbols: BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
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

pub(in crate::analysis) fn module_import_diagnostics(
    story: &Story,
    symbol_index: &ModuleSymbolIndex,
    import_index: &ModuleImportIndex,
) -> Vec<Diagnostic> {
    let qualified_uses = collect_qualified_uses(story);
    let qualified_use_sets = qualified_use_sets_by_module(&qualified_uses);
    let mut diagnostics = Vec::new();

    for module in story.modules() {
        if let Some(uses) = qualified_uses.get(module.name()) {
            diagnostics.extend(qualified_import_use_diagnostics(
                module.name(),
                uses,
                symbol_index,
                import_index,
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
                    "Qualified reference '{}::{}' requires a direct import in module '{}': FROM {} IMPORT {}",
                    qualified_use.source_module,
                    qualified_use.symbol,
                    current_module,
                    qualified_use.source_module,
                    qualified_use.symbol
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
        Object::EnumDeclaration(_) => {}
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
    name: &QualifiedName,
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
