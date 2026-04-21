use super::Object;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Story {
    pub content: Vec<Object>,
    pub is_include: bool,
    pub count_all_visits: bool,
}

impl Story {
    pub fn new(content: Vec<Object>, is_include: bool) -> Self {
        Self {
            content,
            is_include,
            count_all_visits: false,
        }
    }
}
