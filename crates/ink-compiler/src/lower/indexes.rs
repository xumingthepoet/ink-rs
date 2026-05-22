use std::collections::{HashMap, HashSet};

use crate::parsed::{
    Choice, ContentList, Expression, Flow, FlowArgument, InterfaceMemberSignature, Object, Story,
    TypeName, VariableAssignment,
};

use super::labels::build_label_index;
use super::path::LabelIndex;

#[derive(Debug)]
pub(super) struct LoweringIndexes<'a> {
    pub(super) constants: ConstantValues,
    pub(super) global_labels: LabelIndex,
    pub(super) global_variables: HashSet<String>,
    pub(super) global_variable_types: HashMap<String, TypeName>,
    pub(super) variable_declarations: Vec<VariableDeclaration<'a>>,
    pub(super) external_signatures: ExternalSignatures,
    pub(super) interface_members: InterfaceMemberSignatures,
    pub(super) struct_definitions: StructDefinitions,
    pub(super) enum_definitions: EnumDefinitions,
}

pub(super) struct RuntimeLenEstimator {
    pub(super) choice_content_len: fn(
        &Choice,
        &ConstantValues,
        &StructDefinitions,
        &EnumDefinitions,
        &HashSet<String>,
    ) -> usize,
    pub(super) object_len: fn(
        &Object,
        &ConstantValues,
        &StructDefinitions,
        &EnumDefinitions,
        &HashSet<String>,
    ) -> usize,
}

impl<'a> LoweringIndexes<'a> {
    pub(super) fn build(story: &'a Story, estimator: RuntimeLenEstimator) -> Self {
        let constants = build_constant_values(story);
        let struct_definitions = build_struct_definitions(story);
        let enum_definitions = build_enum_definitions(story);
        let variable_declarations = collect_story_variable_declarations(story);
        let global_variables = build_global_variable_names(&variable_declarations);
        let global_variable_types = build_global_variable_types(&variable_declarations);
        let global_labels = build_label_index(
            story,
            &constants,
            &struct_definitions,
            &enum_definitions,
            &global_variables,
            &estimator,
        );
        let external_signatures = build_external_signatures(story);
        let interface_members = build_interface_member_signatures(story);

        Self {
            constants,
            global_labels,
            global_variables,
            global_variable_types,
            variable_declarations,
            external_signatures,
            interface_members,
            struct_definitions,
            enum_definitions,
        }
    }
}

pub(super) type ExternalSignatures = HashMap<String, CallSignature>;
pub(super) type InterfaceMemberSignatures =
    HashMap<String, HashMap<String, InterfaceMemberSignature>>;
pub(super) type StructDefinitions = HashMap<String, Vec<(String, crate::parsed::TypeName)>>;
pub(super) type EnumDefinitions = HashMap<String, Vec<String>>;
pub(super) type ConstantValues = HashMap<String, ConstantValue>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct VariableDeclaration<'a> {
    assignment: &'a VariableAssignment,
    module_name: Option<String>,
    runtime_name: String,
}

impl<'a> VariableDeclaration<'a> {
    fn new(
        assignment: &'a VariableAssignment,
        module_name: Option<&str>,
        runtime_name: impl Into<String>,
    ) -> Self {
        Self {
            assignment,
            module_name: module_name.map(str::to_string),
            runtime_name: runtime_name.into(),
        }
    }

    pub(super) fn assignment(&self) -> &'a VariableAssignment {
        self.assignment
    }

    pub(super) fn runtime_name(&self) -> &str {
        &self.runtime_name
    }

    pub(super) fn module_name(&self) -> Option<&str> {
        self.module_name.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ConstantValue {
    expression: Expression,
    declared_type: TypeName,
    module_name: Option<String>,
}

impl ConstantValue {
    fn new(expression: Expression, declared_type: TypeName, module_name: Option<&str>) -> Self {
        Self {
            expression,
            declared_type,
            module_name: module_name.map(str::to_string),
        }
    }

    pub(super) fn expression(&self) -> &Expression {
        &self.expression
    }

    pub(super) fn declared_type(&self) -> &TypeName {
        &self.declared_type
    }

    pub(super) fn module_name(&self) -> Option<&str> {
        self.module_name.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CallSignature {
    External {
        args: usize,
        return_type: TypeName,
    },
    Internal {
        args: Vec<FlowArgument>,
        return_type: TypeName,
    },
}

fn collect_variable_declarations_in_objects<'a>(
    objects: &'a [Object],
    module_name: Option<&str>,
    declarations: &mut Vec<VariableDeclaration<'a>>,
) {
    for object in objects {
        collect_variable_declarations_in_object(object, module_name, declarations);
    }
}

fn collect_variable_declarations_in_content_list<'a>(
    content_list: &'a ContentList,
    module_name: Option<&str>,
    declarations: &mut Vec<VariableDeclaration<'a>>,
) {
    collect_variable_declarations_in_objects(content_list.objects(), module_name, declarations);
}

fn collect_variable_declarations_in_object<'a>(
    object: &'a Object,
    module_name: Option<&str>,
    declarations: &mut Vec<VariableDeclaration<'a>>,
) {
    match object {
        Object::VariableAssignment(assignment)
            if assignment.is_global() || assignment.is_temporary() =>
        {
            let runtime_name = if assignment.is_global() {
                module_name
                    .map(|module| format!("{module}::{}", assignment.name()))
                    .unwrap_or_else(|| assignment.name().to_string())
            } else {
                assignment.name().to_string()
            };
            declarations.push(VariableDeclaration::new(
                assignment,
                module_name,
                runtime_name,
            ));
        }
        Object::ContentList(content_list) => {
            collect_variable_declarations_in_content_list(content_list, module_name, declarations);
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_variable_declarations_in_objects(
                    branch.content().content(),
                    module_name,
                    declarations,
                );
            }
        }
        Object::ForLoop(for_loop) => {
            collect_variable_declarations_in_objects(
                for_loop.body().content(),
                module_name,
                declarations,
            );
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_variable_declarations_in_content_list(content, module_name, declarations);
            }
            collect_variable_declarations_in_content_list(
                choice.inner_content(),
                module_name,
                declarations,
            );
        }
        Object::Weave(weave) => {
            collect_variable_declarations_in_objects(weave.content(), module_name, declarations);
        }
        _ => {}
    }
}

fn build_global_variable_names(declarations: &[VariableDeclaration<'_>]) -> HashSet<String> {
    declarations
        .iter()
        .filter_map(|declaration| match declaration.assignment() {
            assignment if assignment.is_global() => Some(declaration.runtime_name().to_string()),
            _ => None,
        })
        .collect()
}

fn build_global_variable_types(
    declarations: &[VariableDeclaration<'_>],
) -> HashMap<String, TypeName> {
    declarations
        .iter()
        .filter_map(|declaration| {
            let assignment = declaration.assignment();
            if !assignment.is_global() {
                return None;
            }
            assignment
                .declared_type()
                .cloned()
                .map(|declared_type| (declaration.runtime_name().to_string(), declared_type))
        })
        .collect()
}

fn build_interface_member_signatures(story: &Story) -> InterfaceMemberSignatures {
    let mut signatures = HashMap::new();
    for interface in story.interfaces() {
        let members = interface
            .members()
            .iter()
            .map(|member| (member.name().to_string(), member.clone()))
            .collect::<HashMap<_, _>>();
        signatures.insert(interface.name().to_string(), members);
    }
    signatures
}

fn build_struct_definitions(story: &Story) -> StructDefinitions {
    let mut definitions = HashMap::new();
    collect_struct_definitions_in_objects(story.root_weave().content(), None, &mut definitions);
    for flow in story.flows() {
        collect_struct_definitions_in_flow(flow, None, &mut definitions);
    }
    for module in story.modules() {
        collect_struct_definitions_in_objects(
            module.weave().content(),
            Some(module.name()),
            &mut definitions,
        );
        for flow in module.flows() {
            collect_struct_definitions_in_flow(flow, Some(module.name()), &mut definitions);
        }
    }
    definitions
}

fn build_enum_definitions(story: &Story) -> EnumDefinitions {
    let mut definitions = HashMap::new();
    collect_enum_definitions_in_objects(story.root_weave().content(), None, &mut definitions);
    for flow in story.flows() {
        collect_enum_definitions_in_flow(flow, None, &mut definitions);
    }
    for module in story.modules() {
        collect_enum_definitions_in_objects(
            module.weave().content(),
            Some(module.name()),
            &mut definitions,
        );
        for flow in module.flows() {
            collect_enum_definitions_in_flow(flow, Some(module.name()), &mut definitions);
        }
    }
    definitions
}

fn collect_enum_definitions_in_flow(
    flow: &Flow,
    module_name: Option<&str>,
    definitions: &mut EnumDefinitions,
) {
    collect_enum_definitions_in_objects(flow.weave().content(), module_name, definitions);
    for child in flow.child_flows() {
        collect_enum_definitions_in_flow(child, module_name, definitions);
    }
}

fn collect_enum_definitions_in_objects(
    objects: &[Object],
    module_name: Option<&str>,
    definitions: &mut EnumDefinitions,
) {
    for object in objects {
        collect_enum_definitions_in_object(object, module_name, definitions);
    }
}

fn collect_enum_definitions_in_content_list(
    content_list: &ContentList,
    module_name: Option<&str>,
    definitions: &mut EnumDefinitions,
) {
    collect_enum_definitions_in_objects(content_list.objects(), module_name, definitions);
}

fn collect_enum_definitions_in_object(
    object: &Object,
    module_name: Option<&str>,
    definitions: &mut EnumDefinitions,
) {
    match object {
        Object::EnumDeclaration(declaration) => {
            let definition_name = module_name
                .map(|module| format!("{module}::{}", declaration.name()))
                .unwrap_or_else(|| declaration.name().to_string());
            definitions.entry(definition_name).or_insert_with(|| {
                declaration
                    .members()
                    .iter()
                    .map(|member| member.name().to_string())
                    .collect()
            });
        }
        Object::ContentList(content_list) => {
            collect_enum_definitions_in_content_list(content_list, module_name, definitions);
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_enum_definitions_in_objects(
                    branch.content().content(),
                    module_name,
                    definitions,
                );
            }
        }
        Object::ForLoop(for_loop) => {
            collect_enum_definitions_in_objects(
                for_loop.body().content(),
                module_name,
                definitions,
            );
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_enum_definitions_in_content_list(content, module_name, definitions);
            }
            collect_enum_definitions_in_content_list(
                choice.inner_content(),
                module_name,
                definitions,
            );
        }
        Object::Weave(weave) => {
            collect_enum_definitions_in_objects(weave.content(), module_name, definitions)
        }
        _ => {}
    }
}

fn collect_struct_definitions_in_flow(
    flow: &Flow,
    module_name: Option<&str>,
    definitions: &mut StructDefinitions,
) {
    collect_struct_definitions_in_objects(flow.weave().content(), module_name, definitions);
    for child in flow.child_flows() {
        collect_struct_definitions_in_flow(child, module_name, definitions);
    }
}

fn collect_struct_definitions_in_objects(
    objects: &[Object],
    module_name: Option<&str>,
    definitions: &mut StructDefinitions,
) {
    for object in objects {
        collect_struct_definitions_in_object(object, module_name, definitions);
    }
}

fn collect_struct_definitions_in_content_list(
    content_list: &ContentList,
    module_name: Option<&str>,
    definitions: &mut StructDefinitions,
) {
    collect_struct_definitions_in_objects(content_list.objects(), module_name, definitions);
}

fn collect_struct_definitions_in_object(
    object: &Object,
    module_name: Option<&str>,
    definitions: &mut StructDefinitions,
) {
    match object {
        Object::StructDeclaration(declaration) => {
            let definition_name = module_name
                .map(|module| format!("{module}::{}", declaration.name()))
                .unwrap_or_else(|| declaration.name().to_string());
            definitions.entry(definition_name).or_insert_with(|| {
                declaration
                    .fields()
                    .iter()
                    .map(|field| (field.name().to_string(), field.type_name().clone()))
                    .collect()
            });
        }
        Object::ContentList(content_list) => {
            collect_struct_definitions_in_content_list(content_list, module_name, definitions);
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_struct_definitions_in_objects(
                    branch.content().content(),
                    module_name,
                    definitions,
                );
            }
        }
        Object::ForLoop(for_loop) => {
            collect_struct_definitions_in_objects(
                for_loop.body().content(),
                module_name,
                definitions,
            );
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_struct_definitions_in_content_list(content, module_name, definitions);
            }
            collect_struct_definitions_in_content_list(
                choice.inner_content(),
                module_name,
                definitions,
            );
        }
        Object::Weave(weave) => {
            collect_struct_definitions_in_objects(weave.content(), module_name, definitions)
        }
        _ => {}
    }
}

fn collect_story_variable_declarations(story: &Story) -> Vec<VariableDeclaration<'_>> {
    let mut declarations = Vec::new();
    collect_variable_declarations_in_objects(story.root_weave().content(), None, &mut declarations);
    for flow in story.flows() {
        collect_variable_declarations_in_flow(flow, None, &mut declarations);
    }
    for module in story.modules() {
        collect_variable_declarations_in_objects(
            module.weave().content(),
            Some(module.name()),
            &mut declarations,
        );
        for flow in module.flows() {
            collect_variable_declarations_in_flow(flow, Some(module.name()), &mut declarations);
        }
    }
    declarations
}

fn collect_variable_declarations_in_flow<'a>(
    flow: &'a Flow,
    module_name: Option<&str>,
    declarations: &mut Vec<VariableDeclaration<'a>>,
) {
    collect_variable_declarations_in_objects(flow.weave().content(), module_name, declarations);
    for child in flow.child_flows() {
        collect_variable_declarations_in_flow(child, module_name, declarations);
    }
}

fn build_constant_values(story: &Story) -> ConstantValues {
    let mut constants = HashMap::new();
    collect_constant_values_in_objects(story.root_weave().content(), None, &mut constants);
    for flow in story.flows() {
        collect_constant_values_in_flow(flow, None, &mut constants);
    }
    for module in story.modules() {
        collect_constant_values_in_objects(
            module.weave().content(),
            Some(module.name()),
            &mut constants,
        );
        for flow in module.flows() {
            collect_constant_values_in_flow(flow, Some(module.name()), &mut constants);
        }
    }
    constants
}

fn collect_constant_values_in_flow(
    flow: &Flow,
    module_name: Option<&str>,
    constants: &mut ConstantValues,
) {
    collect_constant_values_in_objects(flow.weave().content(), module_name, constants);
    for child in flow.child_flows() {
        collect_constant_values_in_flow(child, module_name, constants);
    }
}

fn collect_constant_values_in_content_list(
    content_list: &ContentList,
    module_name: Option<&str>,
    constants: &mut ConstantValues,
) {
    collect_constant_values_in_objects(content_list.objects(), module_name, constants);
}

fn collect_constant_values_in_objects(
    objects: &[Object],
    module_name: Option<&str>,
    constants: &mut ConstantValues,
) {
    for object in objects {
        collect_constant_values_in_object(object, module_name, constants);
    }
}

fn collect_constant_values_in_object(
    object: &Object,
    module_name: Option<&str>,
    constants: &mut ConstantValues,
) {
    match object {
        Object::ConstantDeclaration(constant) => {
            let constant_name = module_name
                .map(|module| format!("{module}::{}", constant.name()))
                .unwrap_or_else(|| constant.name().to_string());
            constants.insert(
                constant_name,
                ConstantValue::new(
                    constant.expression().clone(),
                    constant.declared_type().clone(),
                    module_name,
                ),
            );
        }
        Object::ContentList(content_list) => {
            collect_constant_values_in_content_list(content_list, module_name, constants);
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_constant_values_in_objects(
                    branch.content().content(),
                    module_name,
                    constants,
                );
            }
        }
        Object::ForLoop(for_loop) => {
            collect_constant_values_in_objects(for_loop.body().content(), module_name, constants);
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_constant_values_in_content_list(content, module_name, constants);
            }
            collect_constant_values_in_content_list(choice.inner_content(), module_name, constants);
        }
        Object::Weave(weave) => {
            collect_constant_values_in_objects(weave.content(), module_name, constants)
        }
        Object::AuthorWarning(_)
        | Object::EnumDeclaration(_)
        | Object::Text(_)
        | Object::Expression(_)
        | Object::LogicLine(_)
        | Object::Glue(_)
        | Object::Divert(_)
        | Object::TunnelOnwards(_)
        | Object::Gather(_)
        | Object::VariableAssignment(_)
        | Object::IncDec(_)
        | Object::Return(_)
        | Object::ExternalDeclaration(_)
        | Object::StructDeclaration(_)
        | Object::Tag(_) => {}
    }
}

fn build_external_signatures(story: &Story) -> ExternalSignatures {
    let mut signatures = HashMap::new();
    collect_external_signatures_in_objects(story.root_weave().content(), None, &mut signatures);
    for flow in story.flows() {
        collect_external_signatures_in_flow(flow, None, &mut signatures);
        collect_internal_call_signatures_in_flow(flow, None, None, &mut signatures);
    }
    for module in story.modules() {
        collect_external_signatures_in_objects(
            module.weave().content(),
            Some(module.name()),
            &mut signatures,
        );
        for flow in module.flows() {
            collect_external_signatures_in_flow(flow, Some(module.name()), &mut signatures);
            collect_internal_call_signatures_in_flow(
                flow,
                Some(module.name()),
                None,
                &mut signatures,
            );
        }
    }
    signatures
}

fn collect_external_signatures_in_flow(
    flow: &Flow,
    module_name: Option<&str>,
    signatures: &mut ExternalSignatures,
) {
    collect_external_signatures_in_objects(flow.weave().content(), module_name, signatures);
    for child in flow.child_flows() {
        collect_external_signatures_in_flow(child, module_name, signatures);
    }
}

fn collect_internal_call_signatures_in_flow(
    flow: &Flow,
    module_name: Option<&str>,
    parent_flow_path: Option<&str>,
    signatures: &mut ExternalSignatures,
) {
    let flow_path = parent_flow_path
        .map(|parent| format!("{parent}.{}", flow.name()))
        .unwrap_or_else(|| flow.name().to_string());
    if flow.is_function() || flow.has_typed_signature() || !flow.arguments().is_empty() {
        let signature_name = module_name
            .map(|module| format!("{module}::{flow_path}"))
            .unwrap_or_else(|| flow_path.clone());
        let signature = CallSignature::Internal {
            args: flow.arguments().to_vec(),
            return_type: flow.return_type().clone(),
        };
        signatures
            .entry(signature_name)
            .or_insert_with(|| signature.clone());
        if parent_flow_path.is_none() {
            let short_name = module_name
                .map(|module| format!("{module}::{}", flow.name()))
                .unwrap_or_else(|| flow.name().to_string());
            signatures.entry(short_name).or_insert(signature);
        }
    }
    for child in flow.child_flows() {
        collect_internal_call_signatures_in_flow(child, module_name, Some(&flow_path), signatures);
    }
}

fn collect_external_signatures_in_objects(
    objects: &[Object],
    module_name: Option<&str>,
    signatures: &mut ExternalSignatures,
) {
    for object in objects {
        collect_external_signatures_in_object(object, module_name, signatures);
    }
}

fn collect_external_signatures_in_content_list(
    content_list: &ContentList,
    module_name: Option<&str>,
    signatures: &mut ExternalSignatures,
) {
    collect_external_signatures_in_objects(content_list.objects(), module_name, signatures);
}

fn collect_external_signatures_in_object(
    object: &Object,
    module_name: Option<&str>,
    signatures: &mut ExternalSignatures,
) {
    match object {
        Object::ExternalDeclaration(external) => {
            let signature_name = module_name
                .map(|module| format!("{module}::{}", external.name()))
                .unwrap_or_else(|| external.name().to_string());
            signatures.insert(
                signature_name,
                CallSignature::External {
                    args: external.argument_names().len(),
                    return_type: external.return_type().clone(),
                },
            );
        }
        Object::ContentList(content_list) => {
            collect_external_signatures_in_content_list(content_list, module_name, signatures);
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_external_signatures_in_objects(
                    branch.content().content(),
                    module_name,
                    signatures,
                );
            }
        }
        Object::ForLoop(for_loop) => {
            collect_external_signatures_in_objects(
                for_loop.body().content(),
                module_name,
                signatures,
            );
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_external_signatures_in_content_list(content, module_name, signatures);
            }
            collect_external_signatures_in_content_list(
                choice.inner_content(),
                module_name,
                signatures,
            );
        }
        Object::Weave(weave) => {
            collect_external_signatures_in_objects(weave.content(), module_name, signatures);
        }
        _ => {}
    }
}
