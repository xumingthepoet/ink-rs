use super::{Flow, InterfaceDeclaration, Module, Weave};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Story {
    root_weave: Weave,
    flows: Vec<Flow>,
    interfaces: Vec<InterfaceDeclaration>,
    modules: Vec<Module>,
}

impl Story {
    pub fn new(root_content: Vec<super::Object>, flows: Vec<Flow>) -> Self {
        Self::new_with_modules(root_content, flows, Vec::new())
    }

    pub fn new_with_modules(
        root_content: Vec<super::Object>,
        flows: Vec<Flow>,
        modules: Vec<Module>,
    ) -> Self {
        Self::new_with_modules_and_interfaces(root_content, flows, modules, Vec::new())
    }

    pub fn new_with_modules_and_interfaces(
        root_content: Vec<super::Object>,
        flows: Vec<Flow>,
        modules: Vec<Module>,
        interfaces: Vec<InterfaceDeclaration>,
    ) -> Self {
        Self {
            root_weave: Weave::new(root_content, 0),
            flows,
            interfaces,
            modules,
        }
    }

    pub fn root_weave(&self) -> &Weave {
        &self.root_weave
    }

    pub fn flows(&self) -> &[Flow] {
        &self.flows
    }

    pub fn interfaces(&self) -> &[InterfaceDeclaration] {
        &self.interfaces
    }

    pub fn modules(&self) -> &[Module] {
        &self.modules
    }

    pub fn to_parse_snapshot(&self) -> String {
        let mut out = String::from("Story");
        self.root_weave.write_parse_snapshot(&mut out, 2);
        for flow in &self.flows {
            flow.write_parse_snapshot(&mut out, 2);
        }
        for interface in &self.interfaces {
            interface.write_parse_snapshot(&mut out, 2);
        }
        for module in &self.modules {
            module.write_parse_snapshot(&mut out, 2);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parsed::{
            Flow, FlowLevel, FlowParts, ImportDeclaration, ImportedName, InterfaceDeclaration,
            Module, Object, Text,
        },
        source::SourceSpan,
    };

    use super::*;

    #[test]
    fn parse_snapshot_includes_modules() {
        let story = Story::new_with_modules(
            Vec::new(),
            Vec::new(),
            vec![Module::new(
                "game",
                vec![ImportDeclaration::new(
                    vec![ImportedName::new("sword", span_at(2, 8))],
                    "items",
                    span_at(2, 19),
                    span_at(2, 1),
                )],
                vec![Object::Text(Text::new("Line.", span_at(3, 1)))],
                vec![Flow::from_parts(FlowParts::new(
                    FlowLevel::Knot,
                    "main",
                    Vec::new(),
                ))],
                span_at(1, 12),
                span_at(1, 1),
            )],
        );

        assert_eq!(
            story.to_parse_snapshot(),
            "Story\n  Weave(baseIndent=0)\n  Module(name=\"game\")\n    Import(from=\"items\", names=[\"sword\"])\n    Weave(baseIndent=0)\n      Text(\"Line.\")\n    Flow(level=Knot, name=\"main\", function=false)"
        );
    }

    #[test]
    fn parse_snapshot_includes_interfaces() {
        let story = Story::new_with_modules_and_interfaces(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![InterfaceDeclaration::new(
                "IItem",
                span_at(1, 15),
                span_at(1, 1),
            )],
        );

        assert_eq!(
            story.to_parse_snapshot(),
            "Story\n  Weave(baseIndent=0)\n  Interface(name=\"IItem\")"
        );
    }

    fn span_at(line: usize, column: usize) -> SourceSpan {
        SourceSpan::new(Some("module.ink".to_string()), line, column)
    }
}
