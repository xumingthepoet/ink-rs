use super::{
    FlowArgument, FlowBase, FlowLevel, HasContent, Identifier, NamedContent, Object, ObjectRef,
};

#[derive(Debug, Clone)]
pub struct Stitch {
    object: ObjectRef,
    identifier: Option<Identifier>,
    arguments: Vec<FlowArgument>,
    is_function: bool,
}

impl Stitch {
    pub fn new(
        name: Identifier,
        top_level_objects: Vec<ObjectRef>,
        arguments: Vec<FlowArgument>,
        is_function: bool,
    ) -> Self {
        let object = Object::new_ref();
        object
            .borrow_mut()
            .set_flow_kind(FlowLevel::Stitch, Some(name.name.clone()), is_function);

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
        FlowLevel::Stitch
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

impl NamedContent for Stitch {
    fn name(&self) -> Option<&str> {
        Stitch::name(self)
    }
}

impl FlowBase for Stitch {
    fn identifier(&self) -> Option<&Identifier> {
        Stitch::identifier(self)
    }

    fn flow_level(&self) -> FlowLevel {
        Stitch::flow_level(self)
    }

    fn arguments(&self) -> Option<&[FlowArgument]> {
        Some(&self.arguments)
    }

    fn is_function(&self) -> bool {
        Stitch::is_function(self)
    }
}

impl HasContent for Stitch {
    fn content(&self) -> Vec<ObjectRef> {
        Stitch::content(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::parsed::{FlowArgument, FlowLevel, Identifier, Text};

    use super::Stitch;

    #[test]
    fn stitch_exposes_flow_base_fields() {
        let stitch = Stitch::new(
            Identifier::new("body"),
            vec![Text::new("body").object()],
            vec![FlowArgument {
                identifier: Identifier::new("arg"),
                is_by_reference: false,
                is_divert_target: false,
            }],
            false,
        );

        assert_eq!(stitch.name(), Some("body"));
        assert_eq!(stitch.identifier().unwrap().name, "body");
        assert_eq!(stitch.flow_level(), FlowLevel::Stitch);
        assert!(!stitch.is_function());
        assert!(stitch.has_parameters());
        assert_eq!(stitch.arguments().len(), 1);
        assert_eq!(stitch.content().len(), 1);
    }
}
