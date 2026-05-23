use std::{
    any::Any,
    cell::RefCell,
    fmt::Display,
    rc::{Rc, Weak},
};

use as_any::{AsAny, Downcast};

use crate::{
    container::Container,
    path::{Component, Path},
    search_result::SearchResult,
    story_error::StoryError,
};

#[derive(Clone)]
pub struct Object {
    parent: RefCell<Weak<Container>>,
    path: RefCell<Option<Path>>,
    //debug_metadata: DebugMetadata,
}

impl Object {
    pub fn new() -> Object {
        Object {
            parent: RefCell::new(Weak::new()),
            path: RefCell::new(None),
        }
    }

    pub fn is_root(&self) -> bool {
        self.parent.borrow().upgrade().is_none()
    }

    pub fn get_parent(&self) -> Option<Rc<Container>> {
        self.parent.borrow().upgrade()
    }

    pub fn set_parent(&self, parent: &Rc<Container>) {
        self.parent.replace(Rc::downgrade(parent));
    }

    pub fn get_path(rtobject: &dyn RTObject) -> Path {
        if let Some(p) = rtobject.get_object().path.borrow().as_ref() {
            return p.clone();
        }

        match rtobject.get_object().get_parent() {
            Some(_) => {
                let mut comps: Vec<Component> = Vec::new();

                let mut container = rtobject.get_object().get_parent();
                let mut child = rtobject;
                let mut child_rc;

                while let Some(c) = container {
                    let mut child_valid_name = false;

                    if let Some(cc) = child.downcast_ref::<Container>() {
                        if cc.has_valid_name() {
                            child_valid_name = true;
                            comps.push(Component::new(cc.name.as_ref().unwrap()));
                        }
                    }

                    if !child_valid_name {
                        comps.push(Component::new_i(
                            c.content
                                .iter()
                                .position(|r| {
                                    let a = r.as_ref() as *const _ as *const ();
                                    let b = child as *const _ as *const ();
                                    std::ptr::eq(a, b)
                                })
                                .unwrap(),
                        ));
                    }

                    container = c.get_object().get_parent();
                    child_rc = Some(c);
                    child = child_rc.as_ref().unwrap().as_ref();
                }

                // Reverse list because components are searched in reverse order.
                comps.reverse();

                rtobject
                    .get_object()
                    .path
                    .replace(Some(Path::new(&comps, Path::default().is_relative())));
            }
            None => {
                rtobject
                    .get_object()
                    .path
                    .replace(Some(Path::new_with_defaults()));
            }
        }

        rtobject
            .get_object()
            .path
            .borrow()
            .as_ref()
            .unwrap()
            .clone()
    }

    pub fn resolve_path(rtobject: Rc<dyn RTObject>, path: &Path) -> SearchResult {
        match Object::try_resolve_path(rtobject.clone(), path) {
            Ok(result) => result,
            Err(_) => SearchResult::new(rtobject, true),
        }
    }

    pub fn try_resolve_path(
        rtobject: Rc<dyn RTObject>,
        path: &Path,
    ) -> Result<SearchResult, StoryError> {
        if path.is_relative() {
            let mut p = path.clone();
            let mut nearest_container = rtobject.clone().into_any().downcast::<Container>().ok();

            if nearest_container.is_none() {
                nearest_container = rtobject.get_object().get_parent();
                p = path.get_tail();
            };

            nearest_container
                .map(|container| container.content_at_path(&p, 0, -1))
                .ok_or_else(|| {
                    StoryError::InvalidStoryState(
                        "Relative path has no container to resolve from".to_owned(),
                    )
                })
        } else {
            Ok(Object::try_get_root_container(rtobject)?.content_at_path(path, 0, -1))
        }
    }

    pub fn convert_path_to_relative(rtobject: &Rc<dyn RTObject>, global_path: &Path) -> Path {
        // 1. Find last shared ancestor
        // 2. Drill up using ".." style (actually represented as "^")
        // 3. Re-build downward chain from common ancestor
        let own_path = rtobject
            .get_object()
            .path
            .borrow()
            .clone()
            .unwrap_or_else(|| Object::get_path(rtobject.as_ref()));
        let min_path_length = std::cmp::min(global_path.len(), own_path.len());
        let mut last_shared_path_comp_index: i32 = -1;

        for i in 0..min_path_length {
            let own_comp = &own_path.get_component(i);
            let other_comp = &global_path.get_component(i);

            if own_comp == other_comp {
                last_shared_path_comp_index = i as i32;
            } else {
                break;
            }
        }

        // No shared path components, so just use the global path
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

    pub fn compact_path_string(rtobject: Rc<dyn RTObject>, other_path: &Path) -> String {
        let global_path_str: String;
        let relative_path_str: String;

        if other_path.is_relative() {
            relative_path_str = other_path.get_components_string();
            global_path_str = Object::get_path(rtobject.as_ref())
                .path_by_appending_path(other_path)
                .get_components_string();
        } else {
            let relative_path = Object::convert_path_to_relative(&rtobject, other_path);
            relative_path_str = relative_path.get_components_string();
            global_path_str = other_path.get_components_string();
        }

        if relative_path_str.len() < global_path_str.len() {
            relative_path_str
        } else {
            global_path_str
        }
    }

    pub fn get_root_container(rtobject: Rc<dyn RTObject>) -> Rc<Container> {
        Object::try_get_root_container(rtobject).expect("root runtime object should be a container")
    }

    pub fn try_get_root_container(rtobject: Rc<dyn RTObject>) -> Result<Rc<Container>, StoryError> {
        let mut ancestor = rtobject;

        while let Some(p) = ancestor.get_object().get_parent() {
            ancestor = p;
        }

        match ancestor.into_any().downcast::<Container>() {
            Ok(c) => Ok(c.clone()),
            Err(_) => Err(StoryError::InvalidStoryState(
                "Root runtime object is not a container".to_owned(),
            )),
        }
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::new()
    }
}

pub trait IntoAny: AsAny {
    fn into_any(self: Rc<Self>) -> Rc<dyn Any>;
}

impl<T: Any> IntoAny for T {
    #[inline(always)]
    fn into_any(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }
}

pub trait RTObject: Display + IntoAny {
    fn get_object(&self) -> &Object;
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::value::Value;

    use super::*;

    #[test]
    fn get_path_test() {
        let container1 = Container::new(None, Vec::new(), HashMap::new());
        let container21 = Container::new(None, Vec::new(), HashMap::new());
        let container2 = Container::new(None, vec![container21.clone()], HashMap::new());
        let root = Container::new(
            None,
            vec![container1.clone(), container2.clone()],
            HashMap::new(),
        );

        let mut sb = String::new();

        root.build_string_of_hierarchy(&mut sb, 0, None);

        println!("root c:{:p}", &*root);
        println!(
            "container1 p:{:p} c:{:p}",
            &*(container1.get_object().get_parent().unwrap()),
            &*container1
        );
        println!(
            "container2 p:{:p} c:{:p}",
            &*(container2.get_object().get_parent().unwrap()),
            &*container2
        );
        println!(
            "container21 p:{:p} c:{:p}",
            &*(container21.get_object().get_parent().unwrap()),
            &*container21
        );

        println!("root: {}", sb);

        assert_eq!(Object::get_path(container1.as_ref()).to_string(), "0");
        assert_eq!(Object::get_path(container2.as_ref()).to_string(), "1");
        assert_eq!(Object::get_path(container21.as_ref()).to_string(), "1.0");
        assert_eq!(Object::get_path(root.as_ref()).to_string(), "");
    }

    #[test]
    fn try_get_root_container_reports_non_container_root() {
        let rootless: Rc<dyn RTObject> = Rc::new(Value::new("rootless"));

        match Object::try_get_root_container(rootless) {
            Err(StoryError::InvalidStoryState(message)) => {
                assert_eq!(message, "Root runtime object is not a container")
            }
            _ => panic!("expected invalid story state"),
        }
    }
}
