use super::{FlowLevel, Identifier, ObjectRef};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowArgument {
    pub identifier: Identifier,
    pub is_by_reference: bool,
    pub is_divert_target: bool,
}

pub trait NamedContent {
    fn name(&self) -> Option<&str>;
}

pub trait FlowBase: NamedContent {
    fn identifier(&self) -> Option<&Identifier>;

    fn flow_level(&self) -> FlowLevel;

    fn arguments(&self) -> Option<&[FlowArgument]> {
        None
    }

    fn is_function(&self) -> bool {
        false
    }

    fn has_parameters(&self) -> bool {
        self.arguments()
            .is_some_and(|arguments| !arguments.is_empty())
    }
}

pub trait HasContent {
    fn content(&self) -> Vec<ObjectRef>;
}

#[cfg(test)]
mod tests {
    use crate::parsed::Story;

    use super::{FlowArgument, FlowBase, FlowLevel, Identifier, NamedContent};

    struct FlowBaseStub {
        identifier: Option<Identifier>,
        arguments: Vec<FlowArgument>,
        flow_level: FlowLevel,
        is_function: bool,
    }

    impl NamedContent for FlowBaseStub {
        fn name(&self) -> Option<&str> {
            self.identifier
                .as_ref()
                .map(|identifier| identifier.name.as_str())
        }
    }

    impl FlowBase for FlowBaseStub {
        fn identifier(&self) -> Option<&Identifier> {
            self.identifier.as_ref()
        }

        fn flow_level(&self) -> FlowLevel {
            self.flow_level
        }

        fn arguments(&self) -> Option<&[FlowArgument]> {
            Some(&self.arguments)
        }

        fn is_function(&self) -> bool {
            self.is_function
        }
    }

    #[test]
    fn flow_base_trait_exposes_name_parameters_and_level() {
        let flow = FlowBaseStub {
            identifier: Some(Identifier::new("knot")),
            arguments: vec![FlowArgument {
                identifier: Identifier::new("arg"),
                is_by_reference: false,
                is_divert_target: false,
            }],
            flow_level: FlowLevel::Knot,
            is_function: true,
        };

        assert_eq!(flow.name(), Some("knot"));
        assert_eq!(flow.identifier().unwrap().name, "knot");
        assert_eq!(flow.flow_level(), FlowLevel::Knot);
        assert!(flow.has_parameters());
        assert!(flow.is_function());
    }

    #[test]
    fn flow_level_orders_story_before_nested_flows() {
        assert!(FlowLevel::Story < FlowLevel::Knot);
        assert!(FlowLevel::Knot < FlowLevel::Stitch);
        assert!(FlowLevel::Stitch < FlowLevel::WeavePoint);
    }

    #[test]
    fn story_implements_flow_base_traits() {
        let story = Story::new(Vec::new(), false);

        assert_eq!(story.name(), None);
        assert_eq!(story.identifier(), None);
        assert_eq!(story.flow_level(), FlowLevel::Story);
        assert!(!story.has_parameters());
        assert!(!story.is_function());
    }
}
