use crate::parsed::{
    visit::{walk_story, ParsedVisitor, VisitContext},
    ExternalDeclaration, Flow, Object, Story,
};

use super::context::{FlowSymbol, ParameterSymbol, TargetSymbolIndex};

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
            let parameters = flow
                .arguments()
                .iter()
                .map(ParameterSymbol::from_flow_argument)
                .collect();
            let symbol = FlowSymbol::new(
                flow.is_function(),
                parameters,
                flow.return_type().clone(),
                flow.has_typed_signature(),
            );
            let scoped_flow_path = scoped_symbol_name(context.current_module.as_deref(), flow_path);

            self.symbols
                .entry(scoped_flow_path)
                .or_insert_with(|| symbol.clone());
            if context.parent_flow_path.is_none() {
                let scoped_name =
                    scoped_symbol_name(context.current_module.as_deref(), flow.name());
                self.symbols.entry(scoped_name).or_insert(symbol);
            }
        }

        fn visit_object(&mut self, object: &Object, context: &VisitContext) {
            match self.phase {
                TargetSymbolCollectionPhase::Flows => {
                    if let Object::ExternalDeclaration(external) = object {
                        insert_external_symbol(
                            &mut self.symbols,
                            external,
                            context.current_module.as_deref(),
                        );
                    }
                }
                TargetSymbolCollectionPhase::Labels => match object {
                    Object::Choice(choice) => {
                        if let Some(identifier) = choice.identifier() {
                            insert_label_symbol(
                                &mut self.symbols,
                                identifier,
                                context.current_module.as_deref(),
                                context.current_flow_path.as_deref(),
                            );
                        }
                    }
                    Object::Gather(gather) => {
                        if let Some(identifier) = gather.identifier() {
                            insert_label_symbol(
                                &mut self.symbols,
                                identifier,
                                context.current_module.as_deref(),
                                context.current_flow_path.as_deref(),
                            );
                        }
                    }
                    _ => {}
                },
            }
        }
    }

    let mut visitor = TargetSymbolVisitor::default();
    walk_story(story, &mut visitor);
    visitor.phase = TargetSymbolCollectionPhase::Labels;
    walk_story(story, &mut visitor);
    visitor.symbols
}

fn insert_external_symbol(
    symbols: &mut TargetSymbolIndex,
    external: &ExternalDeclaration,
    module: Option<&str>,
) {
    let parameters = external
        .argument_names()
        .iter()
        .zip(external.argument_types())
        .map(|(name, declared_type)| {
            ParameterSymbol::new(name.clone(), Some(declared_type.clone()))
        })
        .collect();
    let symbol = FlowSymbol::new(true, parameters, external.return_type().clone(), true);

    symbols
        .entry(scoped_symbol_name(module, external.name()))
        .or_insert(symbol);
}

fn insert_label_symbol(
    symbols: &mut TargetSymbolIndex,
    identifier: &str,
    module: Option<&str>,
    flow_path: Option<&str>,
) {
    let symbol = FlowSymbol::label();
    symbols
        .entry(scoped_symbol_name(module, identifier))
        .or_insert_with(|| symbol.clone());
    if let Some(flow_path) = flow_path {
        symbols
            .entry(scoped_symbol_name(
                module,
                &format!("{flow_path}.{identifier}"),
            ))
            .or_insert(symbol);
    }
}

pub(super) fn resolve_target_symbol<'a>(
    target: &str,
    current_module: Option<&str>,
    current_flow_path: Option<&str>,
    target_symbols: &'a TargetSymbolIndex,
) -> Option<&'a FlowSymbol> {
    if is_cross_module_stitch_target(target, current_module) {
        return None;
    }

    if target.contains("::") {
        return target_symbols.get(target);
    }

    if target.contains('.') {
        return target_symbols.get(&scoped_symbol_name(current_module, target));
    }

    if let Some(flow_path) = current_flow_path {
        if let Some(symbol) = target_symbols.get(&scoped_symbol_name(
            current_module,
            &format!("{flow_path}.{target}"),
        )) {
            return Some(symbol);
        }
        if let Some((parent_flow_path, _)) = flow_path.rsplit_once('.') {
            if let Some(symbol) = target_symbols.get(&scoped_symbol_name(
                current_module,
                &format!("{parent_flow_path}.{target}"),
            )) {
                return Some(symbol);
            }
        }
    }

    target_symbols.get(&scoped_symbol_name(current_module, target))
}

fn scoped_symbol_name(module: Option<&str>, name: &str) -> String {
    module
        .map(|module| format!("{module}::{name}"))
        .unwrap_or_else(|| name.to_string())
}

pub(super) fn is_cross_module_stitch_target(target: &str, current_module: Option<&str>) -> bool {
    let Some((module, symbol)) = target.split_once("::") else {
        return false;
    };
    symbol.contains('.') && Some(module) != current_module
}
