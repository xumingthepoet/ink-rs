use std::collections::HashMap;

pub(super) fn child_path(parent: &str, child: &str) -> String {
    if parent.is_empty() {
        child.to_string()
    } else {
        format!("{parent}.{child}")
    }
}

#[allow(dead_code)]
pub(super) fn parent_path(path: &str) -> Option<&str> {
    path.rsplit_once('.')
        .map(|(parent, _)| parent)
        .filter(|parent| !parent.is_empty())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn child_path_omits_empty_parent() {
        assert_eq!(child_path("", "knot"), "knot");
        assert_eq!(child_path("knot", "stitch"), "knot.stitch");
    }

    #[test]
    fn parent_path_returns_non_empty_parent() {
        assert_eq!(parent_path("knot.stitch.label"), Some("knot.stitch"));
        assert_eq!(parent_path("knot"), None);
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
}
