mod author_warning;
mod choice;
mod conditional;
mod constant_declaration;
mod content_list;
mod divert;
mod expression;
mod external_declaration;
mod flow;
mod gather;
mod glue;
mod inc_dec;
mod module;
mod qualified_name;
mod return_node;
mod sequence;
mod story;
mod struct_declaration;
mod tag;
mod text;
mod tunnel_onwards;
mod type_name;
mod variable_assignment;
pub(crate) mod visit;
mod weave;

pub use author_warning::AuthorWarning;
pub use choice::Choice;
pub use conditional::{Conditional, ConditionalBranch};
pub use constant_declaration::ConstantDeclaration;
pub use content_list::ContentList;
pub use divert::{Divert, DivertTarget};
pub use expression::{BinaryOperator, Expression, FloatLiteral, StructLiteralField, UnaryOperator};
pub use external_declaration::ExternalDeclaration;
pub use flow::{Flow, FlowArgument, FlowLevel};
pub use gather::Gather;
pub use glue::Glue;
pub use inc_dec::IncDec;
pub use module::{ImportDeclaration, ImportedName, Module};
pub use qualified_name::QualifiedName;
pub use return_node::Return;
pub use sequence::{Sequence, SequenceType};
pub use story::Story;
pub use struct_declaration::{StructDeclaration, StructField};
pub use tag::Tag;
pub use text::Text;
pub use tunnel_onwards::TunnelOnwards;
pub use type_name::{DefaultValue, PrimitiveType, TypeName};
pub use variable_assignment::{AssignmentTarget, VariableAssignment};
pub use weave::Weave;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Text(Text),
    AuthorWarning(AuthorWarning),
    ContentList(ContentList),
    Expression(Expression),
    Conditional(Conditional),
    ConstantDeclaration(ConstantDeclaration),
    LogicLine(Expression),
    Glue(Glue),
    IncDec(IncDec),
    Choice(Choice),
    Divert(Divert),
    Gather(Gather),
    Tag(Tag),
    Sequence(Sequence),
    Return(Return),
    StructDeclaration(StructDeclaration),
    TunnelOnwards(TunnelOnwards),
    VariableAssignment(VariableAssignment),
    ExternalDeclaration(ExternalDeclaration),
    Weave(Weave),
}

impl Object {
    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        match self {
            Object::Text(text) => text.write_parse_snapshot(out, indent),
            Object::AuthorWarning(author_warning) => {
                author_warning.write_parse_snapshot(out, indent)
            }
            Object::ContentList(content_list) => {
                out.push('\n');
                push_indent(out, indent);
                out.push_str("ContentList");
                content_list.write_parse_snapshot(out, indent + 2);
            }
            Object::Expression(expression) => {
                out.push('\n');
                expression.write_parse_snapshot(out, indent);
            }
            Object::Conditional(conditional) => {
                out.push('\n');
                conditional.write_parse_snapshot(out, indent);
            }
            Object::ConstantDeclaration(declaration) => {
                declaration.write_parse_snapshot(out, indent)
            }
            Object::LogicLine(expression) => {
                out.push('\n');
                push_indent(out, indent);
                out.push_str("ContentList");
                out.push('\n');
                expression.write_parse_snapshot(out, indent + 2);
                Text::new("\n", crate::source::SourceSpan::new(None, 1, 1))
                    .write_parse_snapshot(out, indent + 2);
            }
            Object::Glue(glue) => glue.write_parse_snapshot(out, indent),
            Object::IncDec(inc_dec) => inc_dec.write_parse_snapshot(out, indent),
            Object::Return(ret) => ret.write_parse_snapshot(out, indent),
            Object::StructDeclaration(declaration) => declaration.write_parse_snapshot(out, indent),
            Object::Choice(choice) => choice.write_parse_snapshot(out, indent),
            Object::Divert(divert) => divert.write_parse_snapshot(out, indent),
            Object::Gather(gather) => gather.write_parse_snapshot(out, indent),
            Object::Tag(tag) => tag.write_parse_snapshot(out, indent),
            Object::Sequence(sequence) => sequence.write_parse_snapshot(out, indent),
            Object::VariableAssignment(assignment) => assignment.write_parse_snapshot(out, indent),
            Object::ExternalDeclaration(external) => external.write_parse_snapshot(out, indent),
            Object::TunnelOnwards(tunnel_onwards) => {
                tunnel_onwards.write_parse_snapshot(out, indent)
            }
            Object::Weave(weave) => weave.write_parse_snapshot(out, indent),
        }
    }
}

pub(crate) fn push_indent(out: &mut String, indent: usize) {
    out.push_str(&" ".repeat(indent));
}

pub(crate) fn escape_snapshot_text(text: &str) -> String {
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
