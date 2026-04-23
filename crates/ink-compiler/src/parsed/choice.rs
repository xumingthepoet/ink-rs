use crate::source::SourceSpan;

use super::{push_indent, ContentList};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    start_content: Option<ContentList>,
    choice_only_content: Option<ContentList>,
    inner_content: ContentList,
    span: SourceSpan,
    identifier: Option<String>,
    once_only: bool,
    is_invisible_default: bool,
    indentation_depth: usize,
    has_weave_style_inline_brackets: bool,
}

impl Choice {
    pub fn new(
        start_content: Option<ContentList>,
        choice_only_content: Option<ContentList>,
        inner_content: ContentList,
        span: SourceSpan,
    ) -> Self {
        let has_weave_style_inline_brackets = choice_only_content.is_some();
        Self {
            start_content,
            choice_only_content,
            inner_content,
            span,
            identifier: None,
            once_only: true,
            is_invisible_default: false,
            indentation_depth: 1,
            has_weave_style_inline_brackets,
        }
    }

    pub fn start_content(&self) -> Option<&ContentList> {
        self.start_content.as_ref()
    }

    pub fn choice_only_content(&self) -> Option<&ContentList> {
        self.choice_only_content.as_ref()
    }

    pub fn inner_content(&self) -> &ContentList {
        &self.inner_content
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }

    pub fn once_only(&self) -> bool {
        self.once_only
    }

    pub fn has_start_content(&self) -> bool {
        self.start_content.is_some()
    }

    pub fn has_choice_only_content(&self) -> bool {
        self.choice_only_content.is_some()
    }

    pub fn is_invisible_default(&self) -> bool {
        self.is_invisible_default
    }

    pub fn indentation_depth(&self) -> usize {
        self.indentation_depth
    }

    pub fn has_weave_style_inline_brackets(&self) -> bool {
        self.has_weave_style_inline_brackets
    }

    pub fn choice_flags(&self) -> i32 {
        let mut flags = 0;
        if self.has_start_content() {
            flags |= 2;
        }
        if self.has_choice_only_content() {
            flags |= 4;
        }
        if self.is_invisible_default {
            flags |= 8;
        }
        if self.once_only {
            flags |= 16;
        }
        flags
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("Choice(name=");
        match &self.identifier {
            Some(name) => {
                out.push('"');
                out.push_str(name);
                out.push('"');
            }
            None => out.push_str("null"),
        }
        out.push_str(", once=");
        out.push_str(if self.once_only { "true" } else { "false" });
        out.push_str(", invisible=");
        out.push_str(if self.is_invisible_default {
            "true"
        } else {
            "false"
        });
        out.push_str(", depth=");
        out.push_str(&self.indentation_depth.to_string());
        out.push_str(", inline=");
        out.push_str(if self.has_weave_style_inline_brackets {
            "true"
        } else {
            "false"
        });
        out.push(')');

        if let Some(start_content) = &self.start_content {
            out.push('\n');
            push_indent(out, indent + 2);
            out.push_str("ContentList");
            start_content.write_parse_snapshot(out, indent + 4);
        }

        if let Some(choice_only_content) = &self.choice_only_content {
            out.push('\n');
            push_indent(out, indent + 2);
            out.push_str("ContentList");
            choice_only_content.write_parse_snapshot(out, indent + 4);
        }

        out.push('\n');
        push_indent(out, indent + 2);
        out.push_str("ContentList");
        self.inner_content.write_parse_snapshot(out, indent + 4);
    }
}
