use crate::source::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
