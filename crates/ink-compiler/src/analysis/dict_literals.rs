use std::collections::{BTreeSet, HashSet};

use crate::{
    diagnostic::Diagnostic,
    parsed::{
        escape_snapshot_text,
        visit::{walk_story, ParsedVisitor, VisitContext},
        ConstantDeclaration, DictKeyType, DictLiteralEntry, DictLiteralKey, Divert, DivertTarget,
        Expression, InterfaceMemberKind, InterfaceMemberSignature, Object, Story,
        StructLiteralField, TunnelOnwards, TypeName, VariableAssignment,
    },
    source::SourceSpan,
};

use super::{
    context::{EnumTypeIndex, FlowSymbol, StructTypeIndex, TargetSymbolIndex, VariableScopeIndex},
    enums::type_name_contains_enum,
    expression_types::infer_expression_type,
    indexes::AnalysisIndexes,
    interface_values::{infer_expected_interface_expression_type, ModuleImplementationIndex},
    interfaces::InterfaceMemberIndex,
    modules::ModuleImportIndex,
    structs::resolve_struct_symbol,
    target_symbols::resolve_target_symbol,
    type_names::qualify_type_name_for_module,
};

#[cfg(test)]
use super::modules::ModuleAnalysis;

#[cfg(test)]
pub(super) fn dict_literal_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let module_analysis = ModuleAnalysis::build(story);
    let indexes = AnalysisIndexes::build(story, &module_analysis);
    dict_literal_diagnostics_with_indexes(story, &indexes)
}

pub(super) fn dict_literal_diagnostics_with_indexes(
    story: &Story,
    indexes: &AnalysisIndexes<'_>,
) -> Vec<Diagnostic> {
    let mut checker = DictLiteralChecker::new(
        &indexes.struct_types,
        &indexes.enum_types,
        &indexes.variable_scopes,
        &indexes.target_symbols,
        &indexes.module_implementations,
        indexes.module_imports,
        &indexes.interface_members,
    );
    walk_story(story, &mut checker);
    checker.diagnostics
}

struct DictLiteralChecker<'a> {
    struct_types: &'a StructTypeIndex,
    enum_types: &'a EnumTypeIndex,
    variable_scopes: &'a VariableScopeIndex,
    target_symbols: &'a TargetSymbolIndex,
    module_implementations: &'a ModuleImplementationIndex,
    module_imports: &'a ModuleImportIndex,
    interface_members: &'a InterfaceMemberIndex,
    diagnostics: Vec<Diagnostic>,
    expected_expression_ids: HashSet<usize>,
}

impl<'a> DictLiteralChecker<'a> {
    fn new(
        struct_types: &'a StructTypeIndex,
        enum_types: &'a EnumTypeIndex,
        variable_scopes: &'a VariableScopeIndex,
        target_symbols: &'a TargetSymbolIndex,
        module_implementations: &'a ModuleImplementationIndex,
        module_imports: &'a ModuleImportIndex,
        interface_members: &'a InterfaceMemberIndex,
    ) -> Self {
        Self {
            struct_types,
            enum_types,
            variable_scopes,
            target_symbols,
            module_implementations,
            module_imports,
            interface_members,
            diagnostics: Vec::new(),
            expected_expression_ids: HashSet::new(),
        }
    }

    fn check_assignment(&mut self, assignment: &VariableAssignment, context: &VisitContext) {
        let Some(expression) = assignment.expression() else {
            return;
        };

        let expected_type = if assignment.is_global() || assignment.is_temporary() {
            assignment.declared_type().cloned()
        } else {
            assignment
                .target()
                .variable_name()
                .and_then(|name| self.visible_declared_type(name, context))
        };

        let Some(expected_type) = expected_type else {
            return;
        };

        if self.type_name_contains_dict(&expected_type, context.current_module.as_deref())
            || expression_contains_dict_literal(expression)
        {
            self.check_expression_against_type(
                expression,
                &expected_type,
                assignment.name(),
                assignment.span(),
                context,
            );
        }
    }

    fn check_constant(&mut self, declaration: &ConstantDeclaration, context: &VisitContext) {
        if self.type_name_contains_dict(
            declaration.declared_type(),
            context.current_module.as_deref(),
        ) || expression_contains_dict_literal(declaration.expression())
        {
            self.check_expression_against_type(
                declaration.expression(),
                declaration.declared_type(),
                declaration.name(),
                declaration.span(),
                context,
            );
        }
    }

    fn check_divert_arguments(&mut self, divert: &Divert, context: &VisitContext) {
        self.check_divert_target_arguments(
            divert.target(),
            divert.arguments(),
            divert.span(),
            context,
        );
    }

    fn check_tunnel_onwards_arguments(
        &mut self,
        tunnel_onwards: &TunnelOnwards,
        context: &VisitContext,
    ) {
        if let Some(target) = tunnel_onwards.override_target() {
            self.check_divert_target_arguments(
                target,
                tunnel_onwards.arguments(),
                tunnel_onwards.span(),
                context,
            );
        }
    }

    fn check_divert_target_arguments(
        &mut self,
        target: &DivertTarget,
        arguments: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        match target {
            DivertTarget::Path(target) => {
                self.check_static_target_arguments(target, arguments, span, context);
            }
            DivertTarget::QualifiedPath(target) => {
                self.check_static_target_arguments(target.as_str(), arguments, span, context);
            }
            DivertTarget::Dynamic(Expression::DynamicInterfaceAccess { target, member }) => {
                if let Some(signature) = self.dynamic_interface_signature(
                    target,
                    member,
                    InterfaceMemberKind::Knot,
                    context,
                ) {
                    self.check_interface_signature_arguments(
                        member, arguments, &signature, span, context,
                    );
                }
            }
            DivertTarget::Dynamic(_)
            | DivertTarget::Done
            | DivertTarget::End
            | DivertTarget::Empty => {}
        }
    }

    fn check_static_target_arguments(
        &mut self,
        target: &str,
        arguments: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let Some(symbol) = resolve_target_symbol(
            target,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
            self.target_symbols,
        )
        .cloned() else {
            return;
        };

        self.check_flow_symbol_arguments(target, arguments, &symbol, span, context);
    }

    fn check_function_call_arguments(
        &mut self,
        name: &str,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let Some(symbol) = resolve_target_symbol(
            name,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
            self.target_symbols,
        )
        .cloned() else {
            return;
        };

        self.check_flow_symbol_arguments(name, args, &symbol, span, context);
    }

    fn check_dynamic_interface_function_arguments(
        &mut self,
        target: &Expression,
        member: &str,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if let Some(signature) =
            self.dynamic_interface_signature(target, member, InterfaceMemberKind::Function, context)
        {
            self.check_interface_signature_arguments(member, args, &signature, span, context);
        }
    }

    fn check_flow_symbol_arguments(
        &mut self,
        target_name: &str,
        arguments: &[Expression],
        symbol: &FlowSymbol,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let qualified_module = target_name.split_once("::").map(|(module, _)| module);
        for (argument, parameter) in arguments.iter().zip(symbol.arguments()) {
            let Some(expected_type) = parameter.declared_type() else {
                continue;
            };
            let expected_type = qualified_module
                .map(|module| qualify_type_name_for_module(expected_type, module))
                .unwrap_or_else(|| expected_type.clone());
            if self.type_name_contains_dict(&expected_type, context.current_module.as_deref())
                || expression_contains_dict_literal(argument)
            {
                self.check_expression_against_type(
                    argument,
                    &expected_type,
                    parameter.name(),
                    span,
                    context,
                );
            }
        }
    }

    fn check_interface_signature_arguments(
        &mut self,
        member: &str,
        arguments: &[Expression],
        signature: &InterfaceMemberSignature,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        for (argument, parameter) in arguments.iter().zip(signature.arguments()) {
            let Some(expected_type) = parameter.declared_type() else {
                continue;
            };
            if self.type_name_contains_dict(expected_type, context.current_module.as_deref())
                || expression_contains_dict_literal(argument)
            {
                let context_name = format!("{member}.{}", parameter.name());
                self.check_expression_against_type(
                    argument,
                    expected_type,
                    &context_name,
                    span,
                    context,
                );
            }
        }
    }

    fn dynamic_interface_signature(
        &self,
        target: &Expression,
        member: &str,
        expected_kind: InterfaceMemberKind,
        context: &VisitContext,
    ) -> Option<InterfaceMemberSignature> {
        let target_type = infer_expression_type(
            target,
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            self.interface_members,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        )
        .ok()?;
        let interface_name = target_type.as_interface_name()?;
        let signature = self.interface_members.member(interface_name, member)?;
        (signature.kind() == &expected_kind).then(|| signature.clone())
    }

    fn visible_declared_type(&self, name: &str, context: &VisitContext) -> Option<TypeName> {
        self.variable_scopes
            .visible_variable_declared_type(
                name,
                context.current_module.as_deref(),
                context.current_flow_path.as_deref(),
            )
            .and_then(|declared_type| declared_type.cloned())
    }

    fn check_expression_against_type(
        &mut self,
        expression: &Expression,
        expected_type: &TypeName,
        context_name: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        self.mark_expected_expression(expression);
        match (expected_type, expression) {
            (
                TypeName::Dict {
                    key_type,
                    value_type,
                },
                Expression::DictLiteral(entries),
            ) => {
                self.check_dict_literal(*key_type, value_type, entries, context_name, span, context)
            }
            (TypeName::Dict { .. }, Expression::ArrayLiteral(_)) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Value for '{}' is an array literal but expected {}",
                        context_name,
                        expected_type.display_name()
                    ),
                ));
            }
            (TypeName::Dict { .. }, Expression::StructLiteral { .. }) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Value for '{}' is a struct literal but expected {}",
                        context_name,
                        expected_type.display_name()
                    ),
                ));
            }
            (TypeName::Dict { .. }, _) => {
                self.check_exact_expression_type(
                    expression,
                    expected_type,
                    context_name,
                    span,
                    context,
                );
            }
            (TypeName::Array(element_type), Expression::ArrayLiteral(elements)) => {
                self.check_array_literal(element_type, elements, context_name, span, context);
            }
            (TypeName::Struct(struct_name), Expression::StructLiteral { fields, .. }) => {
                self.check_struct_literal(struct_name, fields, span, context);
            }
            (TypeName::QualifiedStruct(struct_name), Expression::StructLiteral { fields, .. }) => {
                self.check_struct_literal(struct_name.as_str(), fields, span, context);
            }
            (_, Expression::DictLiteral(_)) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Value for '{}' is a Dict literal but expected {}",
                        context_name,
                        expected_type.display_name()
                    ),
                ));
            }
            (_, Expression::ArrayLiteral(_)) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Value for '{}' is an array literal but expected {}",
                        context_name,
                        expected_type.display_name()
                    ),
                ));
            }
            (_, Expression::StructLiteral { .. }) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Value for '{}' is a struct literal but expected {}",
                        context_name,
                        expected_type.display_name()
                    ),
                ));
            }
            _ => self.check_exact_expression_type(
                expression,
                expected_type,
                context_name,
                span,
                context,
            ),
        }
    }

    fn check_dict_literal(
        &mut self,
        key_type: DictKeyType,
        value_type: &TypeName,
        entries: &[DictLiteralEntry],
        context_name: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        self.check_dict_literal_key_consistency(entries, context_name, span);
        let mut seen_keys = HashSet::new();
        for entry in entries {
            if !seen_keys.insert(entry.key().clone()) {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Duplicate key {} in Dict literal for '{}'",
                        dict_literal_key_display(entry.key()),
                        context_name
                    ),
                ));
            }

            if !dict_key_matches(key_type, entry.key()) {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Dict literal key for '{}' has type {} but expected {}",
                        context_name,
                        dict_literal_key_type_name(entry.key()),
                        key_type
                    ),
                ));
            }

            let value_context = dict_entry_context(context_name, entry.key());
            self.check_expression_against_type(
                entry.value(),
                value_type,
                &value_context,
                span,
                context,
            );
        }
    }

    fn check_dict_literal_key_consistency(
        &mut self,
        entries: &[DictLiteralEntry],
        context_name: &str,
        span: &SourceSpan,
    ) {
        let mut key_types = entries
            .iter()
            .map(|entry| dict_literal_key_type_name(entry.key()));
        let Some(first_type) = key_types.next() else {
            return;
        };
        if key_types.any(|key_type| key_type != first_type) {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Dict literal for '{}' mixes string and int keys; use one key type per Dict literal",
                    context_name
                ),
            ));
        }
    }

    fn check_array_literal(
        &mut self,
        element_type: &TypeName,
        elements: &[Expression],
        context_name: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        for (index, element) in elements.iter().enumerate() {
            let element_context = format!("{context_name}[{index}]");
            self.check_expression_against_type(
                element,
                element_type,
                &element_context,
                span,
                context,
            );
        }
    }

    fn check_struct_literal(
        &mut self,
        struct_name: &str,
        fields: &[StructLiteralField],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let Some(symbol) = resolve_struct_symbol(
            self.struct_types,
            struct_name,
            context.current_module.as_deref(),
        ) else {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!("Unknown struct type '{struct_name}' for struct literal"),
            ));
            return;
        };

        let mut provided_fields = BTreeSet::new();
        for field in fields {
            if !provided_fields.insert(field.name().to_string()) {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Duplicate field '{}' in struct literal for '{}'",
                        field.name(),
                        struct_name
                    ),
                ));
                continue;
            }

            let Some(field_type) = symbol.fields().get(field.name()) else {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Unknown field '{}' in struct literal for '{}'",
                        field.name(),
                        struct_name
                    ),
                ));
                continue;
            };

            let field_context = format!("{struct_name}.{}", field.name());
            self.check_expression_against_type(
                field.expression(),
                field_type,
                &field_context,
                span,
                context,
            );
        }

        for (field_name, field_type) in symbol.fields() {
            if provided_fields.contains(field_name) {
                continue;
            }
            if field_type.default_value().is_none() {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Missing field '{}' in struct literal for '{}' cannot be default-initialized",
                        field_name, struct_name
                    ),
                ));
            }
        }
    }

    fn check_exact_expression_type(
        &mut self,
        expression: &Expression,
        expected_type: &TypeName,
        context_name: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if let Some(result) = infer_expected_interface_expression_type(
            expression,
            expected_type,
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            self.module_implementations,
            self.module_imports,
            self.interface_members,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) {
            match result {
                Ok(actual_type) if &actual_type != expected_type => {
                    self.diagnostics.push(type_mismatch_diagnostic(
                        context_name,
                        expected_type,
                        &actual_type,
                        span,
                    ));
                }
                Ok(_) => {}
                Err(error) => self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Cannot type-check value for '{}': {}",
                        context_name,
                        error.message()
                    ),
                )),
            }
            return;
        }

        match infer_expression_type(
            expression,
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            self.interface_members,
            context.current_module.as_deref(),
            context.current_flow_path.as_deref(),
        ) {
            Ok(actual_type) if &actual_type != expected_type => {
                self.diagnostics.push(type_mismatch_diagnostic(
                    context_name,
                    expected_type,
                    &actual_type,
                    span,
                ));
            }
            Ok(_) => {}
            Err(error)
                if expected_type.primitive_type().is_some()
                    || expected_type.dict_key_value_types().is_some()
                    || type_name_contains_enum(
                        expected_type,
                        self.enum_types,
                        context.current_module.as_deref(),
                    ) =>
            {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Cannot type-check value for '{}': {}",
                        context_name,
                        error.message()
                    ),
                ));
            }
            Err(_) => {}
        }
    }

    fn mark_expected_expression(&mut self, expression: &Expression) {
        self.expected_expression_ids
            .insert(expression as *const Expression as usize);
    }

    fn type_name_contains_dict(&self, type_name: &TypeName, current_module: Option<&str>) -> bool {
        self.type_name_contains_dict_with_seen(type_name, current_module, &mut BTreeSet::new())
    }

    fn type_name_contains_dict_with_seen(
        &self,
        type_name: &TypeName,
        current_module: Option<&str>,
        seen_structs: &mut BTreeSet<String>,
    ) -> bool {
        match type_name {
            TypeName::Dict { .. } => true,
            TypeName::Array(element_type) => {
                self.type_name_contains_dict_with_seen(element_type, current_module, seen_structs)
            }
            TypeName::Struct(name) => {
                let key = scoped_struct_key(current_module, name);
                self.struct_type_contains_dict(&key, seen_structs)
            }
            TypeName::QualifiedStruct(name) => {
                self.struct_type_contains_dict(name.as_str(), seen_structs)
            }
            TypeName::Primitive(_) | TypeName::Interface { .. } | TypeName::Void => false,
        }
    }

    fn struct_type_contains_dict(
        &self,
        struct_key: &str,
        seen_structs: &mut BTreeSet<String>,
    ) -> bool {
        if !seen_structs.insert(struct_key.to_string()) {
            return false;
        }

        let Some(symbol) = self.struct_types.get(struct_key) else {
            return false;
        };
        let field_module = struct_key_module(struct_key);
        symbol.fields().values().any(|field_type| {
            self.type_name_contains_dict_with_seen(field_type, field_module, seen_structs)
        })
    }
}

impl ParsedVisitor for DictLiteralChecker<'_> {
    fn visit_object(&mut self, object: &Object, context: &VisitContext) {
        match object {
            Object::ConstantDeclaration(declaration) => self.check_constant(declaration, context),
            Object::Divert(divert) => self.check_divert_arguments(divert, context),
            Object::TunnelOnwards(tunnel_onwards) => {
                self.check_tunnel_onwards_arguments(tunnel_onwards, context);
            }
            Object::VariableAssignment(assignment) => self.check_assignment(assignment, context),
            _ => {}
        }
    }

    fn visit_expression(&mut self, expression: &Expression, context: &VisitContext) {
        match expression {
            Expression::FunctionCall { name, args } => self.check_function_call_arguments(
                name,
                args,
                &SourceSpan::new(None, 1, 1),
                context,
            ),
            Expression::QualifiedFunctionCall { name, args } => self.check_function_call_arguments(
                name.as_str(),
                args,
                &SourceSpan::new(None, 1, 1),
                context,
            ),
            Expression::DynamicInterfaceFunctionCall {
                target,
                member,
                args,
            } => self.check_dynamic_interface_function_arguments(
                target,
                member,
                args,
                &SourceSpan::new(None, 1, 1),
                context,
            ),
            Expression::DictLiteral(entries)
                if !self
                    .expected_expression_ids
                    .contains(&(expression as *const Expression as usize)) =>
            {
                let span = SourceSpan::new(None, 1, 1);
                self.check_dict_literal_key_consistency(entries, "Dict literal", &span);
                self.diagnostics.push(Diagnostic::error(
                    span,
                    "Dict literal requires an expected Dict type",
                ));
            }
            _ => {}
        }
    }
}

fn expression_contains_dict_literal(expression: &Expression) -> bool {
    match expression {
        Expression::DictLiteral(_) => true,
        Expression::StringContent(content) => content
            .objects()
            .iter()
            .any(|object| matches!(object, Object::Expression(expression) | Object::LogicLine(expression) if expression_contains_dict_literal(expression))),
        Expression::FunctionCall { args, .. } | Expression::QualifiedFunctionCall { args, .. } => {
            args.iter().any(expression_contains_dict_literal)
        }
        Expression::DynamicInterfaceAccess { target, .. } => {
            expression_contains_dict_literal(target)
        }
        Expression::DynamicInterfaceFunctionCall { target, args, .. } => {
            expression_contains_dict_literal(target) || args.iter().any(expression_contains_dict_literal)
        }
        Expression::ArrayLiteral(elements) | Expression::MultipleCondition(elements) => {
            elements.iter().any(expression_contains_dict_literal)
        }
        Expression::StructLiteral { fields, .. } => fields
            .iter()
            .any(|field| expression_contains_dict_literal(field.expression())),
        Expression::FieldAccess { base, .. } => expression_contains_dict_literal(base),
        Expression::IndexAccess { base, index } => {
            expression_contains_dict_literal(base) || expression_contains_dict_literal(index)
        }
        Expression::Binary { left, right, .. } => {
            expression_contains_dict_literal(left) || expression_contains_dict_literal(right)
        }
        Expression::Unary { expression, .. } => expression_contains_dict_literal(expression),
        Expression::String(_)
        | Expression::NumberInt(_)
        | Expression::NumberFloat(_)
        | Expression::NumberBool(_)
        | Expression::DivertTarget(_)
        | Expression::VariableReference(_)
        | Expression::QualifiedReference(_) => false,
    }
}

fn scoped_struct_key(current_module: Option<&str>, name: &str) -> String {
    if name.contains("::") {
        name.to_string()
    } else {
        current_module
            .map(|module| format!("{module}::{name}"))
            .unwrap_or_else(|| name.to_string())
    }
}

fn struct_key_module(struct_key: &str) -> Option<&str> {
    struct_key.split_once("::").map(|(module, _)| module)
}

fn dict_key_matches(expected: DictKeyType, actual: &DictLiteralKey) -> bool {
    matches!(
        (expected, actual),
        (DictKeyType::String, DictLiteralKey::String(_))
            | (DictKeyType::Int, DictLiteralKey::Int(_))
    )
}

fn dict_literal_key_type_name(key: &DictLiteralKey) -> &'static str {
    match key {
        DictLiteralKey::String(_) => "string",
        DictLiteralKey::Int(_) => "int",
    }
}

fn dict_literal_key_display(key: &DictLiteralKey) -> String {
    match key {
        DictLiteralKey::String(value) => format!("\"{}\"", escape_snapshot_text(value)),
        DictLiteralKey::Int(value) => value.to_string(),
    }
}

fn dict_entry_context(context_name: &str, key: &DictLiteralKey) -> String {
    match key {
        DictLiteralKey::String(value) => {
            format!("{context_name}[\"{}\"]", escape_snapshot_text(value))
        }
        DictLiteralKey::Int(value) => format!("{context_name}[{value}]"),
    }
}

fn type_mismatch_diagnostic(
    context_name: &str,
    expected_type: &TypeName,
    actual_type: &TypeName,
    span: &SourceSpan,
) -> Diagnostic {
    Diagnostic::error(
        span.clone(),
        format!(
            "Value for '{}' has type {} but expected {}",
            context_name,
            actual_type.display_name(),
            expected_type.display_name()
        ),
    )
}

#[cfg(test)]
mod tests {
    use crate::{analysis::test_support::assert_single_diagnostic, diagnostic::DiagnosticSeverity};

    use super::{super::test_support::parse_story, *};

    #[test]
    fn accepts_dict_literals_with_nested_value_types() {
        let story = parse_story(
            "=== interface IRoute ===\n\
             == target ==\n\
             === module game ===\n\
             FROM left\n\
             ENUM State { Idle Busy }\n\
             STRUCT Player {\n\
             hp: int\n\
             }\n\
             STRUCT Bag {\n\
             scores: Dict<string, int>\n\
             }\n\
             VAR players: Dict<string, Player> = %{\"ada\": %Player{ hp: 10 }}\n\
             VAR nested: Dict<int, Dict<string, int[]>> = %{1: %{\"scores\": [1, 2]}}\n\
             VAR states: Dict<string, State> = %{\"current\": State.Idle}\n\
             VAR routes: Dict<string, interface<IRoute>> = %{\"next\": left}\n\
             VAR empty: Dict<string, int> = %{}\n\
             VAR emptyBag: Bag = %Bag{ scores: %{} }\n\
             == main ==\n\
             -> END\n\
             === module left implements IRoute ===\n\
             == target ==\n\
             -> END",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn reports_wrong_dict_literal_key_type() {
        let story = parse_story(
            "VAR scores: Dict<int, int> = %{\"ada\": 1}\n\
             -> DONE",
        );

        let diagnostics = dict_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Dict literal key for 'scores' has type string but expected int",
        );
    }

    #[test]
    fn reports_wrong_dict_literal_value_type() {
        let story = parse_story(
            "VAR scores: Dict<string, int> = %{\"ada\": \"high\"}\n\
             -> DONE",
        );

        let diagnostics = dict_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Value for 'scores[\"ada\"]' has type string but expected int",
        );
    }

    #[test]
    fn reports_wrong_nested_dict_literal_value_type() {
        let story = parse_story(
            "VAR table: Dict<string, Dict<int, string>> = %{\"row\": %{1: 7}}\n\
             -> DONE",
        );

        let diagnostics = dict_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Value for 'table[\"row\"][1]' has type int but expected string",
        );
    }

    #[test]
    fn reports_duplicate_string_dict_literal_keys() {
        let story = parse_story(
            "VAR scores: Dict<string, int> = %{\"ada\": 10, \"ada\": 12}\n\
             -> DONE",
        );

        let diagnostics = dict_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Duplicate key \"ada\" in Dict literal for 'scores'",
        );
    }

    #[test]
    fn reports_duplicate_int_dict_literal_keys() {
        let story = parse_story(
            "VAR names: Dict<int, string> = %{1: \"one\", 1: \"uno\"}\n\
             -> DONE",
        );

        let diagnostics = dict_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Duplicate key 1 in Dict literal for 'names'",
        );
    }

    #[test]
    fn reports_duplicate_nested_dict_literal_keys_with_context() {
        let story = parse_story(
            "VAR table: Dict<string, Dict<int, string>> = %{\"row\": %{1: \"one\", 1: \"uno\"}}\n\
             -> DONE",
        );

        let diagnostics = dict_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Duplicate key 1 in Dict literal for 'table[\"row\"]'",
        );
    }

    #[test]
    fn rejects_dict_literal_without_expected_type() {
        let story = parse_story(
            "~ %{\"ada\": 1}\n\
             -> DONE",
        );

        let diagnostics = dict_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Dict literal requires an expected Dict type",
        );
    }

    #[test]
    fn checks_dict_fields_inside_struct_literals() {
        let story = parse_story(
            "STRUCT Bag {\n\
             scores: Dict<string, int>\n\
             }\n\
             VAR bag: Bag = %Bag{ scores: %Bag{ wrong: 1 } }\n\
             -> DONE",
        );

        let diagnostics = dict_literal_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Value for 'Bag.scores' is a struct literal but expected Dict<string, int>",
        );
    }
}
