use crate::parsed::{
    visit::{walk_story, ParsedVisitor, VisitContext},
    Flow, Object, Story,
};

use super::context::{FlowSymbol, TargetSymbolIndex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum TargetSymbolCollectionPhase {
    #[default]
    Flows,
    Labels,
}

pub(super) fn build_target_symbol_index(story: &Story) -> TargetSymbolIndex {
    #[derive(Default)]
    struct TargetSymbolVisitor {
        phase: TargetSymbolCollectionPhase,
        symbols: TargetSymbolIndex,
    }

    impl ParsedVisitor for TargetSymbolVisitor {
        fn visit_flow(&mut self, flow: &Flow, context: &VisitContext) {
            if self.phase != TargetSymbolCollectionPhase::Flows {
                return;
            }

            let Some(flow_path) = &context.current_flow_path else {
                return;
            };
            let symbol = FlowSymbol {
                is_function: flow.is_function(),
            };

            self.symbols.entry(flow_path.clone()).or_insert(symbol);
            if context.parent_flow_path.is_none() {
                self.symbols
                    .entry(flow.name().to_string())
                    .or_insert(symbol);
            }
        }

        fn visit_object(&mut self, object: &Object, context: &VisitContext) {
            if self.phase != TargetSymbolCollectionPhase::Labels {
                return;
            }

            match object {
                Object::Choice(choice) => {
                    if let Some(identifier) = choice.identifier() {
                        insert_label_symbol(
                            &mut self.symbols,
                            identifier,
                            context.current_flow_path.as_deref(),
                        );
                    }
                }
                Object::Gather(gather) => {
                    if let Some(identifier) = gather.identifier() {
                        insert_label_symbol(
                            &mut self.symbols,
                            identifier,
                            context.current_flow_path.as_deref(),
                        );
                    }
                }
                _ => {}
            }
        }
    }

    let mut visitor = TargetSymbolVisitor::default();
    walk_story(story, &mut visitor);
    visitor.phase = TargetSymbolCollectionPhase::Labels;
    walk_story(story, &mut visitor);
    visitor.symbols
}

fn insert_label_symbol(symbols: &mut TargetSymbolIndex, identifier: &str, flow_path: Option<&str>) {
    let symbol = FlowSymbol { is_function: false };
    symbols.entry(identifier.to_string()).or_insert(symbol);
    if let Some(flow_path) = flow_path {
        symbols
            .entry(format!("{flow_path}.{identifier}"))
            .or_insert(symbol);
    }
}

pub(super) fn resolve_target_symbol<'a>(
    target: &str,
    current_flow_path: Option<&str>,
    target_symbols: &'a TargetSymbolIndex,
) -> Option<&'a FlowSymbol> {
    if target.contains('.') {
        return target_symbols.get(target);
    }

    if let Some(flow_path) = current_flow_path {
        if let Some(symbol) = target_symbols.get(&format!("{flow_path}.{target}")) {
            return Some(symbol);
        }
        if let Some((parent_flow_path, _)) = flow_path.rsplit_once('.') {
            if let Some(symbol) = target_symbols.get(&format!("{parent_flow_path}.{target}")) {
                return Some(symbol);
            }
        }
    }

    target_symbols.get(target)
}
