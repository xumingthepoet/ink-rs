use std::{borrow::Borrow, collections::HashMap, fmt};

use ink_story_json_format::{Container, Object as RuntimeObject};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct RuntimePath(String);

impl RuntimePath {
    pub(super) fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RuntimePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct LabelAlias(String);

impl LabelAlias {
    fn new(alias: impl Into<String>) -> Self {
        Self(alias.into())
    }
}

impl Borrow<str> for LabelAlias {
    fn borrow(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct LabelIndex {
    labels: HashMap<LabelAlias, RuntimePath>,
}

impl LabelIndex {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn insert(&mut self, alias: impl Into<String>, target: impl Into<RuntimePath>) {
        self.labels.insert(LabelAlias::new(alias), target.into());
    }

    pub(super) fn insert_scoped_aliases(
        &mut self,
        identifier: &str,
        container_path: &str,
        flow_alias_prefix: Option<&str>,
        target: impl Into<RuntimePath>,
    ) {
        let target = target.into();
        self.insert(identifier.to_string(), target.clone());
        if let Some(flow_path) = flow_alias_prefix {
            self.insert(format!("{flow_path}.{identifier}"), target.clone());
        }
        if let Some(flow_path) = container_path.strip_suffix(".0") {
            self.insert(format!("{flow_path}.{identifier}"), target);
        }
    }

    pub(super) fn get(&self, alias: &str) -> Option<&str> {
        self.labels.get(alias).map(RuntimePath::as_str)
    }

    pub(super) fn scoped_target<'a>(
        &'a self,
        target: &str,
        current_flow_path: Option<&str>,
    ) -> Option<&'a str> {
        if target.contains('.') {
            return self.get(target);
        }

        current_flow_path
            .and_then(|flow_path| self.get(&format!("{flow_path}.{target}")))
            .or_else(|| self.get(target))
    }
}

impl From<String> for RuntimePath {
    fn from(path: String) -> Self {
        Self::new(path)
    }
}

impl From<&str> for RuntimePath {
    fn from(path: &str) -> Self {
        Self::new(path)
    }
}

pub(super) fn child_path(parent: &str, child: &str) -> String {
    if parent.is_empty() {
        child.to_string()
    } else {
        format!("{parent}.{child}")
    }
}

pub(super) fn source_path_to_runtime_path(source_path: &str) -> String {
    source_path.replace("::", ".")
}

pub(super) fn module_scoped_source_path_to_runtime_path(
    module_name: Option<&str>,
    source_path: &str,
) -> String {
    if source_path.contains("::") {
        return source_path_to_runtime_path(source_path);
    }

    match module_name {
        Some(module_name) if is_absolute_runtime_path(source_path) => {
            child_path(module_name, source_path)
        }
        _ => source_path.to_string(),
    }
}

pub(super) fn is_absolute_runtime_path(target: &str) -> bool {
    !target.is_empty()
        && !target.starts_with('.')
        && !target.starts_with('$')
        && target != "DONE"
        && target != "END"
}

pub(super) fn compact_path_string(object_path: &str, global_path: &str) -> String {
    let own_components = path_components(object_path);
    let global_components = path_components(global_path);
    let mut last_shared_index = None;

    for (index, (own, global)) in own_components
        .iter()
        .zip(global_components.iter())
        .enumerate()
    {
        if own == global {
            last_shared_index = Some(index);
        } else {
            break;
        }
    }

    let Some(last_shared_index) = last_shared_index else {
        return global_path.to_string();
    };

    let upward_moves = own_components
        .len()
        .saturating_sub(1)
        .saturating_sub(last_shared_index);
    let relative = relative_path_string(upward_moves, &global_components[last_shared_index + 1..]);

    compact_relative_path(&relative, global_path)
}

pub(super) fn path_components(path: &str) -> Vec<&str> {
    path.split('.')
        .filter(|component| !component.is_empty())
        .collect()
}

pub(super) fn relative_path_string(upward_moves: usize, downward_components: &[&str]) -> String {
    let mut components = vec!["^"; upward_moves];
    components.extend(downward_components.iter().copied());

    if components
        .first()
        .is_some_and(|component| *component == "^")
    {
        format!(".{}", components.join("."))
    } else {
        components.join(".")
    }
}

pub(super) fn compact_relative_path(relative: &str, global: &str) -> String {
    if relative.len() < global.len() {
        relative.to_string()
    } else {
        global.to_string()
    }
}

pub(super) fn canonical_runtime_path(
    target: &str,
    semantic_paths: &HashMap<String, Option<String>>,
) -> Option<String> {
    let last = path_components(target).pop()?;
    if !is_user_named_path_component(last) {
        return None;
    }
    let key = semantic_path_key(target)?;
    semantic_paths.get(&key).and_then(Clone::clone)
}

pub(super) fn semantic_path_key(path: &str) -> Option<String> {
    let components = path_components(path)
        .into_iter()
        .filter(|component| is_user_named_path_component(component))
        .collect::<Vec<_>>();
    (!components.is_empty()).then(|| components.join("."))
}

pub(super) fn is_user_named_path_component(component: &str) -> bool {
    !component.is_empty()
        && component.parse::<usize>().is_err()
        && !component.starts_with("c-")
        && !component.starts_with("g-")
        && !component.starts_with('$')
}

pub(super) fn compact_path_strings_in_container(container: &mut Container) {
    let semantic_paths = build_semantic_path_index(container);
    compact_path_strings_in_container_at(container, "", &semantic_paths);
}

fn compact_path_strings_in_container_at(
    container: &mut Container,
    container_path: &str,
    semantic_paths: &HashMap<String, Option<String>>,
) {
    for named in &mut container.named_content {
        let path = child_path(container_path, &named.name);
        compact_path_strings_in_container_at(&mut named.container, &path, semantic_paths);
    }

    for (content_index, object) in container.content.iter_mut().enumerate() {
        let object_path = match object {
            RuntimeObject::Container(container) => container
                .name
                .as_deref()
                .map(|name| child_path(container_path, name))
                .unwrap_or_else(|| child_path(container_path, &content_index.to_string())),
            _ => child_path(container_path, &content_index.to_string()),
        };
        compact_path_strings_in_object(object, &object_path, semantic_paths);
    }
}

fn compact_path_strings_in_object(
    object: &mut RuntimeObject,
    object_path: &str,
    semantic_paths: &HashMap<String, Option<String>>,
) {
    match object {
        RuntimeObject::Container(container) => {
            compact_path_strings_in_container_at(container, object_path, semantic_paths);
        }
        RuntimeObject::Divert { target, variable }
        | RuntimeObject::TunnelDivert { target, variable } => {
            if !*variable {
                compact_target_path_string(target, object_path, semantic_paths);
            }
        }
        RuntimeObject::ConditionalDivert { target } | RuntimeObject::ChoicePoint { target, .. } => {
            compact_target_path_string(target, object_path, semantic_paths);
        }
        _ => {}
    }
}

fn compact_target_path_string(
    target: &mut String,
    object_path: &str,
    semantic_paths: &HashMap<String, Option<String>>,
) {
    if !is_absolute_runtime_path(target) {
        return;
    }
    if let Some(canonical) = canonical_runtime_path(target, semantic_paths) {
        *target = canonical;
    }
    *target = compact_path_string(object_path, target);
}

fn build_semantic_path_index(container: &Container) -> HashMap<String, Option<String>> {
    let mut paths = HashMap::new();
    collect_semantic_paths(container, "", &mut paths);
    paths
}

fn collect_semantic_paths(
    container: &Container,
    container_path: &str,
    paths: &mut HashMap<String, Option<String>>,
) {
    for named in &container.named_content {
        let path = child_path(container_path, &named.name);
        collect_semantic_path_for_container(&named.container, &path, paths);
    }

    for (content_index, object) in container.content.iter().enumerate() {
        let object_path = match object {
            RuntimeObject::Container(container) => container
                .name
                .as_deref()
                .map(|name| child_path(container_path, name))
                .unwrap_or_else(|| child_path(container_path, &content_index.to_string())),
            _ => child_path(container_path, &content_index.to_string()),
        };

        if let RuntimeObject::Container(container) = object {
            collect_semantic_path_for_container(container, &object_path, paths);
        }
    }
}

fn collect_semantic_path_for_container(
    container: &Container,
    path: &str,
    paths: &mut HashMap<String, Option<String>>,
) {
    if container
        .name
        .as_deref()
        .is_some_and(is_user_named_path_component)
    {
        if let Some(key) = semantic_path_key(path) {
            paths
                .entry(key)
                .and_modify(|entry| {
                    if entry.as_deref() != Some(path) {
                        *entry = None;
                    }
                })
                .or_insert_with(|| Some(path.to_string()));
        }
    }
    collect_semantic_paths(container, path, paths);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn container(name: Option<&str>, content: Vec<RuntimeObject>) -> Container {
        Container {
            content,
            named_content: Vec::new(),
            name: name.map(str::to_string),
        }
    }

    fn named_container(container: Container) -> ink_story_json_format::NamedContainer {
        ink_story_json_format::NamedContainer::new(container.name.clone().unwrap(), container)
    }

    #[test]
    fn child_path_omits_empty_parent() {
        assert_eq!(child_path("", "knot"), "knot");
        assert_eq!(child_path("knot", "stitch"), "knot.stitch");
    }

    #[test]
    fn compact_path_string_prefers_shorter_relative_paths() {
        assert_eq!(compact_path_string("knot.0.c-0", "knot.0.g-0"), ".^.g-0");
        assert_eq!(
            compact_path_string("root", "much.longer.path"),
            "much.longer.path"
        );
    }

    #[test]
    fn canonical_runtime_path_uses_unique_semantic_target() {
        let semantic_paths =
            HashMap::from([("knot.label".to_string(), Some("knot.0.label".to_string()))]);

        assert_eq!(
            canonical_runtime_path("knot.label", &semantic_paths),
            Some("knot.0.label".to_string())
        );
    }

    #[test]
    fn label_index_returns_runtime_paths_by_alias() {
        let mut labels = LabelIndex::new();
        labels.insert("short_label", RuntimePath::new("knot.0.label"));

        assert_eq!(labels.get("short_label"), Some("knot.0.label"));
    }

    #[test]
    fn label_index_resolves_flow_scoped_aliases_before_global_aliases() {
        let mut labels = LabelIndex::new();
        labels.insert("label", RuntimePath::new("global.label"));
        labels.insert("knot.label", RuntimePath::new("knot.0.label"));

        assert_eq!(
            labels.scoped_target("label", Some("knot")),
            Some("knot.0.label")
        );
        assert_eq!(labels.scoped_target("label", None), Some("global.label"));
    }

    #[test]
    fn label_index_inserts_flow_and_container_scoped_aliases() {
        let mut labels = LabelIndex::new();
        labels.insert_scoped_aliases(
            "choice",
            "knot.0",
            Some("knot"),
            RuntimePath::new("knot.0.c-0"),
        );

        assert_eq!(labels.get("choice"), Some("knot.0.c-0"));
        assert_eq!(labels.get("knot.choice"), Some("knot.0.c-0"));
    }

    #[test]
    fn compact_path_strings_canonicalizes_and_relativizes_targets() {
        let mut root = container(
            None,
            vec![RuntimeObject::Container(container(
                Some("knot"),
                vec![RuntimeObject::Divert {
                    target: "knot.0.label".to_string(),
                    variable: false,
                }],
            ))],
        );
        if let RuntimeObject::Container(knot) = &mut root.content[0] {
            knot.named_content
                .push(named_container(container(Some("label"), Vec::new())));
        }

        compact_path_strings_in_container(&mut root);

        let RuntimeObject::Container(knot) = &root.content[0] else {
            panic!("expected knot container");
        };
        let RuntimeObject::Divert { target, .. } = &knot.content[0] else {
            panic!("expected divert");
        };
        assert_eq!(target, ".^.label");
    }
}
