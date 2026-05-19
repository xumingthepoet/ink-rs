use crate::source::SourceSpan;

use super::{push_indent, FlowArgument, TypeName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfaceMemberKind {
    Knot,
    Function,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceMemberSignature {
    kind: InterfaceMemberKind,
    name: String,
    name_span: SourceSpan,
    arguments: Vec<FlowArgument>,
    return_type: Option<TypeName>,
    has_typed_signature: bool,
    span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceDeclaration {
    name: String,
    name_span: SourceSpan,
    members: Vec<InterfaceMemberSignature>,
    span: SourceSpan,
}

impl InterfaceDeclaration {
    pub fn new(name: impl Into<String>, name_span: SourceSpan, span: SourceSpan) -> Self {
        Self::new_with_members(name, Vec::new(), name_span, span)
    }

    pub fn new_with_members(
        name: impl Into<String>,
        members: Vec<InterfaceMemberSignature>,
        name_span: SourceSpan,
        span: SourceSpan,
    ) -> Self {
        Self {
            name: name.into(),
            name_span,
            members,
            span,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_span(&self) -> &SourceSpan {
        &self.name_span
    }

    pub fn members(&self) -> &[InterfaceMemberSignature] {
        &self.members
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("Interface(name=\"");
        out.push_str(&self.name);
        out.push_str("\")");
    }
}

impl InterfaceMemberSignature {
    pub fn new(
        kind: InterfaceMemberKind,
        name: impl Into<String>,
        arguments: Vec<FlowArgument>,
        return_type: Option<TypeName>,
        has_typed_signature: bool,
        name_span: SourceSpan,
        span: SourceSpan,
    ) -> Self {
        Self {
            kind,
            name: name.into(),
            name_span,
            arguments,
            return_type,
            has_typed_signature,
            span,
        }
    }

    pub fn kind(&self) -> &InterfaceMemberKind {
        &self.kind
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_span(&self) -> &SourceSpan {
        &self.name_span
    }

    pub fn arguments(&self) -> &[FlowArgument] {
        &self.arguments
    }

    pub fn return_type(&self) -> Option<&TypeName> {
        self.return_type.as_ref()
    }

    pub fn has_typed_signature(&self) -> bool {
        self.has_typed_signature
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }
}

#[cfg(test)]
mod tests {
    use crate::source::SourceSpan;

    use super::*;

    #[test]
    fn writes_interface_snapshot() {
        let declaration = InterfaceDeclaration::new("IItem", span_at(1, 15), span_at(1, 1));
        let mut snapshot = String::new();

        declaration.write_parse_snapshot(&mut snapshot, 0);

        assert_eq!(snapshot, "\nInterface(name=\"IItem\")");
    }

    #[test]
    fn constructs_interface_with_member_signature_list() {
        let member = InterfaceMemberSignature::new(
            InterfaceMemberKind::Knot,
            "target",
            Vec::new(),
            None,
            false,
            span_at(2, 4),
            span_at(2, 1),
        );
        let declaration = InterfaceDeclaration::new_with_members(
            "IItem",
            vec![member.clone()],
            span_at(1, 15),
            span_at(1, 1),
        );

        assert_eq!(declaration.members(), &[member]);
    }

    fn span_at(line: usize, column: usize) -> SourceSpan {
        SourceSpan::new(Some("interface.ink".to_string()), line, column)
    }
}
