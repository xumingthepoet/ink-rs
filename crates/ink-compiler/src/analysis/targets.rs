use std::collections::HashMap;

use crate::{
    diagnostic::Diagnostic,
    parsed::{
        visit::{walk_story, ParsedVisitor, VisitContext},
        DivertTarget, Expression, Flow, FlowArgument, InterfaceMemberKind,
        InterfaceMemberSignature, Object, Story, TypeName,
    },
    source::SourceSpan,
    syntax::parse_initial_expression,
};

use super::{
    context::{
        EnumTypeIndex, FlowContext, FlowSymbol, StructTypeIndex, TargetSymbolIndex,
        VariableScopeIndex,
    },
    enums::is_enum_member_reference,
    expression_types::{infer_expression_type, typed_builtin_return_type},
    indexes::AnalysisIndexes,
    interface_values::{
        infer_expected_interface_expression_type, InterfaceModuleLiteralUses,
        ModuleImplementationIndex,
    },
    interfaces::InterfaceMemberIndex,
    modules::ModuleImportIndex,
    span::object_span,
    target_symbols::{is_cross_module_stitch_target, resolve_target_symbol},
    type_names::qualify_type_name_for_module,
};

#[cfg(test)]
use super::modules::ModuleAnalysis;

#[cfg(test)]
pub(super) fn call_target_diagnostics(story: &Story) -> Vec<Diagnostic> {
    let module_analysis = ModuleAnalysis::build(story);
    let indexes = AnalysisIndexes::build(story, &module_analysis);
    call_target_diagnostics_with_indexes(story, &indexes)
}

pub(super) fn call_target_diagnostics_with_indexes(
    story: &Story,
    indexes: &AnalysisIndexes<'_>,
) -> Vec<Diagnostic> {
    let mut checker = CallTargetChecker::new(
        &indexes.target_symbols,
        &indexes.variable_scopes,
        &indexes.struct_types,
        &indexes.enum_types,
        &indexes.interface_members,
        &indexes.module_implementations,
        indexes.module_imports,
        &indexes.interface_module_literal_uses,
    );
    walk_story(story, &mut checker);
    checker.diagnostics
}

struct CallTargetChecker<'a> {
    target_symbols: &'a TargetSymbolIndex,
    variable_scopes: &'a VariableScopeIndex,
    struct_types: &'a StructTypeIndex,
    enum_types: &'a EnumTypeIndex,
    interface_members: &'a InterfaceMemberIndex,
    module_implementations: &'a ModuleImplementationIndex,
    module_imports: &'a ModuleImportIndex,
    interface_module_literal_uses: &'a InterfaceModuleLiteralUses,
    diagnostics: Vec<Diagnostic>,
    flow_contexts_by_path: HashMap<String, FlowContext>,
}

impl<'a> CallTargetChecker<'a> {
    fn new(
        target_symbols: &'a TargetSymbolIndex,
        variable_scopes: &'a VariableScopeIndex,
        struct_types: &'a StructTypeIndex,
        enum_types: &'a EnumTypeIndex,
        interface_members: &'a InterfaceMemberIndex,
        module_implementations: &'a ModuleImplementationIndex,
        module_imports: &'a ModuleImportIndex,
        interface_module_literal_uses: &'a InterfaceModuleLiteralUses,
    ) -> Self {
        Self {
            target_symbols,
            variable_scopes,
            struct_types,
            enum_types,
            interface_members,
            module_implementations,
            module_imports,
            interface_module_literal_uses,
            diagnostics: Vec::new(),
            flow_contexts_by_path: HashMap::new(),
        }
    }

    fn current_flow_path<'context>(
        &self,
        context: &'context VisitContext,
    ) -> Option<&'context str> {
        context.current_flow_path.as_deref()
    }

    fn current_module<'context>(&self, context: &'context VisitContext) -> Option<&'context str> {
        context.current_module.as_deref()
    }

    fn current_flow_context(&self, context: &VisitContext) -> Option<&FlowContext> {
        self.current_flow_path(context).and_then(|flow_path| {
            self.flow_contexts_by_path
                .get(&scoped_context_key(self.current_module(context), flow_path))
        })
    }

    fn current_flow_arguments(&self, context: &VisitContext) -> Option<&[FlowArgument]> {
        self.current_flow_context(context)
            .map(FlowContext::arguments)
    }

    fn current_flow_is_function(&self, context: &VisitContext) -> bool {
        self.current_flow_context(context)
            .is_some_and(FlowContext::is_function)
    }

    fn check_plain_divert_target(
        &mut self,
        divert: &crate::parsed::Divert,
        context: &VisitContext,
    ) {
        let Some(target) = static_divert_target_name(divert.target()) else {
            return;
        };

        if self.check_plain_divert_target_expression(divert, context) {
            return;
        }

        let span = divert.span();
        if self.check_cross_module_stitch_target(target, span, context) {
            return;
        }

        let current_flow_path = self.current_flow_path(context);
        if let Some(symbol) = resolve_target_symbol(
            target,
            self.current_module(context),
            current_flow_path,
            self.target_symbols,
        ) {
            if symbol.is_function() {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "{target} can't be diverted to. It can only be called as a function since it's been marked as such: '{target}(...)'"
                    ),
                ));
            }
        } else if let Some((name, is_divert_target)) =
            resolve_current_flow_argument(target, self.current_flow_arguments(context))
                .map(|argument| (argument.name().to_string(), argument.is_divert_target()))
        {
            if !is_divert_target {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Since '{name}' is used as a variable divert target, it should be marked as: -> {name}"
                    ),
                ));
            }
        } else {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!("target not found: '{target}'"),
            ));
        }
    }

    fn check_plain_divert_target_expression(
        &mut self,
        divert: &crate::parsed::Divert,
        context: &VisitContext,
    ) -> bool {
        let DivertTarget::Path(target) = divert.target() else {
            return false;
        };
        let current_flow_path = self.current_flow_path(context);

        if divert.has_argument_list()
            && resolve_target_symbol(
                target,
                self.current_module(context),
                current_flow_path,
                self.target_symbols,
            )
            .is_some_and(|symbol| {
                symbol.is_function() && symbol.return_type() == &TypeName::divert_target()
            })
        {
            let args = divert
                .arguments()
                .iter()
                .map(Expression::to_source_string)
                .collect::<Vec<_>>()
                .join(", ");
            self.diagnostics.push(Diagnostic::error(
                divert.span().clone(),
                format!(
                    "Static divert targets must be knot or stitch paths. Use `-> {{{target}({args})}}` for a divert-target expression."
                ),
            ));
            return true;
        }

        let Some(expression) = parse_initial_expression(target) else {
            return false;
        };

        if let Some(root) = expression_root_variable_name(&expression) {
            if self.variable_scopes.contains_visible_variable(
                root,
                self.current_module(context),
                current_flow_path,
            ) {
                self.diagnostics.push(Diagnostic::error(
                    divert.span().clone(),
                    format!(
                        "Static divert targets must be knot or stitch paths. Use `-> {{{target}}}` for a divert-target expression."
                    ),
                ));
                return true;
            }
        }

        false
    }

    fn check_dynamic_divert_target(
        &mut self,
        expression: &Expression,
        arguments: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if let Expression::DynamicInterfaceAccess { target, member } = expression {
            self.check_dynamic_interface_divert_target(target, member, arguments, span, context);
            return;
        }

        self.check_expression(expression, span, context);
        match infer_expression_type(
            expression,
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            self.interface_members,
            self.current_module(context),
            self.current_flow_path(context),
        ) {
            Ok(actual_type) if actual_type == TypeName::divert_target() => {}
            Ok(actual_type) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Dynamic divert target has type {} but expected ->",
                    actual_type.display_name()
                ),
            )),
            Err(error) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Cannot type-check dynamic divert target: {}",
                    error.message()
                ),
            )),
        }
    }

    fn check_dynamic_interface_divert_target(
        &mut self,
        target: &Expression,
        member: &str,
        arguments: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        self.check_expression(target, span, context);

        match self.dynamic_interface_member_signature(target, member, span, context) {
            Some(signature) => {
                self.check_dynamic_interface_member_arguments(
                    "target", member, arguments, &signature, span, context,
                );
            }
            None => {
                for argument in arguments {
                    self.check_expression(argument, span, context);
                }
            }
        }
    }

    fn dynamic_interface_member_signature(
        &mut self,
        target: &Expression,
        member: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) -> Option<InterfaceMemberSignature> {
        let target_type = match infer_expression_type(
            target,
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            self.interface_members,
            self.current_module(context),
            self.current_flow_path(context),
        ) {
            Ok(target_type) => target_type,
            Err(error) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Cannot type-check dynamic interface target '{member}': {}",
                        error.message()
                    ),
                ));
                return None;
            }
        };

        let Some(interface_name) = target_type.as_interface_name() else {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Dynamic interface target '{member}' has base type {} but expected interface",
                    target_type.display_name()
                ),
            ));
            return None;
        };

        let Some(signature) = self.interface_members.member(interface_name, member) else {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!("Interface '{interface_name}' does not declare member '{member}'"),
            ));
            return None;
        };

        if signature.kind() != &InterfaceMemberKind::Knot {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Interface '{interface_name}' member '{member}' is a function but dynamic target access requires a knot"
                ),
            ));
            return None;
        }

        Some(signature.clone())
    }

    fn check_dynamic_interface_function_call(
        &mut self,
        target: &Expression,
        member: &str,
        arguments: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        self.check_expression(target, span, context);

        match self.dynamic_interface_function_signature(target, member, span, context) {
            Some(signature) => {
                self.check_dynamic_interface_member_arguments(
                    "function", member, arguments, &signature, span, context,
                );
            }
            None => {
                for argument in arguments {
                    self.check_expression(argument, span, context);
                }
            }
        }
    }

    fn dynamic_interface_function_signature(
        &mut self,
        target: &Expression,
        member: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) -> Option<InterfaceMemberSignature> {
        let target_type = match infer_expression_type(
            target,
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            self.interface_members,
            self.current_module(context),
            self.current_flow_path(context),
        ) {
            Ok(target_type) => target_type,
            Err(error) => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Cannot type-check dynamic interface function '{member}': {}",
                        error.message()
                    ),
                ));
                return None;
            }
        };

        let Some(interface_name) = target_type.as_interface_name() else {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Dynamic interface function '{member}' has base type {} but expected interface",
                    target_type.display_name()
                ),
            ));
            return None;
        };

        let Some(signature) = self.interface_members.member(interface_name, member) else {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!("Interface '{interface_name}' does not declare member '{member}'"),
            ));
            return None;
        };

        if signature.kind() != &InterfaceMemberKind::Function {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Interface '{interface_name}' member '{member}' is a knot but dynamic function call requires a function"
                ),
            ));
            return None;
        }

        Some(signature.clone())
    }

    fn check_dynamic_interface_member_arguments(
        &mut self,
        member_kind: &str,
        member: &str,
        arguments: &[Expression],
        signature: &InterfaceMemberSignature,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let parameters = signature.arguments();
        if arguments.len() != parameters.len() {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Dynamic interface {member_kind} '{member}' expects {} arguments but got {}",
                    parameters.len(),
                    arguments.len()
                ),
            ));
            return;
        }

        for (argument, parameter) in arguments.iter().zip(parameters) {
            let Some(expected_type) = parameter.declared_type() else {
                self.check_expression(argument, span, context);
                continue;
            };

            if is_composite_literal(argument) {
                self.check_expression(argument, span, context);
                continue;
            }

            let argument_type = infer_expected_interface_expression_type(
                argument,
                expected_type,
                self.variable_scopes,
                self.struct_types,
                self.enum_types,
                self.target_symbols,
                self.module_implementations,
                self.module_imports,
                self.interface_members,
                self.current_module(context),
                self.current_flow_path(context),
            )
            .unwrap_or_else(|| {
                infer_expression_type(
                    argument,
                    self.variable_scopes,
                    self.struct_types,
                    self.enum_types,
                    self.target_symbols,
                    self.interface_members,
                    self.current_module(context),
                    self.current_flow_path(context),
                )
            });

            match argument_type {
                Ok(actual_type) if actual_type != *expected_type => {
                    self.diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!(
                            "Argument '{}' for dynamic interface {member_kind} '{member}' has type {} but expected {}",
                            parameter.name(),
                            actual_type.display_name(),
                            expected_type.display_name()
                        ),
                    ));
                    self.check_expression(argument, span, context);
                }
                Ok(_) => {
                    if !is_interface_module_literal_argument(argument, expected_type) {
                        self.check_expression(argument, span, context);
                    }
                }
                Err(error) => self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Cannot type-check argument '{}' for dynamic interface {member_kind} '{member}': {}",
                        parameter.name(),
                        error.message()
                    ),
                )),
            }
        }
    }

    fn check_expression(
        &mut self,
        expression: &Expression,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        match expression {
            Expression::FunctionCall { name, args } => {
                self.check_function_call(name, args, span, context);
            }
            Expression::QualifiedFunctionCall { name, args } => {
                self.check_function_call(name.as_str(), args, span, context);
            }
            Expression::DynamicInterfaceAccess { target, member } => {
                let dynamic_access = Expression::DynamicInterfaceAccess {
                    target: target.clone(),
                    member: member.clone(),
                };
                if let Err(error) = infer_expression_type(
                    &dynamic_access,
                    self.variable_scopes,
                    self.struct_types,
                    self.enum_types,
                    self.target_symbols,
                    self.interface_members,
                    self.current_module(context),
                    self.current_flow_path(context),
                ) {
                    self.diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!(
                            "Cannot type-check dynamic interface access: {}",
                            error.message()
                        ),
                    ));
                }
                self.check_expression(target, span, context);
            }
            Expression::DynamicInterfaceFunctionCall {
                target,
                member,
                args,
            } => {
                self.check_dynamic_interface_function_call(target, member, args, span, context);
            }
            Expression::ArrayLiteral(elements) => {
                for element in elements {
                    self.check_expression(element, span, context);
                }
            }
            Expression::StructLiteral { fields, .. } => {
                for field in fields {
                    self.check_expression(field.expression(), span, context);
                }
            }
            Expression::DictLiteral(entries) => {
                for entry in entries {
                    self.check_expression(entry.value(), span, context);
                }
            }
            Expression::FieldAccess { base, .. }
                if !is_enum_member_reference(
                    expression,
                    self.enum_types,
                    self.current_module(context),
                ) =>
            {
                self.check_expression(base, span, context);
            }
            Expression::FieldAccess { .. } => {}
            Expression::IndexAccess { base, index } => {
                self.check_expression(base, span, context);
                self.check_expression(index, span, context);
            }
            Expression::Binary { left, right, .. } => {
                self.check_expression(left, span, context);
                self.check_expression(right, span, context);
            }
            Expression::Unary { expression, .. } => {
                self.check_expression(expression, span, context);
            }
            Expression::MultipleCondition(expressions) => {
                for expression in expressions {
                    self.check_expression(expression, span, context);
                }
            }
            Expression::DivertTarget(target) => {
                self.check_divert_target_value(target, span, context);
            }
            Expression::VariableReference(name) => {
                self.check_variable_reference(expression, name, span, context);
            }
            Expression::QualifiedReference(name) => {
                self.check_variable_reference(expression, name.as_str(), span, context);
            }
            Expression::StringContent(_)
            | Expression::String(_)
            | Expression::NumberInt(_)
            | Expression::NumberFloat(_)
            | Expression::NumberBool(_) => {}
        }
    }

    fn check_function_call(
        &mut self,
        name: &str,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if self.check_cross_module_stitch_target(name, span, context) {
            for arg in args {
                self.check_expression(arg, span, context);
            }
            return;
        }

        if typed_builtin_return_type(name).is_some() {
            self.check_typed_builtin_call(name, args, span, context);
            for arg in args {
                self.check_expression(arg, span, context);
            }
            return;
        }

        if is_runtime_builtin_function(name) {
            for arg in args {
                self.check_expression(arg, span, context);
            }
            return;
        }

        let symbol = resolve_target_symbol(
            name,
            self.current_module(context),
            self.current_flow_path(context),
            self.target_symbols,
        )
        .cloned();

        if let Some(symbol) = symbol {
            if !symbol.is_function() {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "{name} hasn't been marked as a function, but it's being called as one. Do you need to declare the knot as '== function {name} =='?"
                    ),
                ));
                for arg in args {
                    self.check_expression(arg, span, context);
                }
            } else {
                self.check_function_call_signature(name, args, &symbol, span, context);
            }
        } else {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!("Function '{name}' is not declared"),
            ));
            for arg in args {
                self.check_expression(arg, span, context);
            }
        }
    }

    fn check_typed_builtin_call(
        &mut self,
        name: &str,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        match name {
            "ARRAY_REMOVE" => self.check_array_remove_call(args, span, context),
            "LEN" => self.check_len_call(args, span, context),
            _ => {}
        }
    }

    fn check_array_remove_call(
        &mut self,
        args: &[Expression],
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if args.len() != 2 {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Builtin 'ARRAY_REMOVE' expects 2 arguments but got {}",
                    args.len()
                ),
            ));
            return;
        }

        if !is_mutable_lvalue(&args[0]) {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                "First argument for builtin 'ARRAY_REMOVE' must be a mutable lvalue",
            ));
        }

        match infer_expression_type(
            &args[0],
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            self.interface_members,
            self.current_module(context),
            self.current_flow_path(context),
        ) {
            Ok(argument_type) if argument_type.array_element_type().is_none() => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "First argument for builtin 'ARRAY_REMOVE' has type {} but expected array",
                        argument_type.display_name()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Cannot type-check first argument for builtin 'ARRAY_REMOVE': {}",
                    error.message()
                ),
            )),
        }

        match infer_expression_type(
            &args[1],
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            self.interface_members,
            self.current_module(context),
            self.current_flow_path(context),
        ) {
            Ok(argument_type) if argument_type != TypeName::int() => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Second argument for builtin 'ARRAY_REMOVE' has type {} but expected int",
                        argument_type.display_name()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Cannot type-check second argument for builtin 'ARRAY_REMOVE': {}",
                    error.message()
                ),
            )),
        }
    }

    fn check_len_call(&mut self, args: &[Expression], span: &SourceSpan, context: &VisitContext) {
        if args.len() != 1 {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!("Builtin 'LEN' expects 1 argument but got {}", args.len()),
            ));
            return;
        }

        match infer_expression_type(
            &args[0],
            self.variable_scopes,
            self.struct_types,
            self.enum_types,
            self.target_symbols,
            self.interface_members,
            self.current_module(context),
            self.current_flow_path(context),
        ) {
            Ok(argument_type) if argument_type.array_element_type().is_none() => {
                self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Argument for builtin 'LEN' has type {} but expected array",
                        argument_type.display_name()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) => self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Cannot type-check argument for builtin 'LEN': {}",
                    error.message()
                ),
            )),
        }
    }

    fn check_function_call_signature(
        &mut self,
        name: &str,
        args: &[Expression],
        symbol: &FlowSymbol,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        let parameters = symbol.arguments();
        let qualified_module = name.split_once("::").map(|(module, _)| module);
        if args.len() != parameters.len() {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Function '{name}' expects {} arguments but got {}",
                    parameters.len(),
                    args.len()
                ),
            ));
            for arg in args {
                self.check_expression(arg, span, context);
            }
            return;
        }

        for (argument, parameter) in args.iter().zip(parameters) {
            let Some(expected_type) = parameter.declared_type() else {
                self.check_expression(argument, span, context);
                continue;
            };
            let expected_type = qualified_module
                .map(|module| qualify_type_name_for_module(expected_type, module))
                .unwrap_or_else(|| expected_type.clone());
            if is_composite_literal(argument) {
                self.check_expression(argument, span, context);
                continue;
            }
            let argument_type = infer_expected_interface_expression_type(
                argument,
                &expected_type,
                self.variable_scopes,
                self.struct_types,
                self.enum_types,
                self.target_symbols,
                self.module_implementations,
                self.module_imports,
                self.interface_members,
                self.current_module(context),
                self.current_flow_path(context),
            )
            .unwrap_or_else(|| {
                infer_expression_type(
                    argument,
                    self.variable_scopes,
                    self.struct_types,
                    self.enum_types,
                    self.target_symbols,
                    self.interface_members,
                    self.current_module(context),
                    self.current_flow_path(context),
                )
            });

            match argument_type {
                Ok(actual_type) if actual_type != expected_type => {
                    self.diagnostics.push(Diagnostic::error(
                        span.clone(),
                        format!(
                            "Argument '{}' for function '{name}' has type {} but expected {}",
                            parameter.name(),
                            actual_type.display_name(),
                            expected_type.display_name()
                        ),
                    ));
                    self.check_expression(argument, span, context);
                }
                Ok(_) => {
                    if !is_interface_module_literal_argument(argument, &expected_type) {
                        self.check_expression(argument, span, context);
                    }
                }
                Err(error) => self.diagnostics.push(Diagnostic::error(
                    span.clone(),
                    format!(
                        "Cannot type-check argument '{}' for function '{name}': {}",
                        parameter.name(),
                        error.message()
                    ),
                )),
            }
        }
    }

    fn check_variable_reference(
        &mut self,
        expression: &Expression,
        name: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if self
            .interface_module_literal_uses
            .contains_expression(expression)
        {
            return;
        }

        let current_flow_path = self.current_flow_path(context);
        if name.contains("::")
            && self
                .variable_scopes
                .qualified_constant_declared_type(name)
                .or_else(|| {
                    self.variable_scopes
                        .qualified_global_variable_declared_type(name)
                })
                .is_some()
        {
            return;
        }

        if name.contains('.') {
            return;
        }

        if self.variable_scopes.contains_visible_variable(
            name,
            self.current_module(context),
            current_flow_path,
        ) {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            span.clone(),
            format!("Unresolved variable: {name}"),
        ));
    }

    fn check_divert_target_value(
        &mut self,
        target: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) {
        if self.check_cross_module_stitch_target(target, span, context) {
            return;
        }

        let variable_name = target.split('.').next().unwrap_or(target);
        if self.variable_scopes.contains_visible_variable(
            variable_name,
            self.current_module(context),
            self.current_flow_path(context),
        ) {
            self.diagnostics.push(Diagnostic::error(
                span.clone(),
                format!(
                    "Since '{variable_name}' is a variable, it shouldn't be preceded by '->' here."
                ),
            ));
        }
    }

    fn check_cross_module_stitch_target(
        &mut self,
        target: &str,
        span: &SourceSpan,
        context: &VisitContext,
    ) -> bool {
        if !is_cross_module_stitch_target(target, self.current_module(context)) {
            return false;
        }

        self.diagnostics.push(Diagnostic::error(
            span.clone(),
            format!(
                "Cross-module direct stitch access is not allowed: '{target}'. Import and reference the parent knot instead."
            ),
        ));
        true
    }
}

fn static_divert_target_name(target: &DivertTarget) -> Option<&str> {
    match target {
        DivertTarget::Path(target) => Some(target),
        DivertTarget::QualifiedPath(target) => Some(target.as_str()),
        DivertTarget::Dynamic(_) | DivertTarget::Done | DivertTarget::End | DivertTarget::Empty => {
            None
        }
    }
}

fn is_mutable_lvalue(expression: &Expression) -> bool {
    match expression {
        Expression::VariableReference(_) => true,
        Expression::QualifiedReference(_) => true,
        Expression::FieldAccess { base, .. } | Expression::IndexAccess { base, .. } => {
            is_mutable_lvalue(base)
        }
        Expression::DynamicInterfaceAccess { .. }
        | Expression::DynamicInterfaceFunctionCall { .. } => false,
        _ => false,
    }
}

fn is_composite_literal(expression: &Expression) -> bool {
    matches!(
        expression,
        Expression::ArrayLiteral(_) | Expression::StructLiteral { .. } | Expression::DictLiteral(_)
    )
}

fn is_runtime_builtin_function(name: &str) -> bool {
    matches!(
        name,
        "RANDOM" | "SEED_RANDOM" | "MIN" | "MAX" | "POW" | "FLOOR" | "CEILING" | "INT" | "FLOAT"
    )
}

fn expression_root_variable_name(expression: &Expression) -> Option<&str> {
    match expression {
        Expression::VariableReference(name) => Some(name),
        Expression::QualifiedReference(name) => Some(name.as_str()),
        Expression::FieldAccess { base, .. } | Expression::IndexAccess { base, .. } => {
            expression_root_variable_name(base)
        }
        Expression::DynamicInterfaceAccess { .. }
        | Expression::DynamicInterfaceFunctionCall { .. } => None,
        _ => None,
    }
}

fn scoped_context_key(module: Option<&str>, flow_path: &str) -> String {
    module
        .map(|module| format!("{module}::{flow_path}"))
        .unwrap_or_else(|| flow_path.to_string())
}

impl ParsedVisitor for CallTargetChecker<'_> {
    fn visit_flow(&mut self, flow: &Flow, context: &VisitContext) {
        let Some(flow_path) = self.current_flow_path(context) else {
            return;
        };
        self.flow_contexts_by_path
            .entry(scoped_context_key(
                context.current_module.as_deref(),
                flow_path,
            ))
            .or_insert_with(|| FlowContext::new(flow.arguments().to_vec(), flow.is_function()));
    }

    fn visit_object(&mut self, object: &Object, context: &VisitContext) {
        match object {
            Object::Divert(divert) => {
                let arguments_checked_by_dynamic_interface_target = matches!(
                    divert.target(),
                    DivertTarget::Dynamic(Expression::DynamicInterfaceAccess { .. })
                );
                match divert.target() {
                    DivertTarget::Empty => self.diagnostics.push(Diagnostic::error(
                        divert.span().clone(),
                        "Empty diverts (->) are only valid on choices",
                    )),
                    DivertTarget::Dynamic(expression) => {
                        self.check_dynamic_divert_target(
                            expression,
                            divert.arguments(),
                            divert.span(),
                            context,
                        );
                    }
                    DivertTarget::Path(_) | DivertTarget::QualifiedPath(_)
                        if !self.current_flow_is_function(context) =>
                    {
                        self.check_plain_divert_target(divert, context);
                    }
                    DivertTarget::Path(_)
                    | DivertTarget::QualifiedPath(_)
                    | DivertTarget::Done
                    | DivertTarget::End => {}
                }
                if !arguments_checked_by_dynamic_interface_target {
                    for argument in divert.arguments() {
                        self.check_expression(argument, divert.span(), context);
                    }
                }
            }
            Object::ConstantDeclaration(declaration) => {
                self.check_expression(declaration.expression(), declaration.span(), context);
            }
            Object::Expression(expression) | Object::LogicLine(expression) => {
                self.check_expression(expression, &object_span(object), context);
            }
            Object::VariableAssignment(assignment) => {
                if let Some(expression) = assignment.expression() {
                    self.check_expression(expression, assignment.span(), context);
                }
            }
            Object::IncDec(inc_dec) => {
                self.check_expression(inc_dec.expression(), inc_dec.span(), context);
            }
            Object::Return(ret) => {
                if let Some(expression) = ret.returned_expression() {
                    self.check_expression(expression, ret.span(), context);
                }
            }
            Object::Conditional(conditional) => {
                let span = object_span(object);
                if let Some(condition) = conditional.initial_condition() {
                    self.check_expression(condition, &span, context);
                }
                for branch in conditional.branches() {
                    if let Some(condition) = branch.own_condition() {
                        self.check_expression(condition, &span, context);
                    }
                }
            }
            Object::Choice(choice) => {
                if let Some(condition) = choice.condition() {
                    self.check_expression(condition, choice.span(), context);
                }
            }
            Object::TunnelOnwards(tunnel_onwards) => {
                let arguments_checked_by_dynamic_interface_target = matches!(
                    tunnel_onwards.override_target(),
                    Some(DivertTarget::Dynamic(
                        Expression::DynamicInterfaceAccess { .. }
                    ))
                );
                if let Some(target) = tunnel_onwards.override_target() {
                    match target {
                        DivertTarget::Dynamic(expression) => {
                            self.check_dynamic_divert_target(
                                expression,
                                tunnel_onwards.arguments(),
                                tunnel_onwards.span(),
                                context,
                            );
                        }
                        DivertTarget::Path(_) | DivertTarget::QualifiedPath(_) => {
                            if let Some(target) = static_divert_target_name(target) {
                                self.check_cross_module_stitch_target(
                                    target,
                                    tunnel_onwards.span(),
                                    context,
                                );
                            }
                        }
                        DivertTarget::Done | DivertTarget::End | DivertTarget::Empty => {}
                    }
                }
                if !arguments_checked_by_dynamic_interface_target {
                    for argument in tunnel_onwards.arguments() {
                        self.check_expression(argument, tunnel_onwards.span(), context);
                    }
                }
            }
            Object::AuthorWarning(_)
            | Object::ContentList(_)
            | Object::EnumDeclaration(_)
            | Object::ExternalDeclaration(_)
            | Object::Gather(_)
            | Object::Glue(_)
            | Object::StructDeclaration(_)
            | Object::Tag(_)
            | Object::Text(_)
            | Object::Weave(_) => {}
        }
    }
}

fn is_interface_module_literal_argument(expression: &Expression, expected_type: &TypeName) -> bool {
    expected_type.as_interface_name().is_some()
        && matches!(expression, Expression::VariableReference(_))
}

fn resolve_current_flow_argument<'a>(
    target: &str,
    current_flow_arguments: Option<&'a [FlowArgument]>,
) -> Option<&'a FlowArgument> {
    let variable_target_name = target.split('.').next()?;
    current_flow_arguments?
        .iter()
        .find(|argument| argument.name() == variable_target_name)
}

#[cfg(test)]
mod tests {
    use crate::diagnostic::DiagnosticSeverity;

    use super::{
        super::test_support::{assert_single_diagnostic, parse_story},
        *,
    };

    #[test]
    fn reports_missing_divert_targets() {
        let story = parse_story("-> missing");
        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "target not found: 'missing'",
        );
    }

    #[test]
    fn reports_non_function_call_targets() {
        let story = parse_story("~ knot()\n== knot ==\n-> DONE");
        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "knot hasn't been marked as a function, but it's being called as one. Do you need to declare the knot as '== function knot =='?",
        );
    }

    #[test]
    fn accepts_typed_function_calls_and_return_type_use() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR source_player: Player = %Player{ hp: 10 }\n\
             VAR source_scores: int[] = [1]\n\
             VAR source_lookup: Dict<string, int> = %{\"ada\": 10}\n\
             VAR source_lookup_list: Dict<string, int>[] = [source_lookup]\n\
             VAR result: int = add(1, 2)\n\
             VAR copied_player: Player = echo_player(source_player)\n\
             VAR copied_scores: int[] = echo_scores(source_scores)\n\
             VAR copied_lookup: Dict<string, int> = echo_lookup(source_lookup)\n\
             VAR copied_lookup_list: Dict<string, int>[] = echo_lookup_list(source_lookup_list)\n\
             -> DONE\n\
             == function add(a: int, b: int) => int ==\n\
             ~ return a + b\n\
             == function echo_player(player: Player) => Player ==\n\
             ~ return player\n\
             == function echo_scores(values: int[]) => int[] ==\n\
             ~ return values\n\
             == function echo_lookup(values: Dict<string, int>) => Dict<string, int> ==\n\
             ~ return values\n\
             == function echo_lookup_list(values: Dict<string, int>[]) => Dict<string, int>[] ==\n\
             ~ return values",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_typed_external_calls_and_return_type_use() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             EXTERNAL external_score(value: int) => int\n\
             EXTERNAL describe(player: Player, scores: int[]) => string\n\
             EXTERNAL copy_scores(scores: Dict<string, int>) => Dict<string, int>\n\
             VAR score: int = 1\n\
             VAR source_player: Player = %Player{ hp: 10 }\n\
             VAR scores: int[] = [score]\n\
             VAR score_lookup: Dict<string, int> = %{\"ada\": 10}\n\
             VAR adjusted_score: int = external_score(score) + LEN(scores)\n\
             VAR description: string = describe(source_player, scores)\n\
             VAR copied_scores: Dict<string, int> = copy_scores(score_lookup)\n\
             -> DONE",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_dict_types_in_dynamic_interface_function_signatures() {
        let story = parse_story(
            "=== interface IScorer ===\n\
             == function copy_scores(scores: Dict<string, int>) => Dict<string, int> ==\n\
             === module game ===\n\
             FROM scorer\n\
             VAR route: interface<IScorer> = scorer\n\
             VAR source_scores: Dict<string, int> = %{\"ada\": 10}\n\
             VAR copied_scores: Dict<string, int> = {route}::copy_scores(source_scores)\n\
             == main ==\n\
             -> END\n\
             === module scorer implements IScorer ===\n\
             == function copy_scores(scores: Dict<string, int>) => Dict<string, int> ==\n\
             ~ return scores",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn reports_typed_external_call_argument_count_mismatch() {
        let story = parse_story(
            "EXTERNAL external_score(value: int) => int\n\
             ~ external_score()\n\
             -> DONE",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Function 'external_score' expects 1 arguments but got 0",
        );
    }

    #[test]
    fn reports_typed_external_call_argument_type_mismatch() {
        let story = parse_story(
            "EXTERNAL external_score(value: int) => int\n\
             VAR label: string = \"x\"\n\
             ~ external_score(label)\n\
             -> DONE",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Argument 'value' for function 'external_score' has type string but expected int",
        );
    }

    #[test]
    fn external_return_type_participates_in_expression_type_checks() {
        let story = parse_story(
            "EXTERNAL external_label() => string\n\
             VAR score: int = external_label()\n\
             -> DONE",
        );

        let diagnostics = super::super::run_analysis_passes(&story);

        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.severity == DiagnosticSeverity::Error
                    && diagnostic.message
                        == "Initializer for variable 'score' has type string but declared type is int"
            }),
            "{diagnostics:#?}"
        );
    }

    #[test]
    fn accepts_len_builtin_for_array_types() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR score: int = 1\n\
             VAR source_player: Player = %Player{ hp: 10 }\n\
             VAR scores: int[] = [score]\n\
             VAR players: Player[] = [source_player]\n\
             VAR nested_scores: int[][] = [scores]\n\
             VAR score_count: int = LEN(scores)\n\
             VAR player_count: int = LEN(players)\n\
             VAR row_count: int = LEN(nested_scores)\n\
             -> DONE",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn reports_len_builtin_non_array_arguments() {
        let cases = [
            (
                "VAR value: int = 1\n\
                 VAR count: int = LEN(value)\n\
                 -> DONE",
                "Argument for builtin 'LEN' has type int but expected array",
            ),
            (
                "VAR label: string = \"text\"\n\
                 VAR count: int = LEN(label)\n\
                 -> DONE",
                "Argument for builtin 'LEN' has type string but expected array",
            ),
            (
                "STRUCT Player {\n\
                 hp: int\n\
                 }\n\
                 VAR player: Player = %Player{ hp: 10 }\n\
                 VAR count: int = LEN(player)\n\
                 -> DONE",
                "Argument for builtin 'LEN' has type Player but expected array",
            ),
        ];

        for (source, expected_message) in cases {
            let story = parse_story(source);
            let diagnostics = call_target_diagnostics(&story);

            assert_single_diagnostic(&diagnostics, DiagnosticSeverity::Error, expected_message);
        }
    }

    #[test]
    fn reports_len_builtin_argument_count_mismatch() {
        let cases = [
            (
                "VAR values: int[] = [1]\n\
                 VAR count: int = LEN()\n\
                 -> DONE",
                "Builtin 'LEN' expects 1 argument but got 0",
            ),
            (
                "VAR values: int[] = [1]\n\
                 VAR count: int = LEN(values, values)\n\
                 -> DONE",
                "Builtin 'LEN' expects 1 argument but got 2",
            ),
        ];

        for (source, expected_message) in cases {
            let story = parse_story(source);
            let diagnostics = call_target_diagnostics(&story);

            assert_single_diagnostic(&diagnostics, DiagnosticSeverity::Error, expected_message);
        }
    }

    #[test]
    fn accepts_array_remove_builtin_for_array_types() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR score: int = 1\n\
             VAR source_player: Player = %Player{ hp: 10 }\n\
             VAR scores: int[] = [score]\n\
             VAR players: Player[] = [source_player]\n\
             VAR nested_scores: int[][] = [scores]\n\
             ~ ARRAY_REMOVE(scores, 0)\n\
             ~ ARRAY_REMOVE(players, 0)\n\
             ~ ARRAY_REMOVE(nested_scores, 0)\n\
             -> DONE",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn reports_array_remove_builtin_invalid_arguments() {
        let cases = [
            (
                "VAR value: int = 1\n\
                 ~ ARRAY_REMOVE(value, 0)\n\
                 -> DONE",
                "First argument for builtin 'ARRAY_REMOVE' has type int but expected array",
            ),
            (
                "VAR values: int[] = [1]\n\
                 VAR index: string = \"0\"\n\
                 ~ ARRAY_REMOVE(values, index)\n\
                 -> DONE",
                "Second argument for builtin 'ARRAY_REMOVE' has type string but expected int",
            ),
            (
                "VAR values: int[] = [1]\n\
                 ~ ARRAY_REMOVE(values)\n\
                 -> DONE",
                "Builtin 'ARRAY_REMOVE' expects 2 arguments but got 1",
            ),
            (
                "VAR values: int[] = [1]\n\
                 ~ ARRAY_REMOVE(values, 0, 1)\n\
                 -> DONE",
                "Builtin 'ARRAY_REMOVE' expects 2 arguments but got 3",
            ),
            (
                "VAR values: int[] = [1]\n\
                 ~ ARRAY_REMOVE(copy_values(), 0)\n\
                 -> DONE\n\
                 == function copy_values() => int[] ==\n\
                 ~ return values",
                "First argument for builtin 'ARRAY_REMOVE' must be a mutable lvalue",
            ),
        ];

        for (source, expected_message) in cases {
            let story = parse_story(source);
            let diagnostics = call_target_diagnostics(&story);

            assert_single_diagnostic(&diagnostics, DiagnosticSeverity::Error, expected_message);
        }
    }

    #[test]
    fn reports_function_call_argument_count_mismatch() {
        let story = parse_story(
            "~ add(1)\n\
             -> DONE\n\
             == function add(a: int, b: int) => int ==\n\
             ~ return a",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Function 'add' expects 2 arguments but got 1",
        );
    }

    #[test]
    fn reports_function_call_argument_type_mismatch() {
        let story = parse_story(
            "~ add(1, \"two\")\n\
             -> DONE\n\
             == function add(a: int, b: int) => int ==\n\
             ~ return a",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Argument 'b' for function 'add' has type string but expected int",
        );
    }

    #[test]
    fn reports_composite_function_call_argument_type_mismatch() {
        let story = parse_story(
            "STRUCT Player {\n\
             hp: int\n\
             }\n\
             VAR scores: int[] = [1]\n\
             ~ use_player(scores)\n\
             -> DONE\n\
             == function use_player(player: Player) => void ==\n\
             ~ return",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Argument 'player' for function 'use_player' has type int[] but expected Player",
        );
    }

    #[test]
    fn function_return_type_participates_in_expression_type_checks() {
        let story = parse_story(
            "VAR score: int = label()\n\
             -> DONE\n\
             == function label() => string ==\n\
             ~ return \"ok\"",
        );

        let diagnostics = super::super::run_analysis_passes(&story);

        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic.message
                    == "Initializer for variable 'score' has type string but declared type is int"
            }),
            "{diagnostics:#?}"
        );
    }

    #[test]
    fn module_function_lookup_uses_current_module_only() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             ~ helper()\n\
             -> DONE\n\
             === module items ===\n\
             == function helper() => void ==\n\
             ~ return",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Function 'helper' is not declared",
        );
    }

    #[test]
    fn module_function_lookup_accepts_same_module_functions() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             ~ helper()\n\
             -> DONE\n\
             == function helper() => void ==\n\
             ~ return\n\
             === module items ===\n\
             == function helper() => void ==\n\
             ~ return",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn qualified_module_function_calls_are_type_checked() {
        let story = parse_story(
            "=== module game ===\n\
             FROM math IMPORT add\n\
             == main ==\n\
             ~ temp total: int = math::add(1, 2)\n\
             -> END\n\
             === module math ===\n\
             == function add(left: int, right: int) => int ==\n\
             ~ return left + right",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn qualified_module_function_calls_use_declaring_module_named_types() {
        let story = parse_story(
            "=== module game ===\n\
             FROM data IMPORT State, echo, DEFAULT_STATE\n\
             == main ==\n\
             ~ temp state: data::State = data::echo(data::DEFAULT_STATE)\n\
             -> END\n\
             === module data ===\n\
             ENUM State { Idle Busy }\n\
             CONST DEFAULT_STATE: State = State.Idle\n\
             == function echo(value: State) => State ==\n\
             ~ return value",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn qualified_module_function_calls_report_argument_type_errors() {
        let story = parse_story(
            "=== module game ===\n\
             FROM math IMPORT add\n\
             == main ==\n\
             ~ temp total: int = math::add(1, \"two\")\n\
             -> END\n\
             === module math ===\n\
             == function add(left: int, right: int) => int ==\n\
             ~ return left + right",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Argument 'right' for function 'math::add' has type string but expected int",
        );
    }

    #[test]
    fn qualified_external_calls_are_type_checked() {
        let story = parse_story(
            "=== module game ===\n\
             FROM audio IMPORT play\n\
             == main ==\n\
             ~ temp code: int = audio::play(\"intro\")\n\
             -> END\n\
             === module audio ===\n\
             EXTERNAL play(name: string) => int\n\
             == helper ==\n\
             -> END",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn module_diverts_accept_same_module_knot_stitch_paths() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             -> scene.intro\n\
             == scene ==\n\
             = intro\n\
             -> END",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn module_diverts_accept_relative_stitch_shorthand() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             -> intro\n\
             = intro\n\
             -> END",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn module_diverts_accept_same_module_self_qualified_stitches() {
        let story = parse_story(
            "=== module game ===\n\
             == main ==\n\
             -> game::scene.intro\n\
             == scene ==\n\
             = intro\n\
             -> END",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn module_diverts_reject_cross_module_direct_stitch_paths() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT scene\n\
             == main ==\n\
             -> items::scene.intro\n\
             === module items ===\n\
             == scene ==\n\
             = intro\n\
             -> END",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Cross-module direct stitch access is not allowed: 'items::scene.intro'. Import and reference the parent knot instead.",
        );
    }

    #[test]
    fn module_diverts_accept_cross_module_parent_knots_for_target_resolution() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT scene\n\
             == main ==\n\
             -> items::scene\n\
             === module items ===\n\
             == scene ==\n\
             = intro\n\
             -> END",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn module_tunnel_onwards_reject_cross_module_direct_stitch_paths() {
        let story = parse_story(
            "=== module game ===\n\
             FROM items IMPORT scene\n\
             == main ==\n\
             -> tunnel ->-> items::scene.intro\n\
             == tunnel ==\n\
             ->->\n\
             === module items ===\n\
             == scene ==\n\
             = intro\n\
             -> END",
        );

        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Cross-module direct stitch access is not allowed: 'items::scene.intro'. Import and reference the parent knot instead.",
        );
    }

    #[test]
    fn accepts_dynamic_interface_divert_targets() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target(amount: int) ==\n\
             === module game ===\n\
             FROM left\n\
             STRUCT RouteState {\n\
             current: interface<IItem>\n\
             routes: interface<IItem>[]\n\
             }\n\
             VAR route: interface<IItem> = left\n\
             VAR state: RouteState = %RouteState{ current: left, routes: [left] }\n\
             VAR routes: interface<IItem>[] = [left]\n\
             == main ==\n\
             -> {{route}::target}(1)\n\
             -> {{state.current}::target}(1)\n\
             -> {{routes[0]}::target}(1)\n\
             === module left implements IItem ===\n\
             == target(amount: int) ==\n\
             -> END",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn accepts_interface_module_literals_as_dynamic_interface_target_arguments() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === interface IRouter ===\n\
             == target(next: interface<IItem>) ==\n\
             === module game ===\n\
             FROM left\n\
             FROM router\n\
             VAR route: interface<IRouter> = router\n\
             == main ==\n\
             -> {{route}::target}(left)\n\
             === module router implements IRouter ===\n\
             == target(next: interface<IItem>) ==\n\
             -> END\n\
             === module left implements IItem ===\n\
             == target ==\n\
             -> END",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn accepts_interface_module_literals_as_static_divert_arguments() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             FROM left\n\
             == main ==\n\
             -> register(left)\n\
             == register(next: interface<IItem>) ==\n\
             -> END\n\
             === module left implements IItem ===\n\
             == target ==\n\
             -> END",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_interface_module_literals_as_function_call_arguments() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             FROM left\n\
             == main ==\n\
             ~ temp route: interface<IItem> = select(left)\n\
             -> END\n\
             == function select(next: interface<IItem>) => interface<IItem> ==\n\
             ~ return next\n\
             === module left implements IItem ===\n\
             == target ==\n\
             -> END",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn accepts_interface_module_literals_as_qualified_static_arguments() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             FROM left\n\
             FROM registry IMPORT register, select\n\
             == main ==\n\
             -> registry::register(left)\n\
             ~ temp route: interface<IItem> = registry::select(left)\n\
             -> END\n\
             === module registry ===\n\
             == register(next: interface<IItem>) ==\n\
             -> END\n\
             == function select(next: interface<IItem>) => interface<IItem> ==\n\
             ~ return next\n\
             === module left implements IItem ===\n\
             == target ==\n\
             -> END",
        );

        assert_eq!(super::super::run_analysis_passes(&story), []);
    }

    #[test]
    fn visible_variables_take_precedence_over_interface_module_literals_in_arguments() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             FROM left\n\
             == main ==\n\
             ~ temp left: int = 1\n\
             ~ temp route: interface<IItem> = select(left)\n\
             -> END\n\
             == function select(next: interface<IItem>) => interface<IItem> ==\n\
             ~ return next\n\
             === module left implements IItem ===\n\
             == target ==\n\
             -> END",
        );

        assert_single_diagnostic(
            &call_target_diagnostics(&story),
            DiagnosticSeverity::Error,
            "Argument 'next' for function 'select' has type int but expected interface<IItem>",
        );
    }

    #[test]
    fn reports_missing_import_for_interface_module_literals_in_arguments() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === module game ===\n\
             == main ==\n\
             ~ temp route: interface<IItem> = select(left)\n\
             -> END\n\
             == function select(next: interface<IItem>) => interface<IItem> ==\n\
             ~ return next\n\
             === module left implements IItem ===\n\
             == target ==\n\
             -> END",
        );

        assert_single_diagnostic(
            &call_target_diagnostics(&story),
            DiagnosticSeverity::Error,
            "Cannot type-check argument 'next' for function 'select': Module literal 'left' requires a bare import in module 'game': FROM left",
        );
    }

    #[test]
    fn reports_wrong_interface_for_interface_module_literals_in_arguments() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === interface IOther ===\n\
             == target ==\n\
             === module game ===\n\
             FROM left\n\
             == main ==\n\
             ~ temp route: interface<IItem> = select(left)\n\
             -> END\n\
             == function select(next: interface<IItem>) => interface<IItem> ==\n\
             ~ return next\n\
             === module left implements IOther ===\n\
             == target ==\n\
             -> END",
        );

        assert_single_diagnostic(
            &call_target_diagnostics(&story),
            DiagnosticSeverity::Error,
            "Cannot type-check argument 'next' for function 'select': Module 'left' does not implement interface 'IItem'",
        );
    }

    #[test]
    fn accepts_dynamic_interface_function_calls() {
        let story = parse_story(
            "=== interface IItem ===\n\
             == target ==\n\
             === interface IScorer ===\n\
             == function score(item: interface<IItem>, amount: int) => int ==\n\
             === module game ===\n\
             FROM left\n\
             FROM scorer\n\
             VAR route: interface<IScorer> = scorer\n\
             == main ==\n\
             ~ temp value: int = {route}::score(left, 3)\n\
             -> END\n\
             === module scorer implements IScorer ===\n\
             == function score(item: interface<IItem>, amount: int) => int ==\n\
             ~ return amount\n\
             === module left implements IItem ===\n\
             == target ==\n\
             -> END",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn reports_invalid_dynamic_interface_function_calls() {
        let cases = [
            (
                "~ temp value: int = {label}::score(1)",
                "Dynamic interface function 'score' has base type string but expected interface",
            ),
            (
                "~ temp value: int = {route}::missing()",
                "Interface 'IItem' does not declare member 'missing'",
            ),
            (
                "~ temp value: int = {route}::target(1)",
                "Interface 'IItem' member 'target' is a knot but dynamic function call requires a function",
            ),
            (
                "~ temp value: int = {route}::score()",
                "Dynamic interface function 'score' expects 1 arguments but got 0",
            ),
            (
                "~ temp value: int = {route}::score(\"bad\")",
                "Argument 'amount' for dynamic interface function 'score' has type string but expected int",
            ),
        ];

        for (logic, expected_message) in cases {
            let story = parse_story(&format!(
                "=== interface IItem ===\n\
                 == target(amount: int) ==\n\
                 == function score(amount: int) => int ==\n\
                 === module game ===\n\
                 FROM left\n\
                 VAR route: interface<IItem> = left\n\
                 VAR label: string = \"x\"\n\
                 == main ==\n\
                 {logic}\n\
                 -> END\n\
                 === module left implements IItem ===\n\
                 == target(amount: int) ==\n\
                 -> END\n\
                 == function score(amount: int) => int ==\n\
                 ~ return amount",
            ));
            let diagnostics = call_target_diagnostics(&story);

            assert_single_diagnostic(&diagnostics, DiagnosticSeverity::Error, expected_message);
        }
    }

    #[test]
    fn reports_invalid_dynamic_interface_divert_targets() {
        let cases = [
            (
                "-> {{label}::target}",
                "Dynamic interface target 'target' has base type string but expected interface",
            ),
            (
                "-> {{route}::missing}",
                "Interface 'IItem' does not declare member 'missing'",
            ),
            (
                "-> {{route}::score}",
                "Interface 'IItem' member 'score' is a function but dynamic target access requires a knot",
            ),
            (
                "-> {{route}::target}",
                "Dynamic interface target 'target' expects 1 arguments but got 0",
            ),
            (
                "-> {{route}::target}(\"bad\")",
                "Argument 'amount' for dynamic interface target 'target' has type string but expected int",
            ),
        ];

        for (divert, expected_message) in cases {
            let story = parse_story(&format!(
                "=== interface IItem ===\n\
                 == target(amount: int) ==\n\
                 == function score() => int ==\n\
                 === module game ===\n\
                 FROM left\n\
                 VAR route: interface<IItem> = left\n\
                 VAR label: string = \"x\"\n\
                 == main ==\n\
                 {divert}\n\
                 === module left implements IItem ===\n\
                 == target(amount: int) ==\n\
                 -> END\n\
                 == function score() => int ==\n\
                 ~ return 1",
            ));
            let diagnostics = call_target_diagnostics(&story);

            assert_single_diagnostic(&diagnostics, DiagnosticSeverity::Error, expected_message);
        }
    }

    #[test]
    fn keeps_plain_dynamic_divert_targets_unchanged() {
        let story = parse_story(
            "VAR next: -> = -> done\n\
             -> {next}\n\
             == done ==\n\
             -> END",
        );

        assert_eq!(call_target_diagnostics(&story), []);
    }

    #[test]
    fn checks_plain_dynamic_divert_arguments() {
        let story = parse_story(
            "VAR next: -> = -> done\n\
             -> {next}(missing())\n\
             == done ==\n\
             -> END",
        );
        let diagnostics = call_target_diagnostics(&story);

        assert_single_diagnostic(
            &diagnostics,
            DiagnosticSeverity::Error,
            "Function 'missing' is not declared",
        );
    }
}
