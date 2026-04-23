use super::{push_indent, Flow, Weave};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Story {
    root_weave: Weave,
    flows: Vec<Flow>,
}

impl Story {
    pub fn new(root_content: Vec<super::Object>, flows: Vec<Flow>) -> Self {
        Self {
            root_weave: Weave::new(root_content, 0),
            flows,
        }
    }

    pub fn root_weave(&self) -> &Weave {
        &self.root_weave
    }

    pub fn flows(&self) -> &[Flow] {
        &self.flows
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
        out
    }
}
