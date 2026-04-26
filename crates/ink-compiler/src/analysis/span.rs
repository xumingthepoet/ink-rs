use crate::{
    parsed::{Object, Weave},
    source::SourceSpan,
};

pub(super) fn first_span_in_weave(weave: &Weave) -> SourceSpan {
    weave
        .content()
        .first()
        .map(object_span)
        .unwrap_or_else(default_span)
}

pub(super) fn object_span(object: &Object) -> SourceSpan {
    match object {
        Object::Choice(choice) => choice.span().clone(),
        Object::AuthorWarning(author_warning) => author_warning.span().clone(),
        Object::ConstantDeclaration(declaration) => declaration.span().clone(),
        Object::Divert(divert) => divert.span().clone(),
        Object::ExternalDeclaration(external) => external.span().clone(),
        Object::Gather(gather) => gather.span().clone(),
        Object::IncDec(inc_dec) => inc_dec.span().clone(),
        Object::Return(ret) => ret.span().clone(),
        Object::StructDeclaration(declaration) => declaration.span().clone(),
        Object::Text(text) => text.span().clone(),
        Object::TunnelOnwards(tunnel_onwards) => tunnel_onwards.span().clone(),
        Object::VariableAssignment(assignment) => assignment.span().clone(),
        Object::ContentList(content) => content
            .objects()
            .first()
            .map(object_span)
            .unwrap_or_else(default_span),
        Object::Conditional(conditional) => conditional
            .branches()
            .iter()
            .flat_map(|branch| branch.content().content())
            .next()
            .map(object_span)
            .unwrap_or_else(default_span),
        Object::Sequence(sequence) => sequence
            .elements()
            .iter()
            .flat_map(|element| element.objects())
            .next()
            .map(object_span)
            .unwrap_or_else(default_span),
        Object::Weave(weave) => first_span_in_weave(weave),
        Object::Expression(_) | Object::Glue(_) | Object::LogicLine(_) | Object::Tag(_) => {
            default_span()
        }
    }
}

pub(super) fn default_span() -> SourceSpan {
    SourceSpan::new(None, 1, 1)
}
