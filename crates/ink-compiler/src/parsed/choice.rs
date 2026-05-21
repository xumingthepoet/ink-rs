use crate::source::SourceSpan;

use super::{push_indent, ContentList, Expression};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicChoiceVariable {
    source_name: String,
    runtime_name: String,
}

impl DynamicChoiceVariable {
    pub fn new(source_name: impl Into<String>, runtime_name: impl Into<String>) -> Self {
        Self {
            source_name: source_name.into(),
            runtime_name: runtime_name.into(),
        }
    }

    pub fn source_name(&self) -> &str {
        &self.source_name
    }

    pub fn runtime_name(&self) -> &str {
        &self.runtime_name
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicChoiceBinding {
    variables: Vec<DynamicChoiceVariable>,
    iterable: Expression,
    array_name: String,
    index_name: String,
    limit_name: String,
}

impl DynamicChoiceBinding {
    pub fn new(
        variables: Vec<DynamicChoiceVariable>,
        iterable: Expression,
        array_name: impl Into<String>,
        index_name: impl Into<String>,
        limit_name: impl Into<String>,
    ) -> Self {
        Self {
            variables,
            iterable,
            array_name: array_name.into(),
            index_name: index_name.into(),
            limit_name: limit_name.into(),
        }
    }

    pub fn variables(&self) -> &[DynamicChoiceVariable] {
        &self.variables
    }

    pub fn iterable(&self) -> &Expression {
        &self.iterable
    }

    pub fn array_name(&self) -> &str {
        &self.array_name
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }

    pub fn limit_name(&self) -> &str {
        &self.limit_name
    }

    pub fn with_iterable(mut self, iterable: Expression) -> Self {
        self.iterable = iterable;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    start_content: Option<ContentList>,
    inner_content: ContentList,
    span: SourceSpan,
    identifier: Option<String>,
    is_invisible_default: bool,
    indentation_depth: usize,
    condition: Option<Expression>,
    dynamic_binding: Option<DynamicChoiceBinding>,
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
            dynamic_binding: None,
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

    pub fn dynamic_binding(&self) -> Option<&DynamicChoiceBinding> {
        self.dynamic_binding.as_ref()
    }

    pub fn set_dynamic_binding(&mut self, dynamic_binding: Option<DynamicChoiceBinding>) {
        self.dynamic_binding = dynamic_binding;
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
        if let Some(binding) = &self.dynamic_binding {
            out.push_str(", dynamic=[");
            for (index, variable) in binding.variables.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                out.push_str(variable.source_name());
            }
            out.push_str(" in ");
            out.push_str(&binding.iterable.to_source_string());
            out.push(']');
        }
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
