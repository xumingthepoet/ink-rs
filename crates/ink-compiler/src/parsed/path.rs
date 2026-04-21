use std::fmt;

use super::{FlowLevel, Identifier};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    base_target_level: Option<FlowLevel>,
    pub components: Vec<Identifier>,
}

impl Path {
    pub fn new(components: Vec<Identifier>) -> Self {
        Self {
            base_target_level: None,
            components,
        }
    }

    pub fn with_base_target_level(
        base_target_level: Option<FlowLevel>,
        components: Vec<Identifier>,
    ) -> Self {
        Self {
            base_target_level,
            components,
        }
    }

    pub fn from_identifier(identifier: Identifier) -> Self {
        Self::new(vec![identifier])
    }

    pub fn base_target_level(&self) -> FlowLevel {
        self.base_target_level.unwrap_or(FlowLevel::Story)
    }

    pub fn base_level_is_ambiguous(&self) -> bool {
        self.base_target_level.is_none()
    }

    pub fn components(&self) -> &[Identifier] {
        &self.components
    }

    pub fn from_components(
        base_target_level: Option<FlowLevel>,
        components: Vec<Identifier>,
    ) -> Self {
        Self {
            base_target_level,
            components,
        }
    }

    pub fn first_component(&self) -> Option<&str> {
        self.components
            .first()
            .map(|component| component.name.as_str())
    }

    pub fn number_of_components(&self) -> usize {
        self.components.len()
    }

    pub fn dot_separated_components(&self) -> Option<String> {
        if self.components.is_empty() {
            None
        } else {
            Some(
                self.components
                    .iter()
                    .map(|component| component.name.as_str())
                    .collect::<Vec<_>>()
                    .join("."),
            )
        }
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.dot_separated_components() {
            Some(components) => write!(f, "-> {components}"),
            None if self.base_target_level() == FlowLevel::WeavePoint => {
                f.write_str("-> <next gather point>")
            }
            None => f.write_str("<invalid Path>"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FlowLevel, Identifier, Path};

    #[test]
    fn parsed_path_joins_components() {
        let path = Path::new(vec![Identifier::new("root"), Identifier::new("child")]);

        assert_eq!(path.first_component(), Some("root"));
        assert_eq!(path.number_of_components(), 2);
        assert_eq!(
            path.dot_separated_components().as_deref(),
            Some("root.child")
        );
        assert_eq!(path.to_string(), "-> root.child");
    }

    #[test]
    fn parsed_path_handles_empty_components() {
        let path = Path::new(Vec::new());

        assert_eq!(path.first_component(), None);
        assert_eq!(path.dot_separated_components(), None);
        assert_eq!(path.to_string(), "<invalid Path>");
    }

    #[test]
    fn parsed_path_formats_next_gather_point() {
        let path = Path::with_base_target_level(Some(FlowLevel::WeavePoint), Vec::new());

        assert_eq!(path.base_target_level(), FlowLevel::WeavePoint);
        assert!(!path.base_level_is_ambiguous());
        assert_eq!(path.to_string(), "-> <next gather point>");
    }
}
