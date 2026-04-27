use std::collections::HashSet;

use super::indexes::{ConstantValues, ExternalSignatures, StructDefinitions};
use super::path::{child_path, module_scoped_source_path_to_runtime_path, LabelIndex};

pub(super) struct LoweringContext<'a> {
    path_mode: ChoicePathMode,
    choice_labels: &'a LabelIndex,
    global_labels: &'a LabelIndex,
    global_variables: &'a HashSet<String>,
    external_signatures: &'a ExternalSignatures,
    constants: &'a ConstantValues,
    struct_definitions: &'a StructDefinitions,
}

impl<'a> LoweringContext<'a> {
    pub(super) fn new(
        path_mode: ChoicePathMode,
        choice_labels: &'a LabelIndex,
        global_labels: &'a LabelIndex,
        global_variables: &'a HashSet<String>,
        external_signatures: &'a ExternalSignatures,
        constants: &'a ConstantValues,
        struct_definitions: &'a StructDefinitions,
    ) -> Self {
        Self {
            path_mode,
            choice_labels,
            global_labels,
            global_variables,
            external_signatures,
            constants,
            struct_definitions,
        }
    }

    pub(super) fn path_mode(&self) -> &ChoicePathMode {
        &self.path_mode
    }

    pub(super) fn choice_labels(&self) -> &LabelIndex {
        self.choice_labels
    }

    pub(super) fn global_labels(&self) -> &LabelIndex {
        self.global_labels
    }

    pub(super) fn global_variables(&self) -> &HashSet<String> {
        self.global_variables
    }

    pub(super) fn external_signatures(&self) -> &ExternalSignatures {
        self.external_signatures
    }

    pub(super) fn constants(&self) -> &ConstantValues {
        self.constants
    }

    pub(super) fn struct_definitions(&self) -> &StructDefinitions {
        self.struct_definitions
    }

    pub(super) fn with_path_mode(&self, path_mode: ChoicePathMode) -> Self {
        Self {
            path_mode,
            choice_labels: self.choice_labels,
            global_labels: self.global_labels,
            global_variables: self.global_variables,
            external_signatures: self.external_signatures,
            constants: self.constants,
            struct_definitions: self.struct_definitions,
        }
    }

    pub(super) fn scoped<'b>(
        &'b self,
        path_mode: ChoicePathMode,
        choice_labels: &'b LabelIndex,
    ) -> LoweringContext<'b> {
        LoweringContext {
            path_mode,
            choice_labels,
            global_labels: self.global_labels,
            global_variables: self.global_variables,
            external_signatures: self.external_signatures,
            constants: self.constants,
            struct_definitions: self.struct_definitions,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ChoicePathMode {
    Root,
    Module {
        module_name: String,
    },
    RootGather {
        gather_name: String,
    },
    NestedRoot {
        container_path: String,
        gather_target: String,
        allow_ancestor_fallback: bool,
    },
    Flow {
        module_name: Option<String>,
        flow_name: String,
        container_path: String,
        parent_flow_name: Option<String>,
        sibling_stitch_names: Vec<String>,
        local_variables: HashSet<String>,
        self_target_relative: bool,
        fallback_gather_target: Option<String>,
    },
}

impl ChoicePathMode {
    pub(super) fn current_module_name(&self) -> Option<&str> {
        match self {
            ChoicePathMode::Module { module_name } => Some(module_name.as_str()),
            ChoicePathMode::Flow { module_name, .. } => module_name.as_deref(),
            _ => None,
        }
    }

    pub(super) fn for_choice_nested_content(
        &self,
        choice_container_name: &str,
        gather_container_name: &str,
        has_following_gather: bool,
    ) -> Self {
        match self {
            ChoicePathMode::Module { .. } => ChoicePathMode::Root.for_choice_nested_content(
                choice_container_name,
                gather_container_name,
                has_following_gather,
            ),
            ChoicePathMode::Root => ChoicePathMode::NestedRoot {
                container_path: format!("0.{choice_container_name}"),
                gather_target: format!("0.{gather_container_name}"),
                allow_ancestor_fallback: true,
            },
            ChoicePathMode::RootGather { gather_name } => ChoicePathMode::NestedRoot {
                container_path: format!("0.{gather_name}.{choice_container_name}"),
                gather_target: format!("0.{gather_container_name}"),
                allow_ancestor_fallback: true,
            },
            ChoicePathMode::NestedRoot {
                container_path,
                gather_target,
                allow_ancestor_fallback,
            } => ChoicePathMode::NestedRoot {
                container_path: format!("{container_path}.{choice_container_name}"),
                gather_target: if has_following_gather {
                    format!("{container_path}.{gather_container_name}")
                } else {
                    gather_target.clone()
                },
                allow_ancestor_fallback: *allow_ancestor_fallback,
            },
            ChoicePathMode::Flow {
                module_name,
                flow_name,
                container_path,
                parent_flow_name,
                sibling_stitch_names,
                local_variables,
                ..
            } => ChoicePathMode::Flow {
                module_name: module_name.clone(),
                flow_name: flow_name.clone(),
                container_path: format!("{container_path}.{choice_container_name}"),
                parent_flow_name: parent_flow_name.clone(),
                sibling_stitch_names: sibling_stitch_names.clone(),
                local_variables: local_variables.clone(),
                self_target_relative: true,
                fallback_gather_target: if has_following_gather {
                    Some(child_path(container_path, gather_container_name))
                } else {
                    self.fallback_gather_target()
                },
            },
        }
    }

    pub(super) fn for_nested_weave(&self, container_index: usize) -> Self {
        match self {
            ChoicePathMode::Module { .. } => ChoicePathMode::Root.for_nested_weave(container_index),
            ChoicePathMode::NestedRoot {
                container_path,
                gather_target,
                allow_ancestor_fallback,
            } => ChoicePathMode::NestedRoot {
                container_path: format!("{container_path}.{container_index}"),
                gather_target: gather_target.clone(),
                allow_ancestor_fallback: *allow_ancestor_fallback,
            },
            ChoicePathMode::Root => ChoicePathMode::NestedRoot {
                container_path: format!("0.{container_index}"),
                gather_target: "0.g-0".to_string(),
                allow_ancestor_fallback: true,
            },
            ChoicePathMode::RootGather { gather_name } => ChoicePathMode::NestedRoot {
                container_path: format!("0.{gather_name}.{container_index}"),
                gather_target: "0.g-0".to_string(),
                allow_ancestor_fallback: true,
            },
            ChoicePathMode::Flow {
                module_name,
                flow_name,
                container_path,
                parent_flow_name,
                sibling_stitch_names,
                local_variables,
                self_target_relative,
                fallback_gather_target,
            } => ChoicePathMode::Flow {
                module_name: module_name.clone(),
                flow_name: flow_name.clone(),
                container_path: format!("{container_path}.{container_index}"),
                parent_flow_name: parent_flow_name.clone(),
                sibling_stitch_names: sibling_stitch_names.clone(),
                local_variables: local_variables.clone(),
                self_target_relative: *self_target_relative,
                fallback_gather_target: fallback_gather_target.clone(),
            },
        }
    }

    pub(super) fn for_gather(&self, gather_name: &str) -> Self {
        match self {
            ChoicePathMode::Module { .. } => ChoicePathMode::RootGather {
                gather_name: gather_name.to_string(),
            },
            ChoicePathMode::Root => ChoicePathMode::RootGather {
                gather_name: gather_name.to_string(),
            },
            ChoicePathMode::NestedRoot {
                container_path,
                gather_target,
                allow_ancestor_fallback,
            } => ChoicePathMode::NestedRoot {
                container_path: format!("{container_path}.{gather_name}"),
                gather_target: gather_target.clone(),
                allow_ancestor_fallback: *allow_ancestor_fallback,
            },
            ChoicePathMode::Flow {
                module_name,
                flow_name,
                container_path,
                parent_flow_name,
                sibling_stitch_names,
                local_variables,
                self_target_relative,
                fallback_gather_target,
            } => ChoicePathMode::Flow {
                module_name: module_name.clone(),
                flow_name: flow_name.clone(),
                container_path: format!("{container_path}.{gather_name}"),
                parent_flow_name: parent_flow_name.clone(),
                sibling_stitch_names: sibling_stitch_names.clone(),
                local_variables: local_variables.clone(),
                self_target_relative: *self_target_relative,
                fallback_gather_target: fallback_gather_target.clone(),
            },
            other => other.clone(),
        }
    }

    pub(super) fn for_conditional_branch(&self, branch_index: usize) -> Self {
        let container_path = format!("{}.b", self.runtime_index_path(branch_index));
        match self {
            ChoicePathMode::Root
            | ChoicePathMode::Module { .. }
            | ChoicePathMode::RootGather { .. } => ChoicePathMode::NestedRoot {
                container_path,
                gather_target: "0.g-0".to_string(),
                allow_ancestor_fallback: false,
            },
            ChoicePathMode::NestedRoot { gather_target, .. } => ChoicePathMode::NestedRoot {
                container_path,
                gather_target: gather_target.clone(),
                allow_ancestor_fallback: false,
            },
            ChoicePathMode::Flow {
                module_name,
                flow_name,
                parent_flow_name,
                sibling_stitch_names,
                local_variables,
                ..
            } => ChoicePathMode::Flow {
                module_name: module_name.clone(),
                flow_name: flow_name.clone(),
                container_path,
                parent_flow_name: parent_flow_name.clone(),
                sibling_stitch_names: sibling_stitch_names.clone(),
                local_variables: local_variables.clone(),
                self_target_relative: true,
                fallback_gather_target: None,
            },
        }
    }

    pub(super) fn for_sequence_branch(
        &self,
        sequence_container_path: &str,
        branch_name: &str,
    ) -> Self {
        let container_path = format!("{sequence_container_path}.{branch_name}");
        match self {
            ChoicePathMode::Root
            | ChoicePathMode::Module { .. }
            | ChoicePathMode::RootGather { .. } => ChoicePathMode::NestedRoot {
                container_path,
                gather_target: "0.g-0".to_string(),
                allow_ancestor_fallback: false,
            },
            ChoicePathMode::NestedRoot { gather_target, .. } => ChoicePathMode::NestedRoot {
                container_path,
                gather_target: gather_target.clone(),
                allow_ancestor_fallback: false,
            },
            ChoicePathMode::Flow {
                module_name,
                flow_name,
                parent_flow_name,
                sibling_stitch_names,
                local_variables,
                self_target_relative,
                ..
            } => ChoicePathMode::Flow {
                module_name: module_name.clone(),
                flow_name: flow_name.clone(),
                container_path,
                parent_flow_name: parent_flow_name.clone(),
                sibling_stitch_names: sibling_stitch_names.clone(),
                local_variables: local_variables.clone(),
                self_target_relative: *self_target_relative,
                fallback_gather_target: None,
            },
        }
    }

    pub(super) fn fallback_gather_target(&self) -> Option<String> {
        match self {
            ChoicePathMode::NestedRoot {
                gather_target,
                allow_ancestor_fallback,
                ..
            } => allow_ancestor_fallback.then(|| gather_target.clone()),
            ChoicePathMode::Flow {
                fallback_gather_target,
                ..
            } => fallback_gather_target.clone(),
            _ => None,
        }
    }

    pub(super) fn set_flow_fallback_gather_target(&mut self, target: String) {
        if let ChoicePathMode::Flow {
            fallback_gather_target,
            ..
        } = self
        {
            *fallback_gather_target = Some(target);
        }
    }

    pub(super) fn should_include_choice_gather(&self) -> bool {
        matches!(
            self,
            ChoicePathMode::Root
                | ChoicePathMode::Module { .. }
                | ChoicePathMode::RootGather { .. }
        ) || self.fallback_gather_target().is_some()
    }

    pub(super) fn is_root(&self) -> bool {
        matches!(self, ChoicePathMode::Root | ChoicePathMode::Module { .. })
    }

    pub(super) fn is_nested_root(&self) -> bool {
        matches!(self, ChoicePathMode::NestedRoot { .. })
    }

    pub(super) fn is_local_variable(&self, name: &str) -> bool {
        match self {
            ChoicePathMode::Flow {
                local_variables, ..
            } => local_variables.contains(name),
            _ => false,
        }
    }

    pub(super) fn current_flow_name(&self) -> Option<&str> {
        match self {
            ChoicePathMode::Flow { flow_name, .. } => Some(flow_name),
            _ => None,
        }
    }

    pub(super) fn current_flow_body_start_target(&self, argument_count: usize) -> Option<String> {
        self.current_flow_path()
            .map(|flow_path| format!("{flow_path}.{argument_count}"))
    }

    pub(super) fn runtime_index_path(&self, index: usize) -> String {
        match self {
            ChoicePathMode::Root => format!("0.{index}"),
            ChoicePathMode::Module { .. } => format!("0.{index}"),
            ChoicePathMode::RootGather { gather_name } => format!("0.{gather_name}.{index}"),
            ChoicePathMode::NestedRoot { container_path, .. }
            | ChoicePathMode::Flow { container_path, .. } => format!("{container_path}.{index}"),
        }
    }

    pub(super) fn sequence_container_path(&self, content_index: usize) -> String {
        self.runtime_index_path(content_index)
    }

    pub(super) fn absolute_child_path(&self, child: &str) -> String {
        child_path(&self.container_path(), child)
    }

    pub(super) fn container_path(&self) -> String {
        match self {
            ChoicePathMode::Root => "0".to_string(),
            ChoicePathMode::Module { .. } => "0".to_string(),
            ChoicePathMode::RootGather { gather_name } => format!("0.{gather_name}"),
            ChoicePathMode::NestedRoot { container_path, .. }
            | ChoicePathMode::Flow { container_path, .. } => container_path.clone(),
        }
    }

    pub(super) fn choice_point_target(&self, choice_index: usize) -> String {
        self.absolute_child_path(&format!("c-{choice_index}"))
    }

    pub(super) fn outer_return_target(&self, choice_point_index: usize) -> String {
        self.runtime_index_path(choice_point_index)
    }

    pub(super) fn gather_target(
        &self,
        gather_container_name: &str,
        has_following_gather: bool,
    ) -> String {
        match self {
            ChoicePathMode::Root => format!("0.{gather_container_name}"),
            ChoicePathMode::Module { .. } => format!("0.{gather_container_name}"),
            ChoicePathMode::RootGather { .. } => self.absolute_child_path(gather_container_name),
            ChoicePathMode::NestedRoot { .. } if has_following_gather => {
                self.absolute_child_path(gather_container_name)
            }
            ChoicePathMode::NestedRoot { gather_target, .. } => gather_target.clone(),
            ChoicePathMode::Flow {
                container_path,
                fallback_gather_target,
                ..
            } if has_following_gather || fallback_gather_target.is_none() => {
                child_path(container_path, gather_container_name)
            }
            ChoicePathMode::Flow {
                fallback_gather_target,
                ..
            } => fallback_gather_target
                .clone()
                .unwrap_or_else(|| gather_container_name.to_string()),
        }
    }

    /// Resolve a divert target path, converting absolute flow names to relative
    /// paths when the target is a sibling stitch or child stitch inside a
    /// choice container.
    pub(super) fn resolve_divert_target(&self, target: &str) -> String {
        match self {
            ChoicePathMode::Module { module_name } => {
                module_scoped_source_path_to_runtime_path(Some(module_name), target)
            }
            ChoicePathMode::Root
            | ChoicePathMode::RootGather { .. }
            | ChoicePathMode::NestedRoot { .. } => {
                module_scoped_source_path_to_runtime_path(None, target)
            }
            ChoicePathMode::Flow {
                module_name,
                parent_flow_name,
                flow_name,
                sibling_stitch_names,
                ..
            } => {
                if let Some((first_part, second_part)) = target.split_once('.') {
                    if parent_flow_name.is_none()
                        && first_part == flow_name
                        && sibling_stitch_names.iter().any(|name| name == second_part)
                    {
                        return module_scoped_source_path_to_runtime_path(
                            module_name.as_deref(),
                            &self.resolve_single_stitch_target(second_part),
                        );
                    }
                    return module_scoped_source_path_to_runtime_path(
                        module_name.as_deref(),
                        target,
                    );
                }

                module_scoped_source_path_to_runtime_path(
                    module_name.as_deref(),
                    &self.resolve_single_stitch_target(target),
                )
            }
        }
    }

    pub(super) fn resolve_label_target(&self, label_target: &str) -> String {
        let _ = self;
        label_target.to_string()
    }

    pub(super) fn scoped_label_target<'a>(
        &self,
        target: &str,
        global_labels: &'a LabelIndex,
    ) -> Option<&'a str> {
        global_labels.scoped_target(target, self.current_flow_path().as_deref())
    }

    fn current_flow_path(&self) -> Option<String> {
        match self {
            ChoicePathMode::Flow {
                module_name,
                flow_name,
                parent_flow_name,
                ..
            } => {
                let flow_path = parent_flow_name
                    .as_ref()
                    .map(|parent| format!("{parent}.{flow_name}"))
                    .unwrap_or_else(|| flow_name.clone());
                Some(module_scoped_source_path_to_runtime_path(
                    module_name.as_deref(),
                    &flow_path,
                ))
            }
            _ => None,
        }
    }

    /// Resolve a single stitch name to a relative path if applicable.
    pub(super) fn resolve_single_stitch_target(&self, target: &str) -> String {
        match self {
            ChoicePathMode::Root
            | ChoicePathMode::Module { .. }
            | ChoicePathMode::RootGather { .. }
            | ChoicePathMode::NestedRoot { .. } => target.to_string(),
            ChoicePathMode::Flow {
                sibling_stitch_names,
                parent_flow_name,
                flow_name,
                self_target_relative,
                ..
            } => {
                if parent_flow_name.is_some() {
                    let parent_flow_name = parent_flow_name.as_deref().unwrap();
                    if sibling_stitch_names.iter().any(|s| s == target) {
                        format!("{parent_flow_name}.{target}")
                    } else if target == parent_flow_name {
                        parent_flow_name.to_string()
                    } else {
                        target.to_string()
                    }
                } else if sibling_stitch_names.iter().any(|s| s == target) {
                    format!("{flow_name}.{target}")
                } else if *self_target_relative && target == *flow_name {
                    target.to_string()
                } else {
                    target.to_string()
                }
            }
        }
    }
}
