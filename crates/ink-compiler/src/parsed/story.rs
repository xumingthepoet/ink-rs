use super::{push_indent, Flow, Module, Weave};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Story {
    root_weave: Weave,
    flows: Vec<Flow>,
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
        Self {
            root_weave: Weave::new(root_content, 0),
            flows,
            modules,
        }
    }

    pub fn root_weave(&self) -> &Weave {
        &self.root_weave
    }

    pub fn flows(&self) -> &[Flow] {
        &self.flows
    }

    pub fn modules(&self) -> &[Module] {
        &self.modules
    }

    pub fn to_parse_snapshot(&self) -> String {
        let mut out = String::from("Story");
        self.root_weave.write_parse_snapshot(&mut out, 2);
        out.push('\n');
        push_indent(&mut out, 4);
        out.push_str("Gather(name=null, depth=1)");
        out.push('\n');
        push_indent(&mut out, 4);
        out.push_str("Divert(target=\"-> DONE\", empty=false, tunnel=false, thread=false)");
        for flow in &self.flows {
            flow.write_parse_snapshot(&mut out, 2);
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
            Flow, FlowLevel, ImportDeclaration, ImportedName, Module, Object, Text, TypeName,
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
                vec![Flow::new(
                    FlowLevel::Knot,
                    "main",
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    TypeName::void(),
                    false,
                )],
                span_at(1, 12),
                span_at(1, 1),
            )],
        );

        assert_eq!(
            story.to_parse_snapshot(),
            "Story\n  Weave(baseIndent=0)\n    Gather(name=null, depth=1)\n    Divert(target=\"-> DONE\", empty=false, tunnel=false, thread=false)\n  Module(name=\"game\")\n    Import(from=\"items\", names=[\"sword\"])\n    Weave(baseIndent=0)\n      Text(\"Line.\")\n    Flow(level=Knot, name=\"main\", function=false)"
        );
    }

    fn span_at(line: usize, column: usize) -> SourceSpan {
        SourceSpan::new(Some("module.ink".to_string()), line, column)
    }
}
