use crate::{diagnostic::Diagnostic, parsed::Story, source::SourceSpan};

use super::super::{span::object_span, ModuleEntryPoint};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(in crate::analysis) struct ModuleEntryPointAnalysis {
    main_knots: Vec<ModuleMainKnot>,
    entry_point: Option<ModuleEntryPoint>,
}

pub(in crate::analysis) fn build_module_entry_point_analysis(
    story: &Story,
) -> ModuleEntryPointAnalysis {
    let main_knots = module_main_knots(story);
    let entry_point = (main_knots.len() == 1)
        .then(|| ModuleEntryPoint::new(main_knots[0].module.clone(), "main"));

    ModuleEntryPointAnalysis {
        main_knots,
        entry_point,
    }
}

pub(in crate::analysis) fn mixed_root_module_diagnostics(story: &Story) -> Vec<Diagnostic> {
    if story.modules().is_empty() {
        return Vec::new();
    }

    if let Some(object) = story.root_weave().content().first() {
        return vec![mixed_root_module_diagnostic(object_span(object))];
    }

    if let Some(flow) = story.flows().first() {
        return vec![mixed_root_module_diagnostic(flow.span().clone())];
    }

    Vec::new()
}

fn mixed_root_module_diagnostic(span: SourceSpan) -> Diagnostic {
    Diagnostic::error(
        span,
        "Explicit module compilation cannot be mixed with root story content or top-level flows",
    )
}

impl ModuleEntryPointAnalysis {
    pub(in crate::analysis) fn entry_point(&self) -> Option<&ModuleEntryPoint> {
        self.entry_point.as_ref()
    }

    pub(in crate::analysis) fn into_entry_point(self) -> Option<ModuleEntryPoint> {
        self.entry_point
    }
}

pub(in crate::analysis) fn module_entry_point_diagnostics(
    story: &Story,
    analysis: &ModuleEntryPointAnalysis,
) -> Vec<Diagnostic> {
    if story.modules().is_empty() {
        return Vec::new();
    }

    match analysis.main_knots.len() {
        0 => vec![Diagnostic::error(
            story.modules()[0].span().clone(),
            "Explicit module compilation requires exactly one module to define a knot named 'main'",
        )],
        1 => Vec::new(),
        _ => analysis
            .main_knots
            .iter()
            .skip(1)
            .map(|main| {
                Diagnostic::error(
                    main.span.clone(),
                    "Multiple 'main' knots are declared; runnable entry point must be unique",
                )
            })
            .collect(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModuleMainKnot {
    module: String,
    span: SourceSpan,
}

fn module_main_knots(story: &Story) -> Vec<ModuleMainKnot> {
    story
        .modules()
        .iter()
        .flat_map(|module| {
            module
                .flows()
                .iter()
                .filter(|flow| !flow.is_function() && flow.name() == "main")
                .map(|flow| ModuleMainKnot {
                    module: module.name().to_string(),
                    span: flow.span().clone(),
                })
        })
        .collect()
}
