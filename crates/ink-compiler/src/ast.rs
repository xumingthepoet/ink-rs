use crate::source::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStory {
    pub nodes: Vec<AstNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstNode {
    TextLine {
        text: String,
        span: SourceSpan,
    },
    Choice {
        text: String,
        inline: bool,
        span: SourceSpan,
    },
    Divert {
        target: String,
        span: SourceSpan,
    },
}

impl ParsedStory {
    pub fn to_parse_snapshot(&self) -> String {
        let mut out = String::from("Story\n  Weave(baseIndent=0)");

        for node in &self.nodes {
            match node {
                AstNode::TextLine { text, .. } => {
                    out.push_str("\n    Text(\"");
                    out.push_str(&escape_snapshot_text(text));
                    out.push_str("\")\n    Text(\"\\n\")");
                }
                AstNode::Choice { text, inline, .. } => {
                    out.push_str(
                        "\n    Choice(name=null, once=true, invisible=false, depth=1, inline=",
                    );
                    out.push_str(if *inline { "true" } else { "false" });
                    out.push_str(")\n      ContentList\n        Text(\"");
                    out.push_str(&escape_snapshot_text(text));
                    out.push_str("\")\n      ContentList\n        Text(\"\\n\")");
                }
                AstNode::Divert { target, .. } => {
                    out.push_str("\n    Divert(target=\"-> ");
                    out.push_str(&escape_snapshot_text(target));
                    out.push_str("\", empty=false, tunnel=false, thread=false)");
                }
            }
        }

        out.push_str(
            "\n    Gather(name=null, depth=1)\n    Divert(target=\"-> DONE\", empty=false, tunnel=false, thread=false)",
        );
        out
    }
}

fn escape_snapshot_text(text: &str) -> String {
    let mut escaped = String::new();
    for ch in text.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
