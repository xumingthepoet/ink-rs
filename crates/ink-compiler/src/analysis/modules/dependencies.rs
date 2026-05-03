use std::collections::{BTreeMap, BTreeSet};

use crate::{
    diagnostic::Diagnostic,
    parsed::{Flow, Story},
    source::SourceSpan,
};

use super::super::ModuleEntryPoint;
use super::sort_diagnostics;

#[derive(Debug, Clone, Default)]
pub struct ModuleDependencyGraph {
    direct_dependencies: BTreeMap<String, Vec<String>>,
    dependency_spans: BTreeMap<String, BTreeMap<String, SourceSpan>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleReachability {
    entry_module: Option<String>,
    reachable_modules: BTreeSet<String>,
}

pub fn build_module_dependency_graph(story: &Story) -> ModuleDependencyGraph {
    let dependency_spans = collect_import_dependencies(story);
    let direct_dependencies = dependency_spans
        .iter()
        .map(|(module, dependencies)| {
            (
                module.clone(),
                dependencies.keys().cloned().collect::<Vec<_>>(),
            )
        })
        .collect();

    ModuleDependencyGraph {
        direct_dependencies,
        dependency_spans,
    }
}

pub fn build_module_reachability(
    story: &Story,
    graph: &ModuleDependencyGraph,
    entry_point: Option<&ModuleEntryPoint>,
) -> ModuleReachability {
    let mut reachable_modules = BTreeSet::new();
    let entry_module = entry_point.map(|entry_point| {
        collect_reachable_modules(&entry_point.module, graph, &mut reachable_modules);
        entry_point.module.clone()
    });

    for module in internal_module_roots(story) {
        collect_reachable_modules(&module, graph, &mut reachable_modules);
    }

    ModuleReachability {
        entry_module,
        reachable_modules,
    }
}

pub(in crate::analysis) fn module_dependency_diagnostics(
    graph: &ModuleDependencyGraph,
) -> Vec<Diagnostic> {
    module_dependency_cycle_diagnostics(graph)
}

pub(in crate::analysis) fn unreachable_module_diagnostics(
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

    fn dependency_span(&self, module: &str, dependency: &str) -> Option<&SourceSpan> {
        self.dependency_spans
            .get(module)
            .and_then(|dependencies| dependencies.get(dependency))
    }
}

impl PartialEq for ModuleDependencyGraph {
    fn eq(&self, other: &Self) -> bool {
        self.direct_dependencies == other.direct_dependencies
    }
}

impl Eq for ModuleDependencyGraph {}

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

fn collect_import_dependencies(story: &Story) -> BTreeMap<String, BTreeMap<String, SourceSpan>> {
    let mut dependencies_by_module = BTreeMap::new();

    for module in story.modules() {
        let mut dependencies = BTreeMap::<String, SourceSpan>::new();
        for import in module.imports() {
            dependencies
                .entry(import.source_module().to_string())
                .or_insert_with(|| import.source_module_span().clone());
        }
        dependencies_by_module.insert(module.name().to_string(), dependencies);
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

fn internal_module_roots(story: &Story) -> BTreeSet<String> {
    story
        .modules()
        .iter()
        .filter(|module| module.flows().iter().any(flow_contains_internal))
        .map(|module| module.name().to_string())
        .collect()
}

fn flow_contains_internal(flow: &Flow) -> bool {
    flow.is_internal() || flow.child_flows().iter().any(flow_contains_internal)
}

fn module_dependency_cycle_diagnostics(graph: &ModuleDependencyGraph) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut reported_cycles = BTreeSet::new();

    for module in graph.modules() {
        let mut path = Vec::new();
        collect_cycles_from_module(
            module,
            graph,
            &mut path,
            &mut reported_cycles,
            &mut diagnostics,
        );
    }

    sort_diagnostics(&mut diagnostics);
    diagnostics
}

fn collect_cycles_from_module(
    current: &str,
    graph: &ModuleDependencyGraph,
    path: &mut Vec<String>,
    reported_cycles: &mut BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    path.push(current.to_string());

    if let Some(dependencies) = graph.direct_dependencies.get(current) {
        for dependency in dependencies {
            if !graph.direct_dependencies.contains_key(dependency) {
                continue;
            }

            if let Some(cycle_start) = path.iter().position(|module| module == dependency) {
                let mut cycle = path[cycle_start..].to_vec();
                cycle.push(dependency.clone());
                let key = canonical_cycle_key(&cycle);
                if reported_cycles.insert(key) {
                    let span = graph
                        .dependency_span(current, dependency)
                        .expect("dependency graph must retain import spans")
                        .clone();
                    diagnostics.push(Diagnostic::error(
                        span,
                        format!(
                            "Cyclic module import detected: {}",
                            format_module_path(&cycle)
                        ),
                    ));
                }
                continue;
            }

            collect_cycles_from_module(dependency, graph, path, reported_cycles, diagnostics);
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
