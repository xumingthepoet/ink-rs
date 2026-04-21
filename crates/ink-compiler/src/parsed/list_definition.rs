use std::fmt;

use super::{Identifier, Object, ObjectKind, ObjectRef};

#[derive(Debug, Clone)]
pub struct ListDefinition {
    object: ObjectRef,
}

#[derive(Debug, Clone)]
pub struct ListElementDefinition {
    object: ObjectRef,
}

impl ListDefinition {
    pub(crate) fn from_object(object: ObjectRef) -> Self {
        Self { object }
    }

    pub fn new(identifier: Identifier, item_definitions: Vec<ListElementDefinition>) -> Self {
        let object = Object::new_ref();
        object.borrow_mut().set_list_definition_kind(identifier);

        let mut current_value = 1_i64;
        for element in item_definitions {
            let explicit_value = element.explicit_value();
            let series_value = explicit_value.unwrap_or(current_value);
            let identifier = element.identifier().unwrap_or_else(|| Identifier::new(""));
            let in_initial_list = element.in_initial_list();
            element
                .object
                .borrow_mut()
                .set_list_element_definition_kind(
                    identifier,
                    explicit_value,
                    series_value,
                    in_initial_list,
                );
            Object::add_content(&object, element.object());
            current_value = series_value + 1;
        }

        Self { object }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn identifier(&self) -> Option<Identifier> {
        match self.object.borrow().kind() {
            ObjectKind::ListDefinition { identifier } => Some(identifier.clone()),
            _ => None,
        }
    }

    pub fn item_definitions(&self) -> Vec<ObjectRef> {
        self.object.borrow().content().to_vec()
    }

    pub fn item_named(&self, item_name: &str) -> Option<ObjectRef> {
        self.item_definitions().into_iter().find(|element| {
            matches!(
                element.borrow().kind(),
                ObjectKind::ListElementDefinition { identifier, .. }
                    if identifier.name == item_name
            )
        })
    }

    pub fn runtime_items(&self) -> Vec<(String, i64)> {
        self.item_definitions()
            .into_iter()
            .filter_map(|element| match element.borrow().kind() {
                ObjectKind::ListElementDefinition {
                    identifier,
                    series_value,
                    ..
                } => Some((identifier.name.clone(), *series_value)),
                _ => None,
            })
            .collect()
    }
}

impl fmt::Display for ListDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let identifier = self
            .identifier()
            .map(|identifier| identifier.name)
            .unwrap_or_else(|| "<invalid>".to_string());
        write!(f, "LIST {identifier}")
    }
}

impl ListElementDefinition {
    pub fn new(
        identifier: Identifier,
        in_initial_list: bool,
        explicit_value: Option<i64>,
        series_value: i64,
    ) -> Self {
        let object = Object::new_ref();
        object.borrow_mut().set_list_element_definition_kind(
            identifier,
            explicit_value,
            series_value,
            in_initial_list,
        );

        Self { object }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn identifier(&self) -> Option<Identifier> {
        match self.object.borrow().kind() {
            ObjectKind::ListElementDefinition { identifier, .. } => Some(identifier.clone()),
            _ => None,
        }
    }

    pub fn explicit_value(&self) -> Option<i64> {
        match self.object.borrow().kind() {
            ObjectKind::ListElementDefinition { explicit_value, .. } => *explicit_value,
            _ => None,
        }
    }

    pub fn series_value(&self) -> Option<i64> {
        match self.object.borrow().kind() {
            ObjectKind::ListElementDefinition { series_value, .. } => Some(*series_value),
            _ => None,
        }
    }

    pub fn in_initial_list(&self) -> bool {
        matches!(
            self.object.borrow().kind(),
            ObjectKind::ListElementDefinition {
                in_initial_list: true,
                ..
            }
        )
    }
}

impl fmt::Display for ListElementDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let identifier = self
            .identifier()
            .map(|identifier| identifier.name)
            .unwrap_or_else(|| "<invalid>".to_string());
        write!(f, "{identifier}")
    }
}

#[cfg(test)]
mod tests {
    use super::{ListDefinition, ListElementDefinition};
    use crate::parsed::{Identifier, ObjectKind};

    #[test]
    fn list_definition_wraps_items_and_series_values() {
        let list = ListDefinition::new(
            Identifier::new("terrain"),
            vec![
                ListElementDefinition::new(Identifier::new("forest"), false, None, 1),
                ListElementDefinition::new(Identifier::new("hill"), false, Some(4), 4),
                ListElementDefinition::new(Identifier::new("beach"), true, None, 5),
            ],
        );

        assert_eq!(
            list.identifier().map(|id| id.name),
            Some("terrain".to_string())
        );
        assert_eq!(list.item_definitions().len(), 3);
        assert_eq!(
            list.item_named("hill")
                .and_then(|item| match item.borrow().kind() {
                    ObjectKind::ListElementDefinition { series_value, .. } => Some(*series_value),
                    _ => None,
                }),
            Some(4)
        );
        assert_eq!(list.runtime_items().len(), 3);
    }

    #[test]
    fn list_element_definition_stores_flags() {
        let element = ListElementDefinition::new(Identifier::new("forest"), true, Some(7), 7);

        assert_eq!(
            element.identifier().map(|id| id.name),
            Some("forest".to_string())
        );
        assert_eq!(element.explicit_value(), Some(7));
        assert_eq!(element.series_value(), Some(7));
        assert!(element.in_initial_list());
    }
}
