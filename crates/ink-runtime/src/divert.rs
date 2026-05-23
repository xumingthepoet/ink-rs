use std::{cell::RefCell, fmt, rc::Rc};

use crate::{
    container::Container,
    object::{Object, RTObject},
    path::{Component, Path},
    pointer::{self, Pointer},
    push_pop::PushPopType,
    story_error::StoryError,
};

pub struct Divert {
    obj: Object,
    target_pointer: RefCell<Pointer>,
    target_path: RefCell<Option<Path>>,
    pub external_args: usize,
    pub is_conditional: bool,
    pub is_external: bool,
    pub pushes_to_stack: bool,
    pub stack_push_type: PushPopType,
    pub variable_divert_name: Option<String>,
}

impl Divert {
    pub fn new(
        pushes_to_stack: bool,
        stack_push_type: PushPopType,
        is_external: bool,
        external_args: usize,
        is_conditional: bool,
        var_divert_name: Option<String>,
        target_path: Option<&str>,
    ) -> Self {
        Divert {
            obj: Object::new(),
            is_conditional,
            pushes_to_stack,
            stack_push_type,
            is_external,
            external_args,
            target_pointer: RefCell::new(pointer::NULL.clone()),
            target_path: RefCell::new(Self::target_path_string(target_path)),
            variable_divert_name: var_divert_name,
        }
    }

    fn target_path_string(value: Option<&str>) -> Option<Path> {
        value.map(|value| Path::new_with_components_string(Some(value)))
    }

    pub fn get_target_path_string(self: &Rc<Self>) -> Option<String> {
        self.get_target_path()
            .as_ref()
            .map(|p| self.compact_path_string(p))
    }

    pub fn has_variable_target(&self) -> bool {
        self.variable_divert_name.is_some()
    }

    fn compact_path_string(&self, other_path: &Path) -> String {
        let global_path_str;
        let relative_path_str;

        if other_path.is_relative() {
            relative_path_str = other_path.get_components_string();
            global_path_str = Object::get_path(self)
                .path_by_appending_path(other_path)
                .get_components_string();
        } else {
            let relative_path = self.convert_path_to_relative(other_path);
            relative_path_str = relative_path.get_components_string();
            global_path_str = other_path.get_components_string();
        }

        if relative_path_str.len() < global_path_str.len() {
            relative_path_str.clone()
        } else {
            global_path_str.clone()
        }
    }

    pub fn get_target_pointer(self: &Rc<Self>) -> Pointer {
        self.try_get_target_pointer()
            .unwrap_or_else(|_| pointer::NULL.clone())
    }

    pub fn try_get_target_pointer(self: &Rc<Self>) -> Result<Pointer, StoryError> {
        if !self.target_pointer.borrow().is_null() {
            return Ok(self.target_pointer.borrow().clone());
        }

        let target_path = self.target_path.borrow().clone().ok_or_else(|| {
            StoryError::InvalidStoryState("Divert is missing its target path".to_owned())
        })?;

        let target_result = Object::try_resolve_path(self.clone(), &target_path)?;
        if target_result.approximate {
            return Err(StoryError::InvalidStoryState(format!(
                "Divert target path '{}' could not be resolved",
                target_path.get_components_string()
            )));
        }

        let target_obj = target_result.obj.clone();
        let pointer = match target_path.get_last_component() {
            Some(last_component) if last_component.is_index() => {
                let index = last_component.index.ok_or_else(|| {
                    StoryError::InvalidStoryState(
                        "Divert target path has an index component without an index".to_owned(),
                    )
                })?;
                let parent = target_obj.get_object().get_parent().ok_or_else(|| {
                    StoryError::InvalidStoryState(
                        "Indexed divert target has no parent container".to_owned(),
                    )
                })?;

                Pointer::new(Some(parent), index as i32)
            }
            _ => {
                let target_container =
                    target_obj.into_any().downcast::<Container>().map_err(|_| {
                        StoryError::InvalidStoryState(
                            "Named divert target did not resolve to a container".to_owned(),
                        )
                    })?;

                Pointer::start_of(target_container)
            }
        };

        self.target_pointer.replace(pointer.clone());

        Ok(pointer)
    }

    pub fn get_target_path(self: &Rc<Self>) -> Option<Path> {
        // Resolve any relative paths to global ones as we come across them
        let target_path = self.target_path.borrow().clone();

        match target_path {
            Some(target_path) => {
                if target_path.is_relative() {
                    if let Ok(target_pointer) = self.try_get_target_pointer() {
                        if let Some(target_obj) = target_pointer.resolve() {
                            self.target_path
                                .replace(Some(Object::get_path(target_obj.as_ref())));
                        }
                    }
                }
                self.target_path.borrow().clone()
            }
            None => None,
        }
    }

    fn convert_path_to_relative(&self, global_path: &Path) -> Path {
        let own_path = Object::get_path(self);
        let min_path_length = std::cmp::min(global_path.len(), own_path.len());
        let mut last_shared_path_comp_index: i32 = -1;

        for i in 0..min_path_length {
            let own_comp = own_path.get_component(i);
            let other_comp = global_path.get_component(i);

            if own_comp.eq(&other_comp) {
                last_shared_path_comp_index = i as i32;
            } else {
                break;
            }
        }

        if last_shared_path_comp_index == -1 {
            return global_path.clone();
        }

        let num_upwards_moves = (own_path.len() - 1) - last_shared_path_comp_index as usize;
        let mut new_path_comps = Vec::new();

        for _ in 0..num_upwards_moves {
            new_path_comps.push(Component::to_parent());
        }

        for down in (last_shared_path_comp_index as usize + 1)..global_path.len() {
            if let Some(component) = global_path.get_component(down) {
                new_path_comps.push(component.clone());
            }
        }

        Path::new(&new_path_comps, true)
    }
}

impl RTObject for Divert {
    fn get_object(&self) -> &Object {
        &self.obj
    }
}

impl fmt::Display for Divert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();

        if let Some(variable_diver_name) = &self.variable_divert_name {
            result.push_str(&format!("Divert(variable: {})", variable_diver_name));
        } else if self.target_path.borrow().is_none() {
            result.push_str("Divert(null)");
        } else {
            let target_str = self
                .target_path
                .borrow()
                .as_ref()
                .unwrap()
                .get_components_string();

            result.push_str("Divert");

            if self.is_conditional {
                result.push('?');
            }

            if self.pushes_to_stack {
                if self.stack_push_type == PushPopType::Function {
                    result.push_str(" function");
                } else {
                    result.push_str(" tunnel");
                }
            }

            // TODO result.push_str(&format!(" -> {} ({})", self.get_target_path_string().unwrap_or_default(), target_str));
            let target_path = match self.target_path.borrow().as_ref() {
                Some(t) => t.to_string(),
                None => "".to_owned(),
            };

            result.push_str(&format!(" -> {} ({})", target_path, target_str));
        }

        write!(f, "{result}")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::rc::Rc;

    use super::*;

    #[test]
    fn divert_without_target_returns_error_and_null_pointer() {
        let divert = Rc::new(Divert::new(
            false,
            PushPopType::Tunnel,
            false,
            0,
            false,
            None,
            None,
        ));

        match divert.try_get_target_pointer() {
            Err(StoryError::InvalidStoryState(message)) => {
                assert_eq!(message, "Divert is missing its target path")
            }
            _ => panic!("expected invalid story state"),
        }

        assert!(divert.get_target_pointer().is_null());
    }

    #[test]
    fn unresolved_divert_target_returns_error_and_null_pointer() {
        let divert = Rc::new(Divert::new(
            false,
            PushPopType::Tunnel,
            false,
            0,
            false,
            None,
            Some("missing"),
        ));
        let _root = Container::new(None, vec![divert.clone()], HashMap::new());

        match divert.try_get_target_pointer() {
            Err(StoryError::InvalidStoryState(message)) => assert_eq!(
                message,
                "Divert target path 'missing' could not be resolved"
            ),
            _ => panic!("expected invalid story state"),
        }

        assert!(divert.get_target_pointer().is_null());
    }
}
