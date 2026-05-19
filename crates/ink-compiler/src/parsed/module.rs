use crate::source::SourceSpan;

use super::{push_indent, Flow, Object, Weave};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedName {
    name: String,
    span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementedInterface {
    name: String,
    span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ImportKind {
    Module,
    Symbols(Vec<ImportedName>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDeclaration {
    kind: ImportKind,
    source_module: String,
    source_module_span: SourceSpan,
    span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    name: String,
    name_span: SourceSpan,
    span: SourceSpan,
    implemented_interfaces: Vec<ImplementedInterface>,
    imports: Vec<ImportDeclaration>,
    weave: Weave,
    flows: Vec<Flow>,
}

impl ImportedName {
    pub fn new(name: impl Into<String>, span: SourceSpan) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }
}

impl ImplementedInterface {
    pub fn new(name: impl Into<String>, span: SourceSpan) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }
}

impl ImportDeclaration {
    pub fn new(
        imported_names: Vec<ImportedName>,
        source_module: impl Into<String>,
        source_module_span: SourceSpan,
        span: SourceSpan,
    ) -> Self {
        Self::symbols(imported_names, source_module, source_module_span, span)
    }

    pub fn module(
        source_module: impl Into<String>,
        source_module_span: SourceSpan,
        span: SourceSpan,
    ) -> Self {
        Self {
            kind: ImportKind::Module,
            source_module: source_module.into(),
            source_module_span,
            span,
        }
    }

    pub fn symbols(
        imported_names: Vec<ImportedName>,
        source_module: impl Into<String>,
        source_module_span: SourceSpan,
        span: SourceSpan,
    ) -> Self {
        Self {
            kind: ImportKind::Symbols(imported_names),
            source_module: source_module.into(),
            source_module_span,
            span,
        }
    }

    pub fn imported_names(&self) -> &[ImportedName] {
        match &self.kind {
            ImportKind::Module => &[],
            ImportKind::Symbols(imported_names) => imported_names,
        }
    }

    pub fn is_module_import(&self) -> bool {
        matches!(self.kind, ImportKind::Module)
    }

    pub fn is_symbol_import(&self) -> bool {
        matches!(self.kind, ImportKind::Symbols(_))
    }

    pub fn source_module(&self) -> &str {
        &self.source_module
    }

    pub fn source_module_span(&self) -> &SourceSpan {
        &self.source_module_span
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("Import(from=\"");
        out.push_str(&self.source_module);
        if self.is_module_import() {
            out.push_str("\", module=true)");
            return;
        }
        out.push_str("\", names=[");
        for (index, imported_name) in self.imported_names().iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push('"');
            out.push_str(imported_name.name());
            out.push('"');
        }
        out.push_str("])");
    }
}

impl Module {
    pub fn new(
        name: impl Into<String>,
        imports: Vec<ImportDeclaration>,
        content: Vec<Object>,
        flows: Vec<Flow>,
        name_span: SourceSpan,
        span: SourceSpan,
    ) -> Self {
        Self::new_with_implemented_interfaces(
            name,
            Vec::new(),
            imports,
            content,
            flows,
            name_span,
            span,
        )
    }

    pub fn new_with_implemented_interfaces(
        name: impl Into<String>,
        implemented_interfaces: Vec<ImplementedInterface>,
        imports: Vec<ImportDeclaration>,
        content: Vec<Object>,
        flows: Vec<Flow>,
        name_span: SourceSpan,
        span: SourceSpan,
    ) -> Self {
        Self {
            name: name.into(),
            name_span,
            span,
            implemented_interfaces,
            imports,
            weave: Weave::new(content, 0),
            flows,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_span(&self) -> &SourceSpan {
        &self.name_span
    }

    pub fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub fn imports(&self) -> &[ImportDeclaration] {
        &self.imports
    }

    pub fn implemented_interfaces(&self) -> &[ImplementedInterface] {
        &self.implemented_interfaces
    }

    pub(crate) fn push_import(&mut self, import: ImportDeclaration) {
        self.imports.push(import);
    }

    pub(crate) fn push_objects(&mut self, objects: Vec<Object>) {
        let mut content = self.weave.content().to_vec();
        content.extend(objects);
        self.weave = Weave::new(content, self.weave.base_indent());
    }

    pub(crate) fn push_flow(&mut self, flow: Flow) {
        self.flows.push(flow);
    }

    pub fn weave(&self) -> &Weave {
        &self.weave
    }

    pub fn flows(&self) -> &[Flow] {
        &self.flows
    }

    pub(crate) fn write_parse_snapshot(&self, out: &mut String, indent: usize) {
        out.push('\n');
        push_indent(out, indent);
        out.push_str("Module(name=\"");
        out.push_str(&self.name);
        if self.implemented_interfaces.is_empty() {
            out.push_str("\")");
        } else {
            out.push_str("\", implements=[");
            for (index, implemented_interface) in self.implemented_interfaces.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                out.push('"');
                out.push_str(implemented_interface.name());
                out.push('"');
            }
            out.push_str("])");
        }
        for import in &self.imports {
            import.write_parse_snapshot(out, indent + 2);
        }
        if !self.weave.content().is_empty() {
            self.weave.write_parse_snapshot(out, indent + 2);
        }
        for flow in &self.flows {
            flow.write_parse_snapshot(out, indent + 2);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parsed::{Flow, FlowLevel, FlowParts, ImplementedInterface, Object, Text},
        source::SourceSpan,
    };

    use super::*;

    #[test]
    fn constructs_module_and_import_with_spans() {
        let import_name_span = span_at(2, 8);
        let source_module_span = span_at(2, 19);
        let import_span = span_at(2, 1);
        let module_name_span = span_at(1, 12);
        let module_span = span_at(1, 1);
        let import = ImportDeclaration::new(
            vec![ImportedName::new("sword", import_name_span.clone())],
            "items",
            source_module_span.clone(),
            import_span.clone(),
        );
        let module = Module::new(
            "game",
            vec![import.clone()],
            vec![Object::Text(Text::new("Line.", span_at(3, 1)))],
            vec![Flow::from_parts(FlowParts::new(
                FlowLevel::Knot,
                "main",
                Vec::new(),
            ))],
            module_name_span.clone(),
            module_span.clone(),
        );

        assert_eq!(module.name(), "game");
        assert_eq!(module.name_span(), &module_name_span);
        assert_eq!(module.span(), &module_span);
        assert!(module.implemented_interfaces().is_empty());
        assert_eq!(module.imports(), &[import]);
        assert!(module.imports()[0].is_symbol_import());
        assert!(!module.imports()[0].is_module_import());
        assert_eq!(module.imports()[0].imported_names()[0].name(), "sword");
        assert_eq!(
            module.imports()[0].imported_names()[0].span(),
            &import_name_span
        );
        assert_eq!(module.imports()[0].source_module(), "items");
        assert_eq!(
            module.imports()[0].source_module_span(),
            &source_module_span
        );
        assert_eq!(module.imports()[0].span(), &import_span);
        assert_eq!(module.flows()[0].name(), "main");
    }

    #[test]
    fn constructs_module_import_and_implemented_interface_with_spans() {
        let interface_span = span_at(1, 29);
        let import_span = span_at(2, 1);
        let source_module_span = span_at(2, 6);
        let module = Module::new_with_implemented_interfaces(
            "game",
            vec![ImplementedInterface::new("IItem", interface_span.clone())],
            vec![ImportDeclaration::module(
                "items",
                source_module_span.clone(),
                import_span.clone(),
            )],
            Vec::new(),
            Vec::new(),
            span_at(1, 12),
            span_at(1, 1),
        );

        assert_eq!(module.implemented_interfaces()[0].name(), "IItem");
        assert_eq!(module.implemented_interfaces()[0].span(), &interface_span);
        assert!(module.imports()[0].is_module_import());
        assert!(!module.imports()[0].is_symbol_import());
        assert!(module.imports()[0].imported_names().is_empty());
        assert_eq!(module.imports()[0].source_module(), "items");
        assert_eq!(
            module.imports()[0].source_module_span(),
            &source_module_span
        );
        assert_eq!(module.imports()[0].span(), &import_span);
    }

    #[test]
    fn writes_module_import_snapshot() {
        let module = Module::new(
            "game",
            vec![ImportDeclaration::new(
                vec![
                    ImportedName::new("sword", span_at(2, 8)),
                    ImportedName::new("heal", span_at(2, 15)),
                ],
                "items",
                span_at(2, 25),
                span_at(2, 1),
            )],
            vec![Object::Text(Text::new("Line.", span_at(3, 1)))],
            vec![Flow::from_parts(FlowParts::new(
                FlowLevel::Knot,
                "main",
                Vec::new(),
            ))],
            span_at(1, 12),
            span_at(1, 1),
        );
        let mut snapshot = String::new();

        module.write_parse_snapshot(&mut snapshot, 0);

        assert_eq!(
            snapshot,
            "\nModule(name=\"game\")\n  Import(from=\"items\", names=[\"sword\", \"heal\"])\n  Weave(baseIndent=0)\n    Text(\"Line.\")\n  Flow(level=Knot, name=\"main\", function=false)"
        );
    }

    #[test]
    fn writes_module_import_and_implements_snapshot() {
        let module = Module::new_with_implemented_interfaces(
            "game",
            vec![
                ImplementedInterface::new("IItem", span_at(1, 29)),
                ImplementedInterface::new("IOther", span_at(1, 36)),
            ],
            vec![ImportDeclaration::module(
                "items",
                span_at(2, 6),
                span_at(2, 1),
            )],
            Vec::new(),
            Vec::new(),
            span_at(1, 12),
            span_at(1, 1),
        );
        let mut snapshot = String::new();

        module.write_parse_snapshot(&mut snapshot, 0);

        assert_eq!(
            snapshot,
            "\nModule(name=\"game\", implements=[\"IItem\", \"IOther\"])\n  Import(from=\"items\", module=true)"
        );
    }

    fn span_at(line: usize, column: usize) -> SourceSpan {
        SourceSpan::new(Some("module.ink".to_string()), line, column)
    }
}
