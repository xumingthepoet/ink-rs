use super::{
    FlowArgument, FlowBase, FlowLevel, HasContent, Identifier, NamedContent, Object, ObjectRef,
};

#[derive(Debug, Clone)]
pub struct Knot {
    object: ObjectRef,
    identifier: Option<Identifier>,
    arguments: Vec<FlowArgument>,
    is_function: bool,
}

impl Knot {
    pub fn new(
        name: Identifier,
        top_level_objects: Vec<ObjectRef>,
        arguments: Vec<FlowArgument>,
        is_function: bool,
    ) -> Self {
        let object = Object::new_ref();
        object
            .borrow_mut()
            .set_flow_kind(FlowLevel::Knot, Some(name.name.clone()), is_function);

        for child in top_level_objects {
            Object::add_content(&object, child);
        }

        Self {
            object,
            identifier: Some(name),
            arguments,
            is_function,
        }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn name(&self) -> Option<&str> {
        self.identifier
            .as_ref()
            .map(|identifier| identifier.name.as_str())
    }

    pub fn identifier(&self) -> Option<&Identifier> {
        self.identifier.as_ref()
    }

    pub fn flow_level(&self) -> FlowLevel {
        FlowLevel::Knot
    }

    pub fn arguments(&self) -> &[FlowArgument] {
        &self.arguments
    }

    pub fn has_parameters(&self) -> bool {
        !self.arguments.is_empty()
    }

    pub fn is_function(&self) -> bool {
        self.is_function
    }

    pub fn content(&self) -> Vec<ObjectRef> {
        self.object.borrow().content().to_vec()
    }
}

impl NamedContent for Knot {
    fn name(&self) -> Option<&str> {
        Knot::name(self)
    }
}

impl FlowBase for Knot {
    fn identifier(&self) -> Option<&Identifier> {
        Knot::identifier(self)
    }

    fn flow_level(&self) -> FlowLevel {
        Knot::flow_level(self)
    }

    fn arguments(&self) -> Option<&[FlowArgument]> {
        Some(&self.arguments)
    }

    fn is_function(&self) -> bool {
        Knot::is_function(self)
    }
}

impl HasContent for Knot {
    fn content(&self) -> Vec<ObjectRef> {
        Knot::content(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::parsed::{FlowArgument, FlowLevel, Identifier, Text};

    use super::Knot;

    #[test]
    fn knot_exposes_flow_base_fields() {
        let knot = Knot::new(
            Identifier::new("start"),
            vec![Text::new("body").object()],
            vec![FlowArgument {
                identifier: Identifier::new("arg"),
                is_by_reference: false,
                is_divert_target: false,
            }],
            true,
        );

        assert_eq!(knot.name(), Some("start"));
        assert_eq!(knot.identifier().unwrap().name, "start");
        assert_eq!(knot.flow_level(), FlowLevel::Knot);
        assert!(knot.is_function());
        assert!(knot.has_parameters());
        assert_eq!(knot.arguments().len(), 1);
        assert_eq!(knot.content().len(), 1);
    }
}
