use std::collections::{HashMap, HashSet};

use crate::parsed::{
    Choice, ContentList, Expression, Flow, FlowArgument, Object, Story, TypeName,
    VariableAssignment, Weave,
};

use super::context::ChoicePathMode;
use super::flow::collect_flow_local_variables;
use super::labels::build_label_index;
use super::path::LabelIndex;
use super::weave::weave_has_weave_points;

#[derive(Debug)]
pub(super) struct LoweringIndexes<'a> {
    pub(super) constants: ConstantValues,
    pub(super) global_labels: LabelIndex,
    pub(super) global_variables: HashSet<String>,
    pub(super) variable_declarations: Vec<VariableDeclaration<'a>>,
    pub(super) external_signatures: ExternalSignatures,
    pub(super) struct_definitions: StructDefinitions,
    pub(super) counted_flow_paths: CountedFlowPaths,
}

pub(super) struct RuntimeLenEstimator {
    pub(super) choice_content_len:
        fn(&Choice, &ConstantValues, &StructDefinitions, &HashSet<String>) -> usize,
    pub(super) object_len:
        fn(&Object, &ConstantValues, &StructDefinitions, &HashSet<String>) -> usize,
}

impl<'a> LoweringIndexes<'a> {
    pub(super) fn build(story: &'a Story, estimator: RuntimeLenEstimator) -> Self {
        let constants = build_constant_values(story);
        let struct_definitions = build_struct_definitions(story);
        let variable_declarations = collect_story_variable_declarations(story);
        let global_variables = build_global_variable_names(&variable_declarations);
        let global_labels = build_label_index(
            story,
            &constants,
            &struct_definitions,
            &global_variables,
            &estimator,
        );
        let external_signatures = build_external_signatures(story);
        let counted_flow_paths =
            build_counted_flow_paths(story, &global_labels, &constants, &global_variables);

        Self {
            constants,
            global_labels,
            global_variables,
            variable_declarations,
            external_signatures,
            struct_definitions,
            counted_flow_paths,
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct CountedFlowPaths {
    pub(super) visits: HashSet<String>,
    pub(super) turns: HashSet<String>,
}

pub(super) type ExternalSignatures = HashMap<String, CallSignature>;
pub(super) type StructDefinitions = HashMap<String, Vec<(String, crate::parsed::TypeName)>>;
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
}

impl ConstantValue {
    fn new(expression: Expression, declared_type: TypeName) -> Self {
        Self {
            expression,
            declared_type,
        }
    }

    pub(super) fn expression(&self) -> &Expression {
        &self.expression
    }

    pub(super) fn declared_type(&self) -> &TypeName {
        &self.declared_type
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CallSignature {
    External {
        args: usize,
        return_type: TypeName,
    },
    Ink {
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
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                collect_variable_declarations_in_content_list(element, module_name, declarations);
            }
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
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                collect_struct_definitions_in_content_list(element, module_name, definitions);
            }
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
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_constant_values_in_content_list(content, module_name, constants);
            }
            collect_constant_values_in_content_list(choice.inner_content(), module_name, constants);
        }
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                collect_constant_values_in_content_list(element, module_name, constants);
            }
        }
        Object::Weave(weave) => {
            collect_constant_values_in_objects(weave.content(), module_name, constants)
        }
        Object::AuthorWarning(_)
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
        collect_ink_call_signatures_in_flow(flow, None, &mut signatures);
    }
    for module in story.modules() {
        collect_external_signatures_in_objects(
            module.weave().content(),
            Some(module.name()),
            &mut signatures,
        );
        for flow in module.flows() {
            collect_external_signatures_in_flow(flow, Some(module.name()), &mut signatures);
            collect_ink_call_signatures_in_flow(flow, Some(module.name()), &mut signatures);
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

fn collect_ink_call_signatures_in_flow(
    flow: &Flow,
    module_name: Option<&str>,
    signatures: &mut ExternalSignatures,
) {
    if flow.is_function() {
        let signature_name = module_name
            .map(|module| format!("{module}::{}", flow.name()))
            .unwrap_or_else(|| flow.name().to_string());
        signatures
            .entry(signature_name)
            .or_insert_with(|| CallSignature::Ink {
                args: flow.arguments().to_vec(),
                return_type: flow.return_type().clone(),
            });
    }
    for child in flow.child_flows() {
        collect_ink_call_signatures_in_flow(child, module_name, signatures);
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
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                collect_external_signatures_in_content_list(element, module_name, signatures);
            }
        }
        Object::Weave(weave) => {
            collect_external_signatures_in_objects(weave.content(), module_name, signatures);
        }
        _ => {}
    }
}

fn build_counted_flow_paths(
    story: &Story,
    global_labels: &LabelIndex,
    constants: &ConstantValues,
    global_variables: &HashSet<String>,
) -> CountedFlowPaths {
    let mut paths = CountedFlowPaths::default();
    collect_counted_paths_in_weave(
        story.root_weave(),
        global_labels,
        constants,
        global_variables,
        &ChoicePathMode::Root,
        &mut paths,
    );
    for flow in story.flows() {
        let child_stitch_names = flow
            .child_flows()
            .iter()
            .map(|child| child.name().to_string())
            .collect::<Vec<_>>();
        collect_counted_paths_in_flow(
            flow,
            None,
            None,
            &child_stitch_names,
            global_labels,
            constants,
            global_variables,
            &mut paths,
        );
    }
    for module in story.modules() {
        for flow in module.flows() {
            let child_stitch_names = flow
                .child_flows()
                .iter()
                .map(|child| child.name().to_string())
                .collect::<Vec<_>>();
            collect_counted_paths_in_flow(
                flow,
                Some(module.name()),
                None,
                &child_stitch_names,
                global_labels,
                constants,
                global_variables,
                &mut paths,
            );
        }
    }
    paths
}

pub(super) fn collect_counted_paths_in_weave(
    weave: &Weave,
    global_labels: &LabelIndex,
    constants: &ConstantValues,
    global_variables: &HashSet<String>,
    path_mode: &ChoicePathMode,
    paths: &mut CountedFlowPaths,
) {
    for object in weave.content() {
        collect_counted_paths_in_object(
            object,
            global_labels,
            constants,
            global_variables,
            path_mode,
            paths,
        );
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "counted path collection still passes explicit recursion context"
)]
fn collect_counted_paths_in_flow(
    flow: &Flow,
    module_name: Option<&str>,
    parent_flow_name: Option<&str>,
    sibling_stitch_names: &[String],
    global_labels: &LabelIndex,
    constants: &ConstantValues,
    global_variables: &HashSet<String>,
    paths: &mut CountedFlowPaths,
) {
    let source_flow_path = parent_flow_name
        .map(|parent| format!("{parent}.{}", flow.name()))
        .unwrap_or_else(|| flow.name().to_string());
    let flow_path = module_name
        .map(|module| format!("{module}.{source_flow_path}"))
        .unwrap_or_else(|| source_flow_path.clone());
    let container_path = if weave_has_weave_points(flow.weave()) {
        format!("{}.{}", flow_path, flow.arguments().len())
    } else {
        flow_path.clone()
    };
    let path_mode = ChoicePathMode::Flow {
        module_name: module_name.map(str::to_string),
        flow_name: flow.name().to_string(),
        container_path,
        parent_flow_name: parent_flow_name.map(str::to_string),
        sibling_stitch_names: sibling_stitch_names.to_vec(),
        local_variables: collect_flow_local_variables(flow),
        self_target_relative: false,
        fallback_gather_target: None,
    };
    collect_counted_paths_in_weave(
        flow.weave(),
        global_labels,
        constants,
        global_variables,
        &path_mode,
        paths,
    );

    let child_stitch_names = flow
        .child_flows()
        .iter()
        .map(|child| child.name().to_string())
        .collect::<Vec<_>>();
    for child in flow.child_flows() {
        collect_counted_paths_in_flow(
            child,
            module_name,
            Some(&source_flow_path),
            &child_stitch_names,
            global_labels,
            constants,
            global_variables,
            paths,
        );
    }
}

fn collect_counted_paths_in_content_list(
    content_list: &ContentList,
    global_labels: &LabelIndex,
    constants: &ConstantValues,
    global_variables: &HashSet<String>,
    path_mode: &ChoicePathMode,
    paths: &mut CountedFlowPaths,
) {
    for object in content_list.objects() {
        collect_counted_paths_in_object(
            object,
            global_labels,
            constants,
            global_variables,
            path_mode,
            paths,
        );
    }
}

fn collect_counted_paths_in_object(
    object: &Object,
    global_labels: &LabelIndex,
    constants: &ConstantValues,
    global_variables: &HashSet<String>,
    path_mode: &ChoicePathMode,
    paths: &mut CountedFlowPaths,
) {
    match object {
        Object::ContentList(content_list) => {
            collect_counted_paths_in_content_list(
                content_list,
                global_labels,
                constants,
                global_variables,
                path_mode,
                paths,
            );
        }
        Object::Expression(expression) | Object::LogicLine(expression) => {
            collect_counted_paths_in_expression(
                expression,
                global_labels,
                constants,
                global_variables,
                path_mode,
                paths,
            );
        }
        Object::Conditional(conditional) => {
            if let Some(condition) = conditional.initial_condition() {
                collect_counted_paths_in_expression(
                    condition,
                    global_labels,
                    constants,
                    global_variables,
                    path_mode,
                    paths,
                );
            }
            for branch in conditional.branches() {
                if let Some(condition) = branch.own_condition() {
                    collect_counted_paths_in_expression(
                        condition,
                        global_labels,
                        constants,
                        global_variables,
                        path_mode,
                        paths,
                    );
                }
                collect_counted_paths_in_weave(
                    branch.content(),
                    global_labels,
                    constants,
                    global_variables,
                    path_mode,
                    paths,
                );
            }
        }
        Object::Choice(choice) => {
            if let Some(condition) = choice.condition() {
                collect_counted_paths_in_expression(
                    condition,
                    global_labels,
                    constants,
                    global_variables,
                    path_mode,
                    paths,
                );
            }
            if let Some(content) = choice.start_content() {
                collect_counted_paths_in_content_list(
                    content,
                    global_labels,
                    constants,
                    global_variables,
                    path_mode,
                    paths,
                );
            }
            collect_counted_paths_in_content_list(
                choice.inner_content(),
                global_labels,
                constants,
                global_variables,
                path_mode,
                paths,
            );
        }
        Object::Divert(divert) => {
            for argument in divert.arguments() {
                collect_counted_paths_in_expression(
                    argument,
                    global_labels,
                    constants,
                    global_variables,
                    path_mode,
                    paths,
                );
            }
        }
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                collect_counted_paths_in_content_list(
                    element,
                    global_labels,
                    constants,
                    global_variables,
                    path_mode,
                    paths,
                );
            }
        }
        Object::VariableAssignment(assignment) => {
            if let Some(expression) = assignment.expression() {
                collect_counted_paths_in_expression(
                    expression,
                    global_labels,
                    constants,
                    global_variables,
                    path_mode,
                    paths,
                );
            }
        }
        Object::Return(ret) => {
            if let Some(expr) = ret.returned_expression() {
                collect_counted_paths_in_expression(
                    expr,
                    global_labels,
                    constants,
                    global_variables,
                    path_mode,
                    paths,
                );
            }
        }
        Object::Weave(weave) => collect_counted_paths_in_weave(
            weave,
            global_labels,
            constants,
            global_variables,
            path_mode,
            paths,
        ),
        Object::AuthorWarning(_)
        | Object::Text(_)
        | Object::ConstantDeclaration(_)
        | Object::Glue(_)
        | Object::IncDec(_)
        | Object::Gather(_)
        | Object::TunnelOnwards(_)
        | Object::ExternalDeclaration(_)
        | Object::StructDeclaration(_)
        | Object::Tag(_) => {}
    }
}

fn collect_counted_paths_in_expression(
    expression: &Expression,
    global_labels: &LabelIndex,
    constants: &ConstantValues,
    global_variables: &HashSet<String>,
    path_mode: &ChoicePathMode,
    paths: &mut CountedFlowPaths,
) {
    match expression {
        Expression::VariableReference(name) => {
            if let Some(constant) = constants.get(name) {
                collect_counted_paths_in_expression(
                    constant.expression(),
                    global_labels,
                    constants,
                    global_variables,
                    path_mode,
                    paths,
                );
            }
        }
        Expression::QualifiedReference(name) => {
            if let Some(constant) = constants.get(name.as_str()) {
                collect_counted_paths_in_expression(
                    constant.expression(),
                    global_labels,
                    constants,
                    global_variables,
                    path_mode,
                    paths,
                );
            }
        }
        Expression::StringContent(content) => {
            collect_counted_paths_in_content_list(
                content,
                global_labels,
                constants,
                global_variables,
                path_mode,
                paths,
            );
        }
        Expression::FunctionCall { name: _, args }
        | Expression::QualifiedFunctionCall { name: _, args } => {
            for arg in args {
                collect_counted_paths_in_expression(
                    arg,
                    global_labels,
                    constants,
                    global_variables,
                    path_mode,
                    paths,
                );
            }
        }
        Expression::ArrayLiteral(elements) => {
            for element in elements {
                collect_counted_paths_in_expression(
                    element,
                    global_labels,
                    constants,
                    global_variables,
                    path_mode,
                    paths,
                );
            }
        }
        Expression::StructLiteral(fields) => {
            for field in fields {
                collect_counted_paths_in_expression(
                    field.expression(),
                    global_labels,
                    constants,
                    global_variables,
                    path_mode,
                    paths,
                );
            }
        }
        Expression::FieldAccess { base, .. } => {
            collect_counted_paths_in_expression(
                base,
                global_labels,
                constants,
                global_variables,
                path_mode,
                paths,
            );
        }
        Expression::IndexAccess { base, index } => {
            collect_counted_paths_in_expression(
                base,
                global_labels,
                constants,
                global_variables,
                path_mode,
                paths,
            );
            collect_counted_paths_in_expression(
                index,
                global_labels,
                constants,
                global_variables,
                path_mode,
                paths,
            );
        }
        Expression::MultipleCondition(args) => {
            for arg in args {
                collect_counted_paths_in_expression(
                    arg,
                    global_labels,
                    constants,
                    global_variables,
                    path_mode,
                    paths,
                );
            }
        }
        Expression::Binary { left, right, .. } => {
            collect_counted_paths_in_expression(
                left,
                global_labels,
                constants,
                global_variables,
                path_mode,
                paths,
            );
            collect_counted_paths_in_expression(
                right,
                global_labels,
                constants,
                global_variables,
                path_mode,
                paths,
            );
        }
        Expression::Unary { expression, .. } => {
            collect_counted_paths_in_expression(
                expression,
                global_labels,
                constants,
                global_variables,
                path_mode,
                paths,
            );
        }
        Expression::DivertTarget(_) => {}
        Expression::String(_)
        | Expression::NumberInt(_)
        | Expression::NumberFloat(_)
        | Expression::NumberBool(_) => {}
    }
}
