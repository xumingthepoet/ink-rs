use crate::source::SourceSpan;

use super::{push_indent, ContentList, Expression};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    start_content: Option<ContentList>,
    inner_content: ContentList,
    span: SourceSpan,
    identifier: Option<String>,
    is_invisible_default: bool,
    indentation_depth: usize,
    condition: Option<Expression>,
}

impl Choice {
    pub fn new(
        start_content: Option<ContentList>,
        inner_content: ContentList,
        span: SourceSpan,
    ) -> Self {
        Self {
            start_content,
            inner_content,
            span,
            identifier: None,
            is_invisible_default: false,
            indentation_depth: 1,
            condition: None,
        }
    }

    pub fn start_content(&self) -> Option<&ContentList> {
        self.start_content.as_ref()
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

    pub fn set_identifier(&mut self, identifier: Option<String>) {
        self.identifier = identifier;
    }

    pub fn has_start_content(&self) -> bool {
        self.start_content.is_some()
    }

    pub fn is_invisible_default(&self) -> bool {
        self.is_invisible_default
    }

    pub fn set_is_invisible_default(&mut self, is_invisible_default: bool) {
        self.is_invisible_default = is_invisible_default;
    }

    pub fn indentation_depth(&self) -> usize {
        self.indentation_depth
    }

    pub fn set_indentation_depth(&mut self, indentation_depth: usize) {
        self.indentation_depth = indentation_depth;
    }

    pub fn condition(&self) -> Option<&Expression> {
        self.condition.as_ref()
    }

    pub fn set_condition(&mut self, condition: Option<Expression>) {
        self.condition = condition;
    }

    pub fn choice_flags(&self) -> i32 {
        let mut flags = 0;
        if self.condition.is_some() {
            flags |= 1;
        }
        if self.has_start_content() {
            flags |= 2;
        }
        if self.is_invisible_default {
            flags |= 8;
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
        out.push_str(", invisible=");
        out.push_str(if self.is_invisible_default {
            "true"
        } else {
            "false"
        });
        out.push_str(", depth=");
        out.push_str(&self.indentation_depth.to_string());
        out.push(')');

        if let Some(start_content) = &self.start_content {
            out.push('\n');
            push_indent(out, indent + 2);
            out.push_str("ContentList");
            start_content.write_parse_snapshot(out, indent + 4);
        }

        out.push('\n');
        push_indent(out, indent + 2);
        out.push_str("ContentList");
        self.inner_content.write_parse_snapshot(out, indent + 4);

        if let Some(condition) = &self.condition {
            out.push('\n');
            condition.write_parse_snapshot(out, indent + 2);
        }
    }
}
