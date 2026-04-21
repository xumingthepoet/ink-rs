use super::{Object, ObjectRef};

#[derive(Debug, Clone)]
pub struct Story {
    pub root: ObjectRef,
    pub is_include: bool,
    pub count_all_visits: bool,
}

impl Story {
    pub fn new(content: Vec<ObjectRef>, is_include: bool) -> Self {
        let root = Object::new_ref();
        for child in content {
            Object::add_content(&root, child);
        }

        Self {
            root,
            is_include,
            count_all_visits: false,
        }
    }

    pub fn content(&self) -> Vec<ObjectRef> {
        self.root.borrow().content().to_vec()
    }

    pub fn root(&self) -> ObjectRef {
        self.root.clone()
    }
}
