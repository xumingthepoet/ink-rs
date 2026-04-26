use std::collections::{HashMap, HashSet};

use crate::parsed::{
    Choice, ContentList, Expression, Flow, FlowArgument, Object, Story, VariableAssignment, Weave,
};

use super::context::ChoicePathMode;
use super::flow::collect_flow_local_variables;
use super::labels::build_label_index;
use super::path::LabelIndex;
use super::weave::weave_has_weave_points;

#[derive(Debug)]
pub(super) struct LoweringIndexes<'a> {
    pub(super) constants: HashMap<String, Expression>,
    pub(super) global_labels: LabelIndex,
    pub(super) global_variables: HashSet<String>,
    pub(super) variable_declarations: Vec<&'a VariableAssignment>,
    pub(super) external_signatures: ExternalSignatures,
    pub(super) struct_definitions: StructDefinitions,
    pub(super) counted_flow_paths: CountedFlowPaths,
}

pub(super) struct RuntimeLenEstimator {
    pub(super) choice_content_len:
        fn(&Choice, &HashMap<String, Expression>, &StructDefinitions) -> usize,
    pub(super) object_len: fn(&Object, &HashMap<String, Expression>, &StructDefinitions) -> usize,
}

impl<'a> LoweringIndexes<'a> {
    pub(super) fn build(story: &'a Story, estimator: RuntimeLenEstimator) -> Self {
        let constants = build_constant_values(story);
        let struct_definitions = build_struct_definitions(story);
        let global_labels = build_label_index(story, &constants, &struct_definitions, &estimator);
        let variable_declarations = collect_story_variable_declarations(story);
        let global_variables = build_global_variable_names(&variable_declarations);
        let external_signatures = build_external_signatures(story);
        let counted_flow_paths = build_counted_flow_paths(story, &global_labels, &constants);

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CallSignature {
    External { args: usize },
    Ink { args: Vec<FlowArgument> },
}

fn collect_variable_declarations_in_objects<'a>(
    objects: &'a [Object],
    declarations: &mut Vec<&'a VariableAssignment>,
) {
    for object in objects {
        collect_variable_declarations_in_object(object, declarations);
    }
}

fn collect_variable_declarations_in_content_list<'a>(
    content_list: &'a ContentList,
    declarations: &mut Vec<&'a VariableAssignment>,
) {
    collect_variable_declarations_in_objects(content_list.objects(), declarations);
}

fn collect_variable_declarations_in_object<'a>(
    object: &'a Object,
    declarations: &mut Vec<&'a VariableAssignment>,
) {
    match object {
        Object::VariableAssignment(assignment)
            if assignment.is_global() || assignment.is_temporary() =>
        {
            declarations.push(assignment);
        }
        Object::ContentList(content_list) => {
            collect_variable_declarations_in_content_list(content_list, declarations);
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_variable_declarations_in_objects(branch.content().content(), declarations);
            }
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_variable_declarations_in_content_list(content, declarations);
            }
            if let Some(content) = choice.choice_only_content() {
                collect_variable_declarations_in_content_list(content, declarations);
            }
            collect_variable_declarations_in_content_list(choice.inner_content(), declarations);
        }
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                collect_variable_declarations_in_content_list(element, declarations);
            }
        }
        Object::Weave(weave) => {
            collect_variable_declarations_in_objects(weave.content(), declarations);
        }
        _ => {}
    }
}

fn build_global_variable_names(declarations: &[&VariableAssignment]) -> HashSet<String> {
    declarations
        .iter()
        .filter_map(|assignment| match assignment {
            assignment if assignment.is_global() => Some(assignment.name().to_string()),
            _ => None,
        })
        .collect()
}

fn build_struct_definitions(story: &Story) -> StructDefinitions {
    let mut definitions = HashMap::new();
    collect_struct_definitions_in_objects(story.root_weave().content(), &mut definitions);
    for flow in story.flows() {
        collect_struct_definitions_in_flow(flow, &mut definitions);
    }
    definitions
}

fn collect_struct_definitions_in_flow(flow: &Flow, definitions: &mut StructDefinitions) {
    collect_struct_definitions_in_objects(flow.weave().content(), definitions);
    for child in flow.child_flows() {
        collect_struct_definitions_in_flow(child, definitions);
    }
}

fn collect_struct_definitions_in_objects(objects: &[Object], definitions: &mut StructDefinitions) {
    for object in objects {
        collect_struct_definitions_in_object(object, definitions);
    }
}

fn collect_struct_definitions_in_content_list(
    content_list: &ContentList,
    definitions: &mut StructDefinitions,
) {
    collect_struct_definitions_in_objects(content_list.objects(), definitions);
}

fn collect_struct_definitions_in_object(object: &Object, definitions: &mut StructDefinitions) {
    match object {
        Object::StructDeclaration(declaration) => {
            definitions
                .entry(declaration.name().to_string())
                .or_insert_with(|| {
                    declaration
                        .fields()
                        .iter()
                        .map(|field| (field.name().to_string(), field.type_name().clone()))
                        .collect()
                });
        }
        Object::ContentList(content_list) => {
            collect_struct_definitions_in_content_list(content_list, definitions);
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_struct_definitions_in_objects(branch.content().content(), definitions);
            }
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_struct_definitions_in_content_list(content, definitions);
            }
            if let Some(content) = choice.choice_only_content() {
                collect_struct_definitions_in_content_list(content, definitions);
            }
            collect_struct_definitions_in_content_list(choice.inner_content(), definitions);
        }
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                collect_struct_definitions_in_content_list(element, definitions);
            }
        }
        Object::Weave(weave) => collect_struct_definitions_in_objects(weave.content(), definitions),
        _ => {}
    }
}

fn collect_story_variable_declarations(story: &Story) -> Vec<&VariableAssignment> {
    let mut declarations = Vec::new();
    collect_variable_declarations_in_objects(story.root_weave().content(), &mut declarations);
    for flow in story.flows() {
        collect_variable_declarations_in_flow(flow, &mut declarations);
    }
    declarations
}

fn collect_variable_declarations_in_flow<'a>(
    flow: &'a Flow,
    declarations: &mut Vec<&'a VariableAssignment>,
) {
    collect_variable_declarations_in_objects(flow.weave().content(), declarations);
    for child in flow.child_flows() {
        collect_variable_declarations_in_flow(child, declarations);
    }
}

fn build_constant_values(story: &Story) -> HashMap<String, Expression> {
    let mut constants = HashMap::new();
    collect_constant_values_in_objects(story.root_weave().content(), &mut constants);
    for flow in story.flows() {
        collect_constant_values_in_flow(flow, &mut constants);
    }
    constants
}

fn collect_constant_values_in_flow(flow: &Flow, constants: &mut HashMap<String, Expression>) {
    collect_constant_values_in_objects(flow.weave().content(), constants);
    for child in flow.child_flows() {
        collect_constant_values_in_flow(child, constants);
    }
}

fn collect_constant_values_in_content_list(
    content_list: &ContentList,
    constants: &mut HashMap<String, Expression>,
) {
    collect_constant_values_in_objects(content_list.objects(), constants);
}

fn collect_constant_values_in_objects(
    objects: &[Object],
    constants: &mut HashMap<String, Expression>,
) {
    for object in objects {
        collect_constant_values_in_object(object, constants);
    }
}

fn collect_constant_values_in_object(object: &Object, constants: &mut HashMap<String, Expression>) {
    match object {
        Object::ConstantDeclaration(constant) => {
            constants.insert(constant.name().to_string(), constant.expression().clone());
        }
        Object::ContentList(content_list) => {
            collect_constant_values_in_content_list(content_list, constants);
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_constant_values_in_objects(branch.content().content(), constants);
            }
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_constant_values_in_content_list(content, constants);
            }
            if let Some(content) = choice.choice_only_content() {
                collect_constant_values_in_content_list(content, constants);
            }
            collect_constant_values_in_content_list(choice.inner_content(), constants);
        }
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                collect_constant_values_in_content_list(element, constants);
            }
        }
        Object::Weave(weave) => collect_constant_values_in_objects(weave.content(), constants),
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
    collect_external_signatures_in_objects(story.root_weave().content(), &mut signatures);
    for flow in story.flows() {
        collect_external_signatures_in_flow(flow, &mut signatures);
        collect_ink_call_signatures_in_flow(flow, &mut signatures);
    }
    signatures
}

fn collect_external_signatures_in_flow(flow: &Flow, signatures: &mut ExternalSignatures) {
    collect_external_signatures_in_objects(flow.weave().content(), signatures);
    for child in flow.child_flows() {
        collect_external_signatures_in_flow(child, signatures);
    }
}

fn collect_ink_call_signatures_in_flow(flow: &Flow, signatures: &mut ExternalSignatures) {
    if flow.is_function() {
        signatures
            .entry(flow.name().to_string())
            .or_insert_with(|| CallSignature::Ink {
                args: flow.arguments().to_vec(),
            });
    }
    for child in flow.child_flows() {
        collect_ink_call_signatures_in_flow(child, signatures);
    }
}

fn collect_external_signatures_in_objects(objects: &[Object], signatures: &mut ExternalSignatures) {
    for object in objects {
        collect_external_signatures_in_object(object, signatures);
    }
}

fn collect_external_signatures_in_content_list(
    content_list: &ContentList,
    signatures: &mut ExternalSignatures,
) {
    collect_external_signatures_in_objects(content_list.objects(), signatures);
}

fn collect_external_signatures_in_object(object: &Object, signatures: &mut ExternalSignatures) {
    match object {
        Object::ExternalDeclaration(external) => {
            signatures.insert(
                external.name().to_string(),
                CallSignature::External {
                    args: external.argument_names().len(),
                },
            );
        }
        Object::ContentList(content_list) => {
            collect_external_signatures_in_content_list(content_list, signatures);
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_external_signatures_in_objects(branch.content().content(), signatures);
            }
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_external_signatures_in_content_list(content, signatures);
            }
            if let Some(content) = choice.choice_only_content() {
                collect_external_signatures_in_content_list(content, signatures);
            }
            collect_external_signatures_in_content_list(choice.inner_content(), signatures);
        }
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                collect_external_signatures_in_content_list(element, signatures);
            }
        }
        Object::Weave(weave) => {
            collect_external_signatures_in_objects(weave.content(), signatures);
        }
        _ => {}
    }
}

fn build_counted_flow_paths(
    story: &Story,
    global_labels: &LabelIndex,
    constants: &HashMap<String, Expression>,
) -> CountedFlowPaths {
    let mut paths = CountedFlowPaths::default();
    collect_counted_paths_in_weave(
        story.root_weave(),
        global_labels,
        constants,
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
            &child_stitch_names,
            global_labels,
            constants,
            &mut paths,
        );
    }
    paths
}

pub(super) fn collect_counted_paths_in_weave(
    weave: &Weave,
    global_labels: &LabelIndex,
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
    paths: &mut CountedFlowPaths,
) {
    for object in weave.content() {
        collect_counted_paths_in_object(object, global_labels, constants, path_mode, paths);
    }
}

fn collect_counted_paths_in_flow(
    flow: &Flow,
    parent_flow_name: Option<&str>,
    sibling_stitch_names: &[String],
    global_labels: &LabelIndex,
    constants: &HashMap<String, Expression>,
    paths: &mut CountedFlowPaths,
) {
    let flow_path = parent_flow_name
        .map(|parent| format!("{parent}.{}", flow.name()))
        .unwrap_or_else(|| flow.name().to_string());
    let container_path = if weave_has_weave_points(flow.weave()) {
        format!("{}.{}", flow_path, flow.arguments().len())
    } else {
        flow_path.clone()
    };
    let path_mode = ChoicePathMode::Flow {
        flow_name: flow.name().to_string(),
        container_path,
        parent_flow_name: parent_flow_name.map(str::to_string),
        sibling_stitch_names: sibling_stitch_names.to_vec(),
        local_variables: collect_flow_local_variables(flow),
        self_target_relative: false,
        fallback_gather_target: None,
    };
    collect_counted_paths_in_weave(flow.weave(), global_labels, constants, &path_mode, paths);

    let child_stitch_names = flow
        .child_flows()
        .iter()
        .map(|child| child.name().to_string())
        .collect::<Vec<_>>();
    for child in flow.child_flows() {
        collect_counted_paths_in_flow(
            child,
            Some(&flow_path),
            &child_stitch_names,
            global_labels,
            constants,
            paths,
        );
    }
}

fn collect_counted_paths_in_content_list(
    content_list: &ContentList,
    global_labels: &LabelIndex,
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
    paths: &mut CountedFlowPaths,
) {
    for object in content_list.objects() {
        collect_counted_paths_in_object(object, global_labels, constants, path_mode, paths);
    }
}

fn collect_counted_paths_in_object(
    object: &Object,
    global_labels: &LabelIndex,
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
    paths: &mut CountedFlowPaths,
) {
    match object {
        Object::ContentList(content_list) => {
            collect_counted_paths_in_content_list(
                content_list,
                global_labels,
                constants,
                path_mode,
                paths,
            );
        }
        Object::Expression(expression) | Object::LogicLine(expression) => {
            collect_counted_paths_in_expression(
                expression,
                global_labels,
                constants,
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
                        path_mode,
                        paths,
                    );
                }
                collect_counted_paths_in_weave(
                    branch.content(),
                    global_labels,
                    constants,
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
                    path_mode,
                    paths,
                );
            }
            if let Some(content) = choice.start_content() {
                collect_counted_paths_in_content_list(
                    content,
                    global_labels,
                    constants,
                    path_mode,
                    paths,
                );
            }
            if let Some(content) = choice.choice_only_content() {
                collect_counted_paths_in_content_list(
                    content,
                    global_labels,
                    constants,
                    path_mode,
                    paths,
                );
            }
            collect_counted_paths_in_content_list(
                choice.inner_content(),
                global_labels,
                constants,
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
                    path_mode,
                    paths,
                );
            }
        }
        Object::VariableAssignment(assignment) => {
            if let Some(expression) = assignment.expression() {
                match expression {
                    Expression::DivertTarget(target) if assignment.is_global() => {
                        insert_counted_divert_target(
                            target,
                            global_labels,
                            path_mode,
                            paths,
                            true,
                            true,
                        );
                    }
                    expression => collect_counted_paths_in_expression(
                        expression,
                        global_labels,
                        constants,
                        path_mode,
                        paths,
                    ),
                }
            }
        }
        Object::Return(ret) => {
            if let Some(expr) = ret.returned_expression() {
                collect_counted_paths_in_expression(
                    expr,
                    global_labels,
                    constants,
                    path_mode,
                    paths,
                );
            }
        }
        Object::Weave(weave) => {
            collect_counted_paths_in_weave(weave, global_labels, constants, path_mode, paths)
        }
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
    constants: &HashMap<String, Expression>,
    path_mode: &ChoicePathMode,
    paths: &mut CountedFlowPaths,
) {
    match expression {
        Expression::VariableReference(name) => {
            if let Some(constant) = constants.get(name) {
                collect_counted_paths_in_expression(
                    constant,
                    global_labels,
                    constants,
                    path_mode,
                    paths,
                );
            } else if let Some(target) = path_mode.scoped_label_target(name, global_labels) {
                paths.visits.insert(target.to_string());
            } else if path_mode.is_flow_sibling_stitch(name) {
                paths
                    .visits
                    .insert(path_mode.resolve_single_stitch_target(name));
            }
        }
        Expression::StringContent(content) => {
            collect_counted_paths_in_content_list(
                content,
                global_labels,
                constants,
                path_mode,
                paths,
            );
        }
        Expression::FunctionCall { name, args } => {
            let count_turns = name == "TURNS_SINCE";
            let count_visits = name == "READ_COUNT";
            for arg in args {
                match arg {
                    Expression::DivertTarget(target) if count_turns => {
                        insert_counted_divert_target(
                            target,
                            global_labels,
                            path_mode,
                            paths,
                            false,
                            true,
                        );
                    }
                    Expression::DivertTarget(target) if count_visits => {
                        insert_counted_divert_target(
                            target,
                            global_labels,
                            path_mode,
                            paths,
                            true,
                            false,
                        );
                    }
                    _ => collect_counted_paths_in_expression(
                        arg,
                        global_labels,
                        constants,
                        path_mode,
                        paths,
                    ),
                }
            }
        }
        Expression::ArrayLiteral(elements) => {
            for element in elements {
                collect_counted_paths_in_expression(
                    element,
                    global_labels,
                    constants,
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
                    path_mode,
                    paths,
                );
            }
        }
        Expression::FieldAccess { .. } => {
            if let Some(path) = expression.dotted_path() {
                if let Some(target) = path_mode.scoped_label_target(&path, global_labels) {
                    paths.visits.insert(target.to_string());
                } else if path_mode.is_flow_sibling_stitch(&path) {
                    paths
                        .visits
                        .insert(path_mode.resolve_single_stitch_target(&path));
                }
            }
        }
        Expression::IndexAccess { base, index } => {
            collect_counted_paths_in_expression(base, global_labels, constants, path_mode, paths);
            collect_counted_paths_in_expression(index, global_labels, constants, path_mode, paths);
        }
        Expression::MultipleCondition(args) => {
            for arg in args {
                collect_counted_paths_in_expression(
                    arg,
                    global_labels,
                    constants,
                    path_mode,
                    paths,
                );
            }
        }
        Expression::Binary { left, right, .. } => {
            collect_counted_paths_in_expression(left, global_labels, constants, path_mode, paths);
            collect_counted_paths_in_expression(right, global_labels, constants, path_mode, paths);
        }
        Expression::Unary { expression, .. } => {
            collect_counted_paths_in_expression(
                expression,
                global_labels,
                constants,
                path_mode,
                paths,
            );
        }
        Expression::DivertTarget(target) => {
            insert_counted_divert_target(target, global_labels, path_mode, paths, true, true);
        }
        Expression::String(_)
        | Expression::NumberInt(_)
        | Expression::NumberFloat(_)
        | Expression::NumberBool(_) => {}
    }
}

fn insert_counted_divert_target(
    target: &str,
    global_labels: &LabelIndex,
    path_mode: &ChoicePathMode,
    paths: &mut CountedFlowPaths,
    count_visits: bool,
    count_turns: bool,
) {
    let counted_target = path_mode
        .scoped_label_target(target, global_labels)
        .map(str::to_string)
        .or_else(|| {
            path_mode
                .is_flow_sibling_stitch(target)
                .then(|| path_mode.resolve_single_stitch_target(target))
        })
        .unwrap_or_else(|| target.to_string());
    if count_visits {
        paths.visits.insert(counted_target.clone());
    }
    if count_turns {
        paths.turns.insert(counted_target);
    }
}
