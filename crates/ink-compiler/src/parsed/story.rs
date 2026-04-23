use super::{push_indent, Weave};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Story {
    root_weave: Weave,
}

impl Story {
    pub fn new(root_content: Vec<super::Object>) -> Self {
        Self {
            root_weave: Weave::new(root_content, 0),
        }
    }

    pub fn root_weave(&self) -> &Weave {
        &self.root_weave
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
        out
    }
}
