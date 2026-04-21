use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use super::FlowLevel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugMetadata {
    pub source_name: Option<String>,
    pub start_line_number: usize,
    pub end_line_number: usize,
    pub start_character_number: usize,
    pub end_character_number: usize,
}

pub type ObjectRef = Rc<RefCell<Object>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ObjectKind {
    Generic,
    ContentList {
        dont_flatten: bool,
    },
    Text {
        text: String,
    },
    AuthorWarning {
        warning_message: String,
    },
    Tag {
        is_start: bool,
        in_choice: bool,
    },
    Flow {
        flow_level: FlowLevel,
        name: Option<String>,
        is_function: bool,
    },
}

#[derive(Debug, Clone)]
pub struct Object {
    kind: ObjectKind,
    debug_metadata: Option<DebugMetadata>,
    parent: Option<Weak<RefCell<Object>>>,
    content: Vec<ObjectRef>,
}

impl Object {
    pub fn new() -> Self {
        Self {
            kind: ObjectKind::Generic,
            debug_metadata: None,
            parent: None,
            content: Vec::new(),
        }
    }

    pub fn new_ref() -> ObjectRef {
        Rc::new(RefCell::new(Self::new()))
    }

    pub fn debug_metadata(&self) -> Option<DebugMetadata> {
        self.debug_metadata.clone().or_else(|| {
            self.parent()
                .and_then(|parent| parent.borrow().debug_metadata())
        })
    }

    pub fn set_debug_metadata(&mut self, debug_metadata: Option<DebugMetadata>) {
        self.debug_metadata = debug_metadata;
    }

    pub fn has_own_debug_metadata(&self) -> bool {
        self.debug_metadata.is_some()
    }

    pub(crate) fn kind(&self) -> &ObjectKind {
        &self.kind
    }

    pub(crate) fn kind_mut(&mut self) -> &mut ObjectKind {
        &mut self.kind
    }

    pub(crate) fn set_kind(&mut self, kind: ObjectKind) {
        self.kind = kind;
    }

    pub(crate) fn set_flow_kind(
        &mut self,
        flow_level: FlowLevel,
        name: Option<String>,
        is_function: bool,
    ) {
        self.kind = ObjectKind::Flow {
            flow_level,
            name,
            is_function,
        };
    }

    pub fn parent(&self) -> Option<ObjectRef> {
        self.parent.as_ref().and_then(Weak::upgrade)
    }

    pub fn set_parent(&mut self, parent: Option<&ObjectRef>) {
        self.parent = parent.map(Rc::downgrade);
    }

    pub fn content(&self) -> &[ObjectRef] {
        &self.content
    }

    pub(crate) fn content_mut(&mut self) -> &mut Vec<ObjectRef> {
        &mut self.content
    }

    pub fn add_content(parent: &ObjectRef, sub_content: ObjectRef) -> ObjectRef {
        sub_content.borrow_mut().set_parent(Some(parent));
        parent.borrow_mut().content.push(sub_content.clone());
        sub_content
    }

    pub fn insert_content(parent: &ObjectRef, index: usize, sub_content: ObjectRef) -> ObjectRef {
        sub_content.borrow_mut().set_parent(Some(parent));
        parent
            .borrow_mut()
            .content
            .insert(index, sub_content.clone());
        sub_content
    }

    pub fn ancestry(&self) -> Vec<ObjectRef> {
        let mut result = Vec::new();
        let mut ancestor = self.parent();

        while let Some(parent) = ancestor {
            result.push(parent.clone());
            ancestor = parent.borrow().parent();
        }

        result.reverse();
        result
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::new()
    }
}

pub fn find_first<F>(object: &ObjectRef, query_func: F) -> Option<ObjectRef>
where
    F: Fn(&Object) -> bool + Copy,
{
    let object_borrow = object.borrow();
    if query_func(&object_borrow) {
        return Some(object.clone());
    }

    let children = object_borrow.content.clone();
    drop(object_borrow);

    for child in children {
        if let Some(found) = find_first(&child, query_func) {
            return Some(found);
        }
    }

    None
}

pub fn find_all<F>(object: &ObjectRef, query_func: F) -> Vec<ObjectRef>
where
    F: Fn(&Object) -> bool + Copy,
{
    let mut found = Vec::new();
    find_all_inner(object, query_func, &mut found);
    found
}

fn find_all_inner<F>(object: &ObjectRef, query_func: F, found_so_far: &mut Vec<ObjectRef>)
where
    F: Fn(&Object) -> bool + Copy,
{
    let object_borrow = object.borrow();
    if query_func(&object_borrow) {
        found_so_far.push(object.clone());
    }

    let children = object_borrow.content.clone();
    drop(object_borrow);

    for child in children {
        find_all_inner(&child, query_func, found_so_far);
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::{find_all, find_first, DebugMetadata, Object};

    fn sample_metadata(source_name: &str, line: usize) -> DebugMetadata {
        DebugMetadata {
            source_name: Some(source_name.to_string()),
            start_line_number: line,
            end_line_number: line,
            start_character_number: 1,
            end_character_number: 1,
        }
    }

    #[test]
    fn parsed_object_parent_links_and_debug_metadata_propagate() {
        let root = Object::new_ref();
        let child = Object::new_ref();

        root.borrow_mut()
            .set_debug_metadata(Some(sample_metadata("story.ink", 1)));
        Object::add_content(&root, child.clone());

        let child_parent = child.borrow().parent().expect("child should have a parent");
        assert!(Rc::ptr_eq(&root, &child_parent));
        assert_eq!(
            child.borrow().debug_metadata(),
            Some(sample_metadata("story.ink", 1))
        );
        assert!(child.borrow().has_own_debug_metadata() == false);
    }

    #[test]
    fn parsed_object_ancestry_and_traversal_are_depth_first() {
        let root = Object::new_ref();
        let left = Object::new_ref();
        let right = Object::new_ref();
        let leaf = Object::new_ref();

        Object::add_content(&root, left.clone());
        Object::add_content(&root, right.clone());
        Object::add_content(&left, leaf.clone());

        let ancestry = leaf.borrow().ancestry();
        assert_eq!(ancestry.len(), 2);
        assert!(Rc::ptr_eq(&ancestry[0], &root));
        assert!(Rc::ptr_eq(&ancestry[1], &left));

        let first_leaf = find_first(&root, |object| object.content().is_empty())
            .expect("expected to find a leaf");
        assert!(Rc::ptr_eq(&first_leaf, &leaf));

        let all_nodes = find_all(&root, |_| true);
        assert_eq!(all_nodes.len(), 4);
        assert!(Rc::ptr_eq(&all_nodes[0], &root));
        assert!(Rc::ptr_eq(&all_nodes[1], &left));
        assert!(Rc::ptr_eq(&all_nodes[2], &leaf));
        assert!(Rc::ptr_eq(&all_nodes[3], &right));
    }
}
