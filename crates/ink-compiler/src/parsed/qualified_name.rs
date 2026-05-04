use crate::source::SourceSpan;

#[derive(Debug, Clone)]
pub struct QualifiedName {
    module: String,
    symbol: String,
    module_span: SourceSpan,
    symbol_span: SourceSpan,
    source: String,
}

impl QualifiedName {
    pub fn new(
        module: impl Into<String>,
        module_span: SourceSpan,
        symbol: impl Into<String>,
        symbol_span: SourceSpan,
    ) -> Self {
        let module = module.into();
        let symbol = symbol.into();
        let source = format!("{module}::{symbol}");
        Self {
            module,
            symbol,
            module_span,
            symbol_span,
            source,
        }
    }

    pub fn module(&self) -> &str {
        &self.module
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn module_span(&self) -> &SourceSpan {
        &self.module_span
    }

    pub fn symbol_span(&self) -> &SourceSpan {
        &self.symbol_span
    }

    pub fn as_str(&self) -> &str {
        &self.source
    }
}

impl PartialEq for QualifiedName {
    fn eq(&self, other: &Self) -> bool {
        self.module == other.module && self.symbol == other.symbol
    }
}

impl Eq for QualifiedName {}

impl std::hash::Hash for QualifiedName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.module.hash(state);
        self.symbol.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality_ignores_source_spans() {
        let first = QualifiedName::new(
            "data",
            SourceSpan::new(Some("a.ink".to_string()), 1, 1),
            "State",
            SourceSpan::new(Some("a.ink".to_string()), 1, 7),
        );
        let second = QualifiedName::new(
            "data",
            SourceSpan::new(Some("b.ink".to_string()), 10, 3),
            "State",
            SourceSpan::new(Some("b.ink".to_string()), 10, 9),
        );

        assert_eq!(first, second);
    }
}
