use std::collections::{HashMap, HashSet};

use crate::{
    analysis::CheckedStory,
    compiler::StageOutput,
    parsed::{
        BinaryOperator, Choice, Conditional, ContentList, Divert, DivertTarget, Expression,
        FloatLiteral, Flow, Object, Sequence, SequenceType, Story, Weave,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeProgram {
    pub root: Container,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub content: Vec<RuntimeObject>,
    pub name: Option<String>,
    pub flags: Option<i32>,
    pub merge_tail_metadata: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeObject {
    Container(Container),
    NamedContent(Vec<Container>),
    String(String),
    ControlCommand(ControlCommand),
    Divert { target: String, variable: bool },
    FunctionDivert { target: String },
    ConditionalDivert { target: String },
    DivertTarget(String),
    ReadCount(String),
    VariableAssignment(String),
    GlobalVariableAssignment(String),
    TempVariableReassignment(String),
    VariableReassignment(String),
    VariableReference(String),
    ChoicePoint { target: String, flags: i32 },
    Glue,
    Tag { is_start: bool },
    Bool(bool),
    Int(i32),
    Float(FloatLiteral),
    Void,
    NativeFunction(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlCommand {
    Done,
    End,
    EvalStart,
    EvalOutput,
    EvalEnd,
    BeginString,
    EndString,
    VisitIndex,
    SequenceShuffleIndex,
    Duplicate,
    NoOp,
    Pop,
    PopFunction,
}

enum ChoiceOuter {
    Inline(Vec<RuntimeObject>),
    Nested(Container),
}

#[derive(Debug, Default)]
struct CountedFlowPaths {
    visits: HashSet<String>,
    turns: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ChoicePathMode {
    Root,
    RootGather {
        gather_name: String,
    },
    NestedRoot {
        container_path: String,
        gather_target: String,
    },
    Flow {
        flow_name: String,
        container_path: String,
        parent_flow_name: Option<String>,
        sibling_stitch_names: Vec<String>,
        local_variables: HashSet<String>,
        self_target_relative: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GatherLocation {
    Main(Vec<usize>),
    Named(Vec<usize>),
}

impl GatherLocation {
    fn child(&self, child_index: usize) -> Self {
        match self {
            GatherLocation::Main(path) => {
                let mut path = path.clone();
                path.push(child_index);
                GatherLocation::Main(path)
            }
            GatherLocation::Named(path) => {
                let mut path = path.clone();
                path.push(child_index);
                GatherLocation::Named(path)
            }
        }
    }
}

impl ChoicePathMode {
    fn with_self_target_relative(&self, self_target_relative: bool) -> Self {
        match self {
            ChoicePathMode::Root => ChoicePathMode::Root,
            ChoicePathMode::RootGather { gather_name } => ChoicePathMode::RootGather {
                gather_name: gather_name.clone(),
            },
            ChoicePathMode::NestedRoot {
                container_path,
                gather_target,
            } => ChoicePathMode::NestedRoot {
                container_path: container_path.clone(),
                gather_target: gather_target.clone(),
            },
            ChoicePathMode::Flow {
                flow_name,
                container_path,
                parent_flow_name,
                sibling_stitch_names,
                local_variables,
                ..
            } => ChoicePathMode::Flow {
                flow_name: flow_name.clone(),
                container_path: container_path.clone(),
                parent_flow_name: parent_flow_name.clone(),
                sibling_stitch_names: sibling_stitch_names.clone(),
                local_variables: local_variables.clone(),
                self_target_relative,
            },
        }
    }

    fn for_choice_nested_content(
        &self,
        choice_container_name: &str,
        gather_container_name: &str,
        has_following_gather: bool,
    ) -> Self {
        match self {
            ChoicePathMode::Root => ChoicePathMode::NestedRoot {
                container_path: format!("0.{choice_container_name}"),
                gather_target: format!("0.{gather_container_name}"),
            },
            ChoicePathMode::RootGather { gather_name } => ChoicePathMode::NestedRoot {
                container_path: format!("0.{gather_name}.{choice_container_name}"),
                gather_target: format!("0.{gather_container_name}"),
            },
            ChoicePathMode::NestedRoot {
                container_path,
                gather_target,
            } => ChoicePathMode::NestedRoot {
                container_path: format!("{container_path}.{choice_container_name}"),
                gather_target: if has_following_gather {
                    format!("{container_path}.{gather_container_name}")
                } else {
                    gather_target.clone()
                },
            },
            ChoicePathMode::Flow { .. } => self.with_self_target_relative(true),
        }
    }

    fn for_nested_weave(&self, container_index: usize) -> Self {
        match self {
            ChoicePathMode::NestedRoot {
                container_path,
                gather_target,
            } => ChoicePathMode::NestedRoot {
                container_path: format!("{container_path}.{container_index}"),
                gather_target: gather_target.clone(),
            },
            ChoicePathMode::Root => ChoicePathMode::NestedRoot {
                container_path: format!("0.{container_index}"),
                gather_target: "0.g-0".to_string(),
            },
            ChoicePathMode::RootGather { gather_name } => ChoicePathMode::NestedRoot {
                container_path: format!("0.{gather_name}.{container_index}"),
                gather_target: "0.g-0".to_string(),
            },
            ChoicePathMode::Flow {
                flow_name,
                container_path,
                parent_flow_name,
                sibling_stitch_names,
                local_variables,
                self_target_relative,
            } => ChoicePathMode::Flow {
                flow_name: flow_name.clone(),
                container_path: format!("{container_path}.{container_index}"),
                parent_flow_name: parent_flow_name.clone(),
                sibling_stitch_names: sibling_stitch_names.clone(),
                local_variables: local_variables.clone(),
                self_target_relative: *self_target_relative,
            },
        }
    }

    fn for_gather(&self, gather_name: &str) -> Self {
        match self {
            ChoicePathMode::Root => ChoicePathMode::RootGather {
                gather_name: gather_name.to_string(),
            },
            ChoicePathMode::NestedRoot {
                container_path,
                gather_target,
            } => ChoicePathMode::NestedRoot {
                container_path: format!("{container_path}.{gather_name}"),
                gather_target: gather_target.clone(),
            },
            ChoicePathMode::Flow {
                flow_name,
                container_path,
                parent_flow_name,
                sibling_stitch_names,
                local_variables,
                self_target_relative,
            } => ChoicePathMode::Flow {
                flow_name: flow_name.clone(),
                container_path: format!("{container_path}.{gather_name}"),
                parent_flow_name: parent_flow_name.clone(),
                sibling_stitch_names: sibling_stitch_names.clone(),
                local_variables: local_variables.clone(),
                self_target_relative: *self_target_relative,
            },
            other => other.clone(),
        }
    }

    fn for_conditional_branch(&self, branch_index: usize) -> Self {
        let container_path = format!("{}.b", runtime_index_path(self, branch_index));
        match self {
            ChoicePathMode::Root | ChoicePathMode::RootGather { .. } => {
                ChoicePathMode::NestedRoot {
                    container_path,
                    gather_target: "0.g-0".to_string(),
                }
            }
            ChoicePathMode::NestedRoot { gather_target, .. } => ChoicePathMode::NestedRoot {
                container_path,
                gather_target: gather_target.clone(),
            },
            ChoicePathMode::Flow {
                flow_name,
                parent_flow_name,
                sibling_stitch_names,
                local_variables,
                ..
            } => ChoicePathMode::Flow {
                flow_name: flow_name.clone(),
                container_path,
                parent_flow_name: parent_flow_name.clone(),
                sibling_stitch_names: sibling_stitch_names.clone(),
                local_variables: local_variables.clone(),
                self_target_relative: true,
            },
        }
    }

    fn for_sequence_branch(&self, sequence_container_path: &str, branch_name: &str) -> Self {
        let container_path = format!("{sequence_container_path}.{branch_name}");
        match self {
            ChoicePathMode::Root | ChoicePathMode::RootGather { .. } => {
                ChoicePathMode::NestedRoot {
                    container_path,
                    gather_target: "0.g-0".to_string(),
                }
            }
            ChoicePathMode::NestedRoot { gather_target, .. } => ChoicePathMode::NestedRoot {
                container_path,
                gather_target: gather_target.clone(),
            },
            ChoicePathMode::Flow {
                flow_name,
                parent_flow_name,
                sibling_stitch_names,
                local_variables,
                self_target_relative,
                ..
            } => ChoicePathMode::Flow {
                flow_name: flow_name.clone(),
                container_path,
                parent_flow_name: parent_flow_name.clone(),
                sibling_stitch_names: sibling_stitch_names.clone(),
                local_variables: local_variables.clone(),
                self_target_relative: *self_target_relative,
            },
        }
    }

    fn fallback_gather_target(&self) -> Option<String> {
        match self {
            ChoicePathMode::NestedRoot { gather_target, .. } => Some(gather_target.clone()),
            _ => None,
        }
    }

    fn is_local_variable(&self, name: &str) -> bool {
        match self {
            ChoicePathMode::Flow {
                local_variables, ..
            } => local_variables.contains(name),
            _ => false,
        }
    }
}

pub(crate) fn lower(story: &CheckedStory, count_all_visits: bool) -> StageOutput<RuntimeProgram> {
    let global_labels = build_label_index(&story.parsed);
    let global_variables = build_global_variable_names(&story.parsed);
    let counted_flow_paths = build_counted_flow_paths(&story.parsed, &global_labels);
    let root_weave = story.parsed.root_weave();
    let main_content = lower_root_weave(
        root_weave,
        &global_labels,
        &global_variables,
        count_all_visits,
    );

    let main_container = RuntimeObject::Container(Container {
        content: main_content,
        name: None,
        flags: None,
        merge_tail_metadata: true,
    });

    let mut root_content = vec![
        main_container,
        RuntimeObject::ControlCommand(ControlCommand::Done),
    ];

    let mut named_containers = story
        .parsed
        .flows()
        .iter()
        .map(|flow| {
            lower_flow(
                flow,
                &global_labels,
                &global_variables,
                &counted_flow_paths,
                count_all_visits,
            )
        })
        .collect::<Vec<_>>();
    if let Some(global_declarations) = lower_global_declarations(&story.parsed, &global_labels) {
        named_containers.push(global_declarations);
    }
    if !named_containers.is_empty() {
        root_content.push(RuntimeObject::NamedContent(named_containers));
    }

    let root = Container {
        content: root_content,
        name: None,
        flags: count_all_visits.then_some(1),
        merge_tail_metadata: true,
    };

    StageOutput {
        artifact: Some(RuntimeProgram { root }),
        diagnostics: Vec::new(),
    }
}

fn lower_global_declarations(
    story: &Story,
    global_labels: &HashMap<String, String>,
) -> Option<Container> {
    let declarations = story
        .root_weave()
        .content()
        .iter()
        .filter_map(|object| match object {
            Object::VariableAssignment(assignment) if assignment.is_global() => Some(assignment),
            _ => None,
        })
        .collect::<Vec<_>>();

    if declarations.is_empty() {
        return None;
    }

    let choice_labels = HashMap::new();
    let mut content = vec![RuntimeObject::ControlCommand(ControlCommand::EvalStart)];
    for declaration in declarations {
        lower_expression_into(
            &mut content,
            declaration.expression(),
            &choice_labels,
            global_labels,
            &ChoicePathMode::Root,
            false,
        );
        content.push(RuntimeObject::GlobalVariableAssignment(
            declaration.name().to_string(),
        ));
    }
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    content.push(RuntimeObject::ControlCommand(ControlCommand::End));

    Some(Container {
        content,
        name: Some("global decl".to_string()),
        flags: None,
        merge_tail_metadata: true,
    })
}

fn build_global_variable_names(story: &Story) -> HashSet<String> {
    story
        .root_weave()
        .content()
        .iter()
        .filter_map(|object| match object {
            Object::VariableAssignment(assignment) if assignment.is_global() => {
                Some(assignment.name().to_string())
            }
            _ => None,
        })
        .collect()
}

fn build_counted_flow_paths(
    story: &Story,
    global_labels: &HashMap<String, String>,
) -> CountedFlowPaths {
    let mut paths = CountedFlowPaths::default();
    collect_counted_paths_in_weave(story.root_weave(), global_labels, &mut paths);
    for flow in story.flows() {
        collect_counted_paths_in_flow(flow, global_labels, &mut paths);
    }
    paths
}

fn collect_counted_paths_in_flow(
    flow: &Flow,
    global_labels: &HashMap<String, String>,
    paths: &mut CountedFlowPaths,
) {
    collect_counted_paths_in_weave(flow.weave(), global_labels, paths);
    for child in flow.child_flows() {
        collect_counted_paths_in_flow(child, global_labels, paths);
    }
}

fn collect_counted_paths_in_weave(
    weave: &Weave,
    global_labels: &HashMap<String, String>,
    paths: &mut CountedFlowPaths,
) {
    for object in weave.content() {
        collect_counted_paths_in_object(object, global_labels, paths);
    }
}

fn collect_counted_paths_in_content_list(
    content_list: &ContentList,
    global_labels: &HashMap<String, String>,
    paths: &mut CountedFlowPaths,
) {
    for object in content_list.objects() {
        collect_counted_paths_in_object(object, global_labels, paths);
    }
}

fn collect_counted_paths_in_object(
    object: &Object,
    global_labels: &HashMap<String, String>,
    paths: &mut CountedFlowPaths,
) {
    match object {
        Object::ContentList(content_list) => {
            collect_counted_paths_in_content_list(content_list, global_labels, paths);
        }
        Object::Expression(expression) | Object::LogicLine(expression) => {
            collect_counted_paths_in_expression(expression, global_labels, paths);
        }
        Object::Conditional(conditional) => {
            if let Some(condition) = conditional.initial_condition() {
                collect_counted_paths_in_expression(condition, global_labels, paths);
            }
            for branch in conditional.branches() {
                if let Some(condition) = branch.own_condition() {
                    collect_counted_paths_in_expression(condition, global_labels, paths);
                }
                collect_counted_paths_in_weave(branch.content(), global_labels, paths);
            }
        }
        Object::Choice(choice) => {
            if let Some(condition) = choice.condition() {
                collect_counted_paths_in_expression(condition, global_labels, paths);
            }
            if let Some(content) = choice.start_content() {
                collect_counted_paths_in_content_list(content, global_labels, paths);
            }
            if let Some(content) = choice.choice_only_content() {
                collect_counted_paths_in_content_list(content, global_labels, paths);
            }
            collect_counted_paths_in_content_list(choice.inner_content(), global_labels, paths);
        }
        Object::Divert(divert) => {
            for argument in divert.arguments() {
                collect_counted_paths_in_expression(argument, global_labels, paths);
            }
        }
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                collect_counted_paths_in_content_list(element, global_labels, paths);
            }
        }
        Object::VariableAssignment(assignment) => match assignment.expression() {
            Expression::DivertTarget(target) if assignment.is_global() => {
                paths.turns.insert(target.clone());
            }
            expression => collect_counted_paths_in_expression(expression, global_labels, paths),
        },
        Object::Return(ret) => {
            if let Some(expr) = ret.returned_expression() {
                collect_counted_paths_in_expression(expr, global_labels, paths);
            }
        }
        Object::Weave(weave) => collect_counted_paths_in_weave(weave, global_labels, paths),
        Object::Text(_)
        | Object::Glue(_)
        | Object::IncDec(_)
        | Object::Gather(_)
        | Object::Tag(_) => {}
    }
}

fn collect_counted_paths_in_expression(
    expression: &Expression,
    global_labels: &HashMap<String, String>,
    paths: &mut CountedFlowPaths,
) {
    match expression {
        Expression::VariableReference(name) => {
            if let Some(target) = global_labels.get(name) {
                paths.visits.insert(target.clone());
            }
        }
        Expression::FunctionCall { name, args } => {
            let count_turns = name == "TURNS_SINCE";
            let count_visits = name == "READ_COUNT";
            for arg in args {
                match arg {
                    Expression::DivertTarget(target) if count_turns => {
                        paths.turns.insert(target.clone());
                    }
                    Expression::DivertTarget(target) if count_visits => {
                        paths.visits.insert(target.clone());
                    }
                    _ => collect_counted_paths_in_expression(arg, global_labels, paths),
                }
            }
        }
        Expression::MultipleCondition(args) => {
            for arg in args {
                collect_counted_paths_in_expression(arg, global_labels, paths);
            }
        }
        Expression::Binary { left, right, .. } => {
            collect_counted_paths_in_expression(left, global_labels, paths);
            collect_counted_paths_in_expression(right, global_labels, paths);
        }
        Expression::Unary { expression, .. } => {
            collect_counted_paths_in_expression(expression, global_labels, paths);
        }
        Expression::String(_)
        | Expression::NumberInt(_)
        | Expression::NumberFloat(_)
        | Expression::NumberBool(_)
        | Expression::DivertTarget(_) => {}
    }
}

fn build_label_index(story: &Story) -> HashMap<String, String> {
    let mut labels = HashMap::new();
    collect_weave_labels(story.root_weave(), "0", &mut labels);
    for flow in story.flows() {
        collect_flow_labels(flow, None, &mut labels);
    }
    labels
}

fn collect_flow_labels(
    flow: &Flow,
    parent_flow_name: Option<&str>,
    labels: &mut HashMap<String, String>,
) {
    let flow_path = parent_flow_name
        .map(|parent| format!("{parent}.{}", flow.name()))
        .unwrap_or_else(|| flow.name().to_string());
    labels.insert(flow_path.clone(), flow_path.clone());
    if parent_flow_name.is_none() {
        labels.insert(flow.name().to_string(), flow_path.clone());
    }
    collect_weave_labels(flow.weave(), &format!("{flow_path}.0"), labels);
    for child in flow.child_flows() {
        collect_flow_labels(child, Some(&flow_path), labels);
    }
}

fn collect_weave_labels(weave: &Weave, container_path: &str, labels: &mut HashMap<String, String>) {
    let mut choice_count = 0;
    let mut gather_count = 0;
    for object in weave.content() {
        match object {
            Object::Choice(choice) => {
                if let Some(identifier) = choice.identifier() {
                    insert_label_aliases(
                        labels,
                        identifier,
                        container_path,
                        &format!("{container_path}.c-{choice_count}"),
                    );
                }
                choice_count += 1;
            }
            Object::Gather(gather) => {
                let gather_name = gather.identifier().map(str::to_string).unwrap_or_else(|| {
                    let name = format!("g-{gather_count}");
                    gather_count += 1;
                    name
                });
                if let Some(identifier) = gather.identifier() {
                    insert_label_aliases(
                        labels,
                        identifier,
                        container_path,
                        &format!("{container_path}.{gather_name}"),
                    );
                }
            }
            Object::Weave(weave) => {
                collect_weave_labels(weave, container_path, labels);
            }
            _ => {}
        }
    }
}

fn insert_label_aliases(
    labels: &mut HashMap<String, String>,
    identifier: &str,
    container_path: &str,
    target_path: &str,
) {
    labels.insert(identifier.to_string(), target_path.to_string());
    if let Some(flow_path) = container_path.strip_suffix(".0") {
        labels.insert(format!("{flow_path}.{identifier}"), target_path.to_string());
    }
}

fn lower_root_weave(
    weave: &Weave,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    count_all_visits: bool,
) -> Vec<RuntimeObject> {
    if weave_has_choice(weave) {
        lower_choice_weave(
            weave,
            ChoicePathMode::Root,
            global_labels,
            global_variables,
            count_all_visits,
        )
    } else {
        let mut content = lower_linear_weave(weave, global_labels, global_variables);
        content.push(RuntimeObject::Container(done_container(
            "g-0",
            count_all_visits,
        )));
        content
    }
}

fn lower_flow(
    flow: &Flow,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    counted_flow_paths: &CountedFlowPaths,
    count_all_visits: bool,
) -> Container {
    // Collect child stitch names upfront so knot-level choices can reference them
    let child_stitch_names: Vec<String> = flow
        .child_flows()
        .iter()
        .map(|f| f.name().to_string())
        .collect();
    lower_flow_with_context(
        flow,
        None,
        &child_stitch_names,
        global_labels,
        global_variables,
        counted_flow_paths,
        count_all_visits,
    )
}

fn lower_flow_with_context(
    flow: &Flow,
    parent_knot_name: Option<&str>,
    sibling_stitch_names: &[String],
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    counted_flow_paths: &CountedFlowPaths,
    count_all_visits: bool,
) -> Container {
    let mut content = Vec::new();
    let flow_path = parent_knot_name
        .map(|parent| format!("{parent}.{}", flow.name()))
        .unwrap_or_else(|| flow.name().to_string());
    let local_variables = collect_flow_local_variables(flow);

    lower_flow_arguments_into(&mut content, flow);

    // Lower any content in the flow's own weave
    if weave_has_choice(flow.weave()) {
        // For stitches inside a knot, pass the parent knot name and sibling stitch names
        let flow_container_path = parent_knot_name
            .map(|parent| format!("{parent}.{}.0", flow.name()))
            .unwrap_or_else(|| format!("{}.0", flow.name()));
        let path_mode = ChoicePathMode::Flow {
            flow_name: flow.name().to_string(),
            container_path: flow_container_path,
            parent_flow_name: parent_knot_name.map(|s| s.to_string()),
            sibling_stitch_names: sibling_stitch_names.to_vec(),
            local_variables: local_variables.clone(),
            self_target_relative: false,
        };
        content.push(RuntimeObject::Container(Container {
            content: lower_choice_weave(
                flow.weave(),
                path_mode,
                global_labels,
                global_variables,
                count_all_visits,
            ),
            name: None,
            flags: None,
            merge_tail_metadata: true,
        }));
    } else if !flow.weave().content().is_empty() {
        let path_mode = ChoicePathMode::Flow {
            flow_name: flow.name().to_string(),
            container_path: flow_path.clone(),
            parent_flow_name: parent_knot_name.map(str::to_string),
            sibling_stitch_names: sibling_stitch_names.to_vec(),
            local_variables,
            self_target_relative: false,
        };
        lower_linear_weave_into_context(
            &mut content,
            flow.weave(),
            global_labels,
            global_variables,
            &path_mode,
        );
    }

    // Lower child flows (stitches)
    if !flow.child_flows().is_empty() {
        // Only add auto-divert to first child flow if the knot's weave doesn't have choices
        // When choices are present, they explicitly divert to stitches
        let weave_has_choices = weave_has_choice(flow.weave());
        if !weave_has_choices && !ends_with_flow_terminator(&content) {
            let first_child_name = flow.child_flows()[0].name();
            content.push(RuntimeObject::Divert {
                target: format!(".^.{}", first_child_name),
                variable: false,
            });
        }

        // Collect all child stitch names for sibling reference
        let child_stitch_names: Vec<String> = flow
            .child_flows()
            .iter()
            .map(|f| f.name().to_string())
            .collect();

        // Lower each child flow as named content, passing sibling stitch names
        let child_containers: Vec<Container> = flow
            .child_flows()
            .iter()
            .map(|child| {
                lower_flow_with_context(
                    child,
                    Some(flow.name()),
                    &child_stitch_names,
                    global_labels,
                    global_variables,
                    counted_flow_paths,
                    count_all_visits,
                )
            })
            .collect();
        content.push(RuntimeObject::NamedContent(child_containers));
    }

    Container {
        content,
        name: Some(flow.name().to_string()),
        flags: flow_container_flags(
            counted_flow_paths.turns.contains(&flow_path),
            counted_flow_paths.visits.contains(&flow_path),
            count_all_visits,
        ),
        merge_tail_metadata: true,
    }
}

fn flow_container_flags(
    count_turns: bool,
    count_visits: bool,
    count_all_visits: bool,
) -> Option<i32> {
    match (count_turns, count_visits || count_all_visits) {
        (true, _) => Some(3),
        (false, true) => Some(1),
        (false, false) => None,
    }
}

fn lower_flow_arguments_into(content: &mut Vec<RuntimeObject>, flow: &Flow) {
    for argument in flow.arguments().iter().rev() {
        content.push(RuntimeObject::VariableAssignment(
            argument.name().to_string(),
        ));
    }
}

fn collect_flow_local_variables(flow: &Flow) -> HashSet<String> {
    let mut local_variables = flow
        .arguments()
        .iter()
        .map(|argument| argument.name().to_string())
        .collect::<HashSet<_>>();
    collect_local_variables_in_weave(flow.weave(), &mut local_variables);
    local_variables
}

fn collect_local_variables_in_weave(weave: &Weave, local_variables: &mut HashSet<String>) {
    for object in weave.content() {
        collect_local_variables_in_object(object, local_variables);
    }
}

fn collect_local_variables_in_content_list(
    content_list: &ContentList,
    local_variables: &mut HashSet<String>,
) {
    for object in content_list.objects() {
        collect_local_variables_in_object(object, local_variables);
    }
}

fn collect_local_variables_in_object(object: &Object, local_variables: &mut HashSet<String>) {
    match object {
        Object::VariableAssignment(assignment) if assignment.is_temporary() => {
            local_variables.insert(assignment.name().to_string());
        }
        Object::ContentList(content_list) => {
            collect_local_variables_in_content_list(content_list, local_variables);
        }
        Object::Conditional(conditional) => {
            for branch in conditional.branches() {
                collect_local_variables_in_weave(branch.content(), local_variables);
            }
        }
        Object::Choice(choice) => {
            if let Some(content) = choice.start_content() {
                collect_local_variables_in_content_list(content, local_variables);
            }
            if let Some(content) = choice.choice_only_content() {
                collect_local_variables_in_content_list(content, local_variables);
            }
            collect_local_variables_in_content_list(choice.inner_content(), local_variables);
        }
        Object::Sequence(sequence) => {
            for element in sequence.elements() {
                collect_local_variables_in_content_list(element, local_variables);
            }
        }
        Object::Weave(weave) => collect_local_variables_in_weave(weave, local_variables),
        _ => {}
    }
}

fn weave_has_choice(weave: &Weave) -> bool {
    weave
        .content()
        .iter()
        .any(|object| matches!(object, Object::Choice(_)))
}

fn content_list_has_choice(content_list: &ContentList) -> bool {
    content_list
        .objects()
        .iter()
        .any(|object| matches!(object, Object::Choice(_)))
}

fn lower_linear_weave(
    weave: &Weave,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
) -> Vec<RuntimeObject> {
    lower_linear_weave_with_context(
        weave,
        global_labels,
        global_variables,
        &ChoicePathMode::Root,
    )
}

fn lower_linear_weave_with_context(
    weave: &Weave,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    path_mode: &ChoicePathMode,
) -> Vec<RuntimeObject> {
    let mut content = Vec::new();
    lower_linear_weave_into_context(
        &mut content,
        weave,
        global_labels,
        global_variables,
        path_mode,
    );
    content
}

fn lower_linear_weave_into_context(
    content: &mut Vec<RuntimeObject>,
    weave: &Weave,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    path_mode: &ChoicePathMode,
) {
    let choice_labels = HashMap::new();
    for object in weave.content() {
        lower_object_into_with_context(
            content,
            object,
            path_mode,
            &choice_labels,
            global_labels,
            global_variables,
        );
    }
}

fn lower_choice_weave(
    weave: &Weave,
    path_mode: ChoicePathMode,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    count_all_visits: bool,
) -> Vec<RuntimeObject> {
    lower_choice_weave_with_initial_content(
        weave,
        path_mode,
        global_labels,
        global_variables,
        count_all_visits,
        Vec::new(),
    )
}

fn lower_choice_weave_with_initial_content(
    weave: &Weave,
    path_mode: ChoicePathMode,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    count_all_visits: bool,
    initial_content: Vec<RuntimeObject>,
) -> Vec<RuntimeObject> {
    let mut main_content = initial_content;
    let mut named_content = Vec::new();
    let mut index = 0;
    let mut gather_count = 0;
    let mut choice_count = 0;
    let mut needs_terminal_gather = false;
    let mut last_gather_location = None;
    let mut last_section_had_choice = false;
    let objects = weave.content();
    let mut choice_labels = collect_local_weave_labels(objects);

    // Check if there's an explicit gather anywhere in the weave
    let has_explicit_gather = objects.iter().any(|o| matches!(o, Object::Gather(_)));

    while index < objects.len() {
        match &objects[index] {
            Object::Text(_)
            | Object::ContentList(_)
            | Object::Expression(_)
            | Object::Conditional(_)
            | Object::LogicLine(_)
            | Object::Glue(_)
            | Object::Divert(_)
            | Object::Tag(_)
            | Object::Sequence(_)
            | Object::IncDec(_)
            | Object::VariableAssignment(_)
            | Object::Return(_)
            | Object::Weave(_) => {
                last_section_had_choice = lower_weave_section(
                    objects,
                    &mut index,
                    &mut main_content,
                    &mut named_content,
                    &mut choice_count,
                    &mut needs_terminal_gather,
                    &mut choice_labels,
                    global_labels,
                    global_variables,
                    gather_count,
                    &path_mode,
                    has_explicit_gather,
                    count_all_visits,
                );
            }
            Object::Gather(_gather) => {
                // Create a named container for the gather
                let gather = match &objects[index] {
                    Object::Gather(gather) => gather,
                    _ => unreachable!(),
                };
                let gather_name = if let Some(identifier) = gather.identifier() {
                    identifier.to_string()
                } else {
                    let gather_name = format!("g-{gather_count}");
                    gather_count += 1;
                    gather_name
                };
                if let Some(identifier) = gather.identifier() {
                    choice_labels.insert(identifier.to_string(), gather_name.clone());
                }
                let auto_enter_gather = !last_section_had_choice;

                let mut gather_content = Vec::new();
                let mut gather_named_content = Vec::new();
                index += 1;

                let gather_path_mode = path_mode.for_gather(&gather_name);
                let gather_has_choice = lower_weave_section(
                    objects,
                    &mut index,
                    &mut gather_content,
                    &mut gather_named_content,
                    &mut choice_count,
                    &mut needs_terminal_gather,
                    &mut choice_labels,
                    global_labels,
                    global_variables,
                    gather_count,
                    &gather_path_mode,
                    has_explicit_gather,
                    count_all_visits,
                );
                if !gather_named_content.is_empty() {
                    gather_content.push(RuntimeObject::NamedContent(gather_named_content));
                }
                if matches!(path_mode, ChoicePathMode::NestedRoot { .. })
                    && !gather_has_choice
                    && !ends_with_flow_terminator(&gather_content)
                {
                    if let Some(target) = gather_path_mode.fallback_gather_target() {
                        gather_content.push(RuntimeObject::Divert {
                            target,
                            variable: false,
                        });
                    }
                }

                let gather_container = Container {
                    content: gather_content,
                    name: Some(gather_name),
                    flags: if count_all_visits || gather.identifier().is_some() {
                        Some(5)
                    } else {
                        None
                    },
                    merge_tail_metadata: true,
                };
                if auto_enter_gather {
                    if let Some(location) = last_gather_location.clone() {
                        if let Some(parent) = gather_container_at_location_mut(
                            &mut main_content,
                            &mut named_content,
                            &location,
                        ) {
                            let child_index = parent.content.len();
                            parent
                                .content
                                .push(RuntimeObject::Container(gather_container));
                            last_gather_location = Some(location.child(child_index));
                        }
                    } else {
                        let child_index = main_content.len();
                        main_content.push(RuntimeObject::Container(gather_container));
                        last_gather_location = Some(GatherLocation::Main(vec![child_index]));
                    }
                } else {
                    named_content.push(gather_container);
                    last_gather_location =
                        Some(GatherLocation::Named(vec![named_content.len() - 1]));
                }
                last_section_had_choice = gather_has_choice;
            }
            Object::Choice(_) => {
                last_section_had_choice = lower_weave_section(
                    objects,
                    &mut index,
                    &mut main_content,
                    &mut named_content,
                    &mut choice_count,
                    &mut needs_terminal_gather,
                    &mut choice_labels,
                    global_labels,
                    global_variables,
                    gather_count,
                    &path_mode,
                    has_explicit_gather,
                    count_all_visits,
                );
            }
        }
    }

    if matches!(path_mode, ChoicePathMode::Root) && has_explicit_gather {
        if let Some(location) = last_gather_location {
            if let Some(container) =
                gather_container_at_location_mut(&mut main_content, &mut named_content, &location)
            {
                container
                    .content
                    .push(RuntimeObject::Container(done_container(
                        &format!("g-{gather_count}"),
                        count_all_visits,
                    )));
            }
        }
    } else if !matches!(path_mode, ChoicePathMode::NestedRoot { .. })
        && !has_explicit_gather
        && needs_terminal_gather
    {
        // Add a terminal gather for choices that divert to it.
        named_content.push(done_container(
            &format!("g-{gather_count}"),
            count_all_visits,
        ));
    }
    if !named_content.is_empty() {
        main_content.push(RuntimeObject::NamedContent(named_content));
    }

    main_content
}

fn lower_weave_section(
    objects: &[Object],
    index: &mut usize,
    content: &mut Vec<RuntimeObject>,
    named_content: &mut Vec<Container>,
    choice_count: &mut usize,
    needs_terminal_gather: &mut bool,
    choice_labels: &mut HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    gather_count: usize,
    path_mode: &ChoicePathMode,
    has_explicit_gather: bool,
    count_all_visits: bool,
) -> bool {
    let mut section_has_choice = false;
    while *index < objects.len() {
        match &objects[*index] {
            Object::Gather(_) => break,
            Object::Text(_)
            | Object::ContentList(_)
            | Object::Expression(_)
            | Object::Conditional(_)
            | Object::LogicLine(_)
            | Object::Glue(_)
            | Object::Divert(_)
            | Object::Tag(_)
            | Object::Sequence(_)
            | Object::IncDec(_)
            | Object::VariableAssignment(_)
            | Object::Return(_)
            | Object::Weave(_) => {
                lower_object_into_with_context(
                    content,
                    &objects[*index],
                    path_mode,
                    choice_labels,
                    global_labels,
                    global_variables,
                );
                *index += 1;
            }
            Object::Choice(_) => {
                section_has_choice = true;
                lower_choice_in_section(
                    objects,
                    index,
                    content,
                    named_content,
                    choice_count,
                    needs_terminal_gather,
                    choice_labels,
                    global_labels,
                    global_variables,
                    gather_count,
                    path_mode,
                    has_explicit_gather,
                    count_all_visits,
                );
            }
        }
    }
    section_has_choice
}

fn gather_container_at_location_mut<'a>(
    main_content: &'a mut Vec<RuntimeObject>,
    named_content: &'a mut [Container],
    location: &GatherLocation,
) -> Option<&'a mut Container> {
    match location {
        GatherLocation::Main(path) => nested_container_in_runtime_content_mut(main_content, path),
        GatherLocation::Named(path) => nested_container_in_named_content_mut(named_content, path),
    }
}

fn nested_container_in_named_content_mut<'a>(
    named_content: &'a mut [Container],
    path: &[usize],
) -> Option<&'a mut Container> {
    let (&first, rest) = path.split_first()?;
    let container = named_content.get_mut(first)?;
    nested_container_in_container_mut(container, rest)
}

fn nested_container_in_runtime_content_mut<'a>(
    content: &'a mut Vec<RuntimeObject>,
    path: &[usize],
) -> Option<&'a mut Container> {
    let (&first, rest) = path.split_first()?;
    let RuntimeObject::Container(container) = content.get_mut(first)? else {
        return None;
    };
    nested_container_in_container_mut(container, rest)
}

fn nested_container_in_container_mut<'a>(
    container: &'a mut Container,
    path: &[usize],
) -> Option<&'a mut Container> {
    let Some((&first, rest)) = path.split_first() else {
        return Some(container);
    };
    let RuntimeObject::Container(child) = container.content.get_mut(first)? else {
        return None;
    };
    nested_container_in_container_mut(child, rest)
}

fn lower_choice_in_section(
    objects: &[Object],
    index: &mut usize,
    content: &mut Vec<RuntimeObject>,
    named_content: &mut Vec<Container>,
    choice_count: &mut usize,
    needs_terminal_gather: &mut bool,
    choice_labels: &mut HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    gather_count: usize,
    path_mode: &ChoicePathMode,
    has_explicit_gather: bool,
    count_all_visits: bool,
) {
    let Object::Choice(choice) = &objects[*index] else {
        return;
    };

    let choice_index = *choice_count;
    let has_following_gather = objects[*index + 1..]
        .iter()
        .any(|object| matches!(object, Object::Gather(_)));
    *choice_count += 1;
    let choice_container_name = format!("c-{choice_index}");
    let gather_container_name = next_gather_name(objects, *index + 1, gather_count);
    let choice_container_path =
        choice_point_target(path_mode, choice.has_start_content(), choice_index);

    match choice_outer(
        choice,
        &choice_container_path,
        content.len(),
        path_mode,
        choice_labels,
        global_labels,
        global_variables,
    ) {
        ChoiceOuter::Inline(objects) => content.extend(objects),
        ChoiceOuter::Nested(container) => content.push(RuntimeObject::Container(container)),
    }

    let mut choice_content = Vec::new();
    let choice_content_path_mode = path_mode.with_self_target_relative(true);
    if choice.has_start_content() {
        choice_content =
            choice_container_prefix(path_mode, &choice_container_name, content.len() - 1, 2);
    }
    lower_content_list_into_context(
        &mut choice_content,
        choice.inner_content(),
        &choice_content_path_mode,
        choice_labels,
        global_labels,
        global_variables,
    );
    let nested_choice_content_path_mode = path_mode.for_choice_nested_content(
        &choice_container_name,
        &gather_container_name,
        has_following_gather,
    );
    let mut has_nested_weave_content = false;

    *index += 1;
    while *index < objects.len() {
        if matches!(objects[*index], Object::Choice(_) | Object::Gather(_)) {
            break;
        }
        if matches!(objects[*index], Object::Weave(_)) {
            has_nested_weave_content = true;
        }
        lower_object_into_with_context(
            &mut choice_content,
            &objects[*index],
            &nested_choice_content_path_mode,
            choice_labels,
            global_labels,
            global_variables,
        );
        *index += 1;
    }

    let include_gather = if has_nested_weave_content {
        false
    } else if has_explicit_gather {
        true
    } else {
        match path_mode {
            ChoicePathMode::Root
            | ChoicePathMode::RootGather { .. }
            | ChoicePathMode::NestedRoot { .. } => true,
            ChoicePathMode::Flow { .. } => false,
        }
    };
    if include_gather {
        choice_content.push(RuntimeObject::Divert {
            target: gather_target(path_mode, &gather_container_name, has_following_gather),
            variable: false,
        });
        *needs_terminal_gather = true;
    }

    named_content.push(Container {
        content: choice_content,
        name: Some(choice_container_name),
        // Only set visitsShouldBeCounted flag (5) for once-only choices.
        flags: if count_all_visits || choice.once_only() {
            Some(5)
        } else {
            None
        },
        merge_tail_metadata: true,
    });
    if let Some(identifier) = choice.identifier() {
        choice_labels.insert(identifier.to_string(), format!("c-{choice_index}"));
    }
}

fn collect_local_weave_labels(objects: &[Object]) -> HashMap<String, String> {
    let mut labels = HashMap::new();
    let mut choice_count = 0;
    let mut gather_count = 0;
    for object in objects {
        match object {
            Object::Choice(choice) => {
                if let Some(identifier) = choice.identifier() {
                    labels.insert(identifier.to_string(), format!("c-{choice_count}"));
                }
                choice_count += 1;
            }
            Object::Gather(gather) => {
                let gather_name = gather.identifier().map(str::to_string).unwrap_or_else(|| {
                    let name = format!("g-{gather_count}");
                    gather_count += 1;
                    name
                });
                if let Some(identifier) = gather.identifier() {
                    labels.insert(identifier.to_string(), gather_name);
                }
            }
            _ => {}
        }
    }
    labels
}

fn next_gather_name(objects: &[Object], start_index: usize, unnamed_gather_count: usize) -> String {
    objects[start_index..]
        .iter()
        .find_map(|object| match object {
            Object::Gather(gather) => Some(
                gather
                    .identifier()
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("g-{unnamed_gather_count}")),
            ),
            _ => None,
        })
        .unwrap_or_else(|| format!("g-{unnamed_gather_count}"))
}

fn choice_outer(
    choice: &Choice,
    choice_container_path: &str,
    choice_point_index: usize,
    path_mode: &ChoicePathMode,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
) -> ChoiceOuter {
    let mut outer_content = Vec::new();
    let has_eval_content = choice.has_start_content()
        || choice.has_choice_only_content()
        || choice.condition().is_some();

    if has_eval_content {
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    }

    if choice.has_start_content() {
        outer_content.push(RuntimeObject::DivertTarget(format!(
            "{}.$r1",
            outer_return_target(path_mode, choice_point_index)
        )));
        outer_content.push(RuntimeObject::VariableAssignment("$r".to_string()));
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::BeginString));
        outer_content.push(RuntimeObject::Divert {
            target: ".^.s".to_string(),
            variable: false,
        });
        outer_content.push(RuntimeObject::Container(Container {
            content: Vec::new(),
            name: Some("$r1".to_string()),
            flags: None,
            merge_tail_metadata: true,
        }));
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::EndString));
    }

    if let Some(choice_only_content) = choice.choice_only_content() {
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::BeginString));
        lower_content_list_into_context(
            &mut outer_content,
            choice_only_content,
            path_mode,
            choice_labels,
            global_labels,
            global_variables,
        );
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::EndString));
    }

    if let Some(condition) = choice.condition() {
        lower_expression_into(
            &mut outer_content,
            condition,
            choice_labels,
            global_labels,
            path_mode,
            choice.has_start_content(),
        );
    }

    if has_eval_content {
        outer_content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    }

    outer_content.push(RuntimeObject::ChoicePoint {
        target: choice_container_path.to_string(),
        flags: choice.choice_flags(),
    });

    if !choice.has_start_content() {
        return ChoiceOuter::Inline(outer_content);
    }

    let mut start_content = choice
        .start_content()
        .map(|cl| {
            lower_content_list_with_context(
                cl,
                path_mode,
                choice_labels,
                global_labels,
                global_variables,
            )
        })
        .unwrap_or_default();
    start_content.push(RuntimeObject::Divert {
        target: "$r".to_string(),
        variable: true,
    });
    outer_content.push(RuntimeObject::NamedContent(vec![Container {
        content: start_content,
        name: Some("s".to_string()),
        flags: None,
        merge_tail_metadata: true,
    }]));

    ChoiceOuter::Nested(Container {
        content: outer_content,
        name: None,
        flags: None,
        merge_tail_metadata: true,
    })
}

fn choice_container_prefix(
    path_mode: &ChoicePathMode,
    choice_container_name: &str,
    choice_point_index: usize,
    return_index: usize,
) -> Vec<RuntimeObject> {
    vec![
        RuntimeObject::ControlCommand(ControlCommand::EvalStart),
        RuntimeObject::DivertTarget(format!(
            "{}.$r{return_index}",
            choice_content_return_target(path_mode, choice_container_name)
        )),
        RuntimeObject::ControlCommand(ControlCommand::EvalEnd),
        RuntimeObject::VariableAssignment("$r".to_string()),
        RuntimeObject::Divert {
            target: start_content_target(path_mode, choice_point_index),
            variable: false,
        },
        RuntimeObject::Container(Container {
            content: Vec::new(),
            name: Some(format!("$r{return_index}")),
            flags: None,
            merge_tail_metadata: true,
        }),
    ]
}

fn lower_content_list_with_context(
    content_list: &ContentList,
    path_mode: &ChoicePathMode,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
) -> Vec<RuntimeObject> {
    let mut content = Vec::new();
    lower_content_list_into_context(
        &mut content,
        content_list,
        path_mode,
        choice_labels,
        global_labels,
        global_variables,
    );
    content
}

fn lower_content_list_into_context(
    content: &mut Vec<RuntimeObject>,
    content_list: &ContentList,
    path_mode: &ChoicePathMode,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
) {
    for object in content_list.objects() {
        lower_object_into_with_context(
            content,
            object,
            path_mode,
            choice_labels,
            global_labels,
            global_variables,
        );
    }
}

fn lower_expression_into(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    path_mode: &ChoicePathMode,
    has_start_content: bool,
) {
    match expression {
        Expression::String(value) => {
            content.push(RuntimeObject::ControlCommand(ControlCommand::BeginString));
            content.push(RuntimeObject::String(value.clone()));
            content.push(RuntimeObject::ControlCommand(ControlCommand::EndString));
        }
        Expression::NumberInt(value) => content.push(RuntimeObject::Int(*value)),
        Expression::NumberFloat(value) => content.push(RuntimeObject::Float(*value)),
        Expression::NumberBool(value) => content.push(RuntimeObject::Bool(*value)),
        Expression::DivertTarget(target) => content.push(RuntimeObject::DivertTarget(
            resolve_divert_target(target, path_mode),
        )),
        Expression::VariableReference(name) => {
            if let Some(choice_container_name) = choice_labels.get(name) {
                content.push(RuntimeObject::ReadCount(choice_label_count_target(
                    path_mode,
                    choice_container_name,
                    has_start_content,
                )));
            } else if let Some(label_target) = global_labels.get(name) {
                content.push(RuntimeObject::ReadCount(label_target.clone()));
            } else {
                content.push(RuntimeObject::VariableReference(name.clone()));
            }
        }
        Expression::FunctionCall { name, args } => {
            for arg in args {
                lower_expression_into(
                    content,
                    arg,
                    choice_labels,
                    global_labels,
                    path_mode,
                    has_start_content,
                );
            }
            if is_builtin_function(name) {
                content.push(RuntimeObject::NativeFunction(
                    function_runtime_name(name).to_string(),
                ));
            } else {
                content.push(RuntimeObject::FunctionDivert {
                    target: name.clone(),
                });
            }
        }
        Expression::Binary {
            operator,
            left,
            right,
        } => {
            lower_expression_into(
                content,
                left,
                choice_labels,
                global_labels,
                path_mode,
                has_start_content,
            );
            lower_expression_into(
                content,
                right,
                choice_labels,
                global_labels,
                path_mode,
                has_start_content,
            );
            content.push(RuntimeObject::NativeFunction(
                operator_runtime_name(*operator).to_string(),
            ));
        }
        Expression::Unary {
            operator,
            expression,
        } => {
            lower_expression_into(
                content,
                expression,
                choice_labels,
                global_labels,
                path_mode,
                has_start_content,
            );
            content.push(RuntimeObject::NativeFunction(
                operator.runtime_name().to_string(),
            ));
        }
        Expression::MultipleCondition(expressions) => {
            for (index, expression) in expressions.iter().enumerate() {
                lower_expression_into(
                    content,
                    expression,
                    choice_labels,
                    global_labels,
                    path_mode,
                    has_start_content,
                );
                if index > 0 {
                    content.push(RuntimeObject::NativeFunction("&&".to_string()));
                }
            }
        }
    }
}

fn lower_output_expression_into(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    path_mode: &ChoicePathMode,
) {
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    lower_expression_into(
        content,
        expression,
        choice_labels,
        global_labels,
        path_mode,
        false,
    );
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalOutput));
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
}

fn lower_logic_line_into(
    content: &mut Vec<RuntimeObject>,
    expression: &Expression,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    path_mode: &ChoicePathMode,
) {
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    lower_expression_into(
        content,
        expression,
        choice_labels,
        global_labels,
        path_mode,
        false,
    );
    content.push(RuntimeObject::ControlCommand(ControlCommand::Pop));
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    content.push(RuntimeObject::String("\n".to_string()));
}

fn operator_runtime_name(operator: BinaryOperator) -> &'static str {
    operator.runtime_name()
}

fn function_runtime_name(name: &str) -> &str {
    match name {
        "RANDOM" => "rnd",
        "SEED_RANDOM" => "srnd",
        other => other,
    }
}

fn is_builtin_function(name: &str) -> bool {
    matches!(
        name,
        "RANDOM"
            | "SEED_RANDOM"
            | "CHOICE_COUNT"
            | "TURNS"
            | "TURNS_SINCE"
            | "READ_COUNT"
            | "LIST_RANGE"
            | "LIST_RANDOM"
            | "LIST_VALUE"
            | "MIN"
            | "MAX"
            | "POW"
            | "FLOOR"
            | "CEILING"
            | "INT"
            | "FLOAT"
            | "LIST_MIN"
            | "LIST_MAX"
            | "LIST_ALL"
            | "LIST_COUNT"
            | "LIST_INVERT"
    )
}

fn lower_sequence(
    sequence: &Sequence,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    path_mode: &ChoicePathMode,
    sequence_container_path: &str,
) -> Container {
    let mut content = vec![
        RuntimeObject::ControlCommand(ControlCommand::EvalStart),
        RuntimeObject::ControlCommand(ControlCommand::VisitIndex),
    ];

    let sequence_type = sequence.sequence_type();
    let once = sequence_type.contains(SequenceType::ONCE);
    let cycle = sequence_type.contains(SequenceType::CYCLE);
    let stopping = sequence_type.contains(SequenceType::STOPPING);
    let shuffle = sequence_type.contains(SequenceType::SHUFFLE);
    let branch_count = sequence.elements().len() + usize::from(once);

    if stopping || once {
        content.push(RuntimeObject::Int(branch_count.saturating_sub(1) as i32));
        content.push(RuntimeObject::NativeFunction("MIN".to_string()));
    } else if cycle {
        content.push(RuntimeObject::Int(sequence.elements().len() as i32));
        content.push(RuntimeObject::NativeFunction("%".to_string()));
    }

    if shuffle {
        if once || stopping {
            let last_index = if stopping {
                sequence.elements().len().saturating_sub(1)
            } else {
                sequence.elements().len()
            };
            let post_shuffle_noop_index = content.len() + 6;
            content.extend([
                RuntimeObject::ControlCommand(ControlCommand::Duplicate),
                RuntimeObject::Int(last_index as i32),
                RuntimeObject::NativeFunction("==".to_string()),
                RuntimeObject::ConditionalDivert {
                    target: format!(".^.{post_shuffle_noop_index}"),
                },
            ]);
        }

        let element_count_to_shuffle = sequence.elements().len() - usize::from(stopping);
        content.push(RuntimeObject::Int(element_count_to_shuffle as i32));
        content.push(RuntimeObject::ControlCommand(
            ControlCommand::SequenceShuffleIndex,
        ));
        if once || stopping {
            content.push(RuntimeObject::ControlCommand(ControlCommand::NoOp));
        }
    }

    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));

    for index in 0..branch_count {
        content.extend([
            RuntimeObject::ControlCommand(ControlCommand::EvalStart),
            RuntimeObject::ControlCommand(ControlCommand::Duplicate),
            RuntimeObject::Int(index as i32),
            RuntimeObject::NativeFunction("==".to_string()),
            RuntimeObject::ControlCommand(ControlCommand::EvalEnd),
            RuntimeObject::ConditionalDivert {
                target: format!(".^.s{index}"),
            },
        ]);
    }

    let post_sequence_index = content.len();
    content.push(RuntimeObject::ControlCommand(ControlCommand::NoOp));

    let branch_containers = sequence
        .elements()
        .iter()
        .map(Some)
        .chain((branch_count > sequence.elements().len()).then_some(None))
        .enumerate()
        .map(|(index, element)| {
            let branch_name = format!("s{index}");
            let branch_path_mode =
                path_mode.for_sequence_branch(sequence_container_path, &branch_name);
            let mut branch_content = vec![RuntimeObject::ControlCommand(ControlCommand::Pop)];
            if let Some(element) = element {
                if content_list_has_choice(element) {
                    let element_weave = Weave::new(element.objects().to_vec(), 0);
                    branch_content = lower_choice_weave_with_initial_content(
                        &element_weave,
                        branch_path_mode.clone(),
                        global_labels,
                        global_variables,
                        false,
                        branch_content,
                    );
                } else {
                    lower_content_list_into_context(
                        &mut branch_content,
                        element,
                        &branch_path_mode,
                        choice_labels,
                        global_labels,
                        global_variables,
                    );
                }
            }
            let relative_return_target = format!(".^.^.{post_sequence_index}");
            let global_return_target = format!("{sequence_container_path}.{post_sequence_index}");
            let trailing_named_content =
                if matches!(branch_content.last(), Some(RuntimeObject::NamedContent(_))) {
                    branch_content.pop()
                } else {
                    None
                };
            branch_content.push(RuntimeObject::Divert {
                target: compact_relative_path(&relative_return_target, &global_return_target),
                variable: false,
            });
            if let Some(named_content) = trailing_named_content {
                branch_content.push(named_content);
            }
            Container {
                content: branch_content,
                name: Some(branch_name),
                flags: None,
                merge_tail_metadata: true,
            }
        })
        .collect::<Vec<_>>();
    content.push(RuntimeObject::NamedContent(branch_containers));

    Container {
        content,
        name: None,
        flags: Some(5),
        merge_tail_metadata: true,
    }
}

fn lower_conditional_into(
    content: &mut Vec<RuntimeObject>,
    conditional: &Conditional,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
    path_mode: &ChoicePathMode,
) {
    if let Some(condition) = conditional.initial_condition() {
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
        lower_expression_into(
            content,
            condition,
            choice_labels,
            global_labels,
            path_mode,
            false,
        );
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    }

    let rejoin_index = content.len() + conditional.branches().len();
    let rejoin_target = runtime_index_path(path_mode, rejoin_index);
    let branch_rejoin_target =
        compact_relative_path(&format!(".^.^.^.{rejoin_index}"), &rejoin_target);
    let has_initial_condition = conditional.initial_condition().is_some();

    for branch in conditional.branches() {
        let branch_path_mode = path_mode.for_conditional_branch(content.len());
        let mut branch_content = Vec::new();
        if !has_initial_condition {
            if let Some(condition) = branch.own_condition() {
                branch_content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
                lower_expression_into(
                    &mut branch_content,
                    condition,
                    choice_labels,
                    global_labels,
                    path_mode,
                    false,
                );
                branch_content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
            }
        }

        if branch.is_else() {
            branch_content.push(RuntimeObject::Divert {
                target: ".^.b".to_string(),
                variable: false,
            });
        } else {
            branch_content.push(RuntimeObject::ConditionalDivert {
                target: ".^.b".to_string(),
            });
        }

        if weave_has_choice(branch.content()) {
            let initial_content = if branch.is_inline() {
                Vec::new()
            } else {
                vec![RuntimeObject::String("\n".to_string())]
            };
            let mut content_container = Vec::new();
            let mut lowered_branch = lower_choice_weave_with_initial_content(
                branch.content(),
                branch_path_mode,
                global_labels,
                global_variables,
                false,
                initial_content,
            );
            let trailing_named_content =
                if matches!(lowered_branch.last(), Some(RuntimeObject::NamedContent(_))) {
                    lowered_branch.pop()
                } else {
                    None
                };
            content_container.extend(lowered_branch);
            content_container.push(RuntimeObject::Divert {
                target: branch_rejoin_target.clone(),
                variable: false,
            });
            if let Some(named_content) = trailing_named_content {
                content_container.push(named_content);
            }
            branch_content.push(RuntimeObject::NamedContent(vec![Container {
                content: content_container,
                name: Some("b".to_string()),
                flags: None,
                merge_tail_metadata: true,
            }]));
        } else {
            let mut content_container = Vec::new();
            if !branch.is_inline() {
                content_container.push(RuntimeObject::String("\n".to_string()));
            }
            for object in branch.content().content() {
                lower_object_into_with_context(
                    &mut content_container,
                    object,
                    &branch_path_mode,
                    choice_labels,
                    global_labels,
                    global_variables,
                );
            }
            content_container.push(RuntimeObject::Divert {
                target: branch_rejoin_target.clone(),
                variable: false,
            });
            branch_content.push(RuntimeObject::NamedContent(vec![Container {
                content: content_container,
                name: Some("b".to_string()),
                flags: None,
                merge_tail_metadata: true,
            }]));
        }

        content.push(RuntimeObject::Container(Container {
            content: branch_content,
            name: None,
            flags: None,
            merge_tail_metadata: true,
        }));
    }

    content.push(RuntimeObject::ControlCommand(ControlCommand::NoOp));
}

fn runtime_index_path(path_mode: &ChoicePathMode, index: usize) -> String {
    match path_mode {
        ChoicePathMode::Root => format!("0.{index}"),
        ChoicePathMode::RootGather { gather_name } => format!("0.{gather_name}.{index}"),
        ChoicePathMode::NestedRoot { container_path, .. } => format!("{container_path}.{index}"),
        ChoicePathMode::Flow { container_path, .. } => format!("{container_path}.{index}"),
    }
}

fn lower_object_into_with_context(
    content: &mut Vec<RuntimeObject>,
    object: &Object,
    path_mode: &ChoicePathMode,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
) {
    match object {
        Object::Text(text) => content.push(RuntimeObject::String(text.text().to_string())),
        Object::ContentList(content_list) => {
            lower_content_list_into_context(
                content,
                content_list,
                path_mode,
                choice_labels,
                global_labels,
                global_variables,
            );
        }
        Object::Expression(expression) => lower_output_expression_into(
            content,
            expression,
            choice_labels,
            global_labels,
            path_mode,
        ),
        Object::Conditional(conditional) => lower_conditional_into(
            content,
            conditional,
            choice_labels,
            global_labels,
            global_variables,
            path_mode,
        ),
        Object::LogicLine(expression) => {
            lower_logic_line_into(content, expression, choice_labels, global_labels, path_mode);
        }
        Object::Glue(_) => content.push(RuntimeObject::Glue),
        Object::Divert(divert) => push_divert_with_context(
            content,
            divert,
            path_mode,
            choice_labels,
            global_labels,
            global_variables,
        ),
        Object::Choice(_) => {}
        Object::Gather(_) => {} // Handled in lower_choice_weave
        Object::VariableAssignment(assignment) => {
            lower_variable_assignment_into(
                content,
                assignment,
                path_mode,
                choice_labels,
                global_labels,
            );
        }
        Object::IncDec(inc_dec) => {
            lower_inc_dec_into(content, inc_dec, path_mode, choice_labels, global_labels);
        }
        Object::Return(ret) => {
            content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
            if let Some(expr) = ret.returned_expression() {
                lower_expression_into(
                    content,
                    expr,
                    choice_labels,
                    global_labels,
                    path_mode,
                    false,
                );
            } else {
                content.push(RuntimeObject::Void);
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
            content.push(RuntimeObject::ControlCommand(ControlCommand::PopFunction));
        }
        Object::Tag(tag) => content.push(RuntimeObject::Tag {
            is_start: tag.is_start(),
        }),
        Object::Sequence(sequence) => content.push(RuntimeObject::Container(lower_sequence(
            sequence,
            choice_labels,
            global_labels,
            global_variables,
            path_mode,
            &sequence_container_path_for(path_mode, content.len()),
        ))),
        Object::Weave(weave) => {
            let nested_path_mode = path_mode.for_nested_weave(content.len());
            content.push(RuntimeObject::Container(Container {
                content: lower_choice_weave(
                    weave,
                    nested_path_mode,
                    global_labels,
                    global_variables,
                    false,
                ),
                name: None,
                flags: None,
                merge_tail_metadata: true,
            }));
        }
    }
}

fn lower_variable_assignment_into(
    content: &mut Vec<RuntimeObject>,
    assignment: &crate::parsed::VariableAssignment,
    path_mode: &ChoicePathMode,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
) {
    if assignment.is_global() {
        return;
    }

    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    lower_expression_into(
        content,
        assignment.expression(),
        choice_labels,
        global_labels,
        path_mode,
        false,
    );
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));

    if assignment.is_temporary() {
        content.push(RuntimeObject::VariableAssignment(
            assignment.name().to_string(),
        ));
    } else if path_mode.is_local_variable(assignment.name()) {
        content.push(RuntimeObject::TempVariableReassignment(
            assignment.name().to_string(),
        ));
    } else {
        content.push(RuntimeObject::VariableReassignment(
            assignment.name().to_string(),
        ));
    }
}

fn lower_inc_dec_into(
    content: &mut Vec<RuntimeObject>,
    inc_dec: &crate::parsed::IncDec,
    path_mode: &ChoicePathMode,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
) {
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    content.push(RuntimeObject::VariableReference(inc_dec.name().to_string()));
    lower_expression_into(
        content,
        inc_dec.expression(),
        choice_labels,
        global_labels,
        path_mode,
        false,
    );
    content.push(RuntimeObject::NativeFunction(
        if inc_dec.is_increment() { "+" } else { "-" }.to_string(),
    ));
    if path_mode.is_local_variable(inc_dec.name()) {
        content.push(RuntimeObject::TempVariableReassignment(
            inc_dec.name().to_string(),
        ));
    } else {
        content.push(RuntimeObject::VariableReassignment(
            inc_dec.name().to_string(),
        ));
    }
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
}

fn push_divert_with_context(
    content: &mut Vec<RuntimeObject>,
    divert: &Divert,
    path_mode: &ChoicePathMode,
    choice_labels: &HashMap<String, String>,
    global_labels: &HashMap<String, String>,
    global_variables: &HashSet<String>,
) {
    if !divert.arguments().is_empty() {
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
        for argument in divert.arguments() {
            lower_expression_into(
                content,
                argument,
                choice_labels,
                global_labels,
                path_mode,
                false,
            );
        }
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    }

    match divert.target() {
        DivertTarget::Done => content.push(RuntimeObject::ControlCommand(ControlCommand::Done)),
        DivertTarget::End => content.push(RuntimeObject::ControlCommand(ControlCommand::End)),
        DivertTarget::Path(target) => {
            let resolved_target = if let Some(choice_container_name) = choice_labels.get(target) {
                RuntimeObject::Divert {
                    target: choice_label_divert_target(path_mode, choice_container_name),
                    variable: false,
                }
            } else if let Some(label_target) = global_labels
                .get(target)
                .filter(|label_target| label_target.as_str() != target)
            {
                RuntimeObject::Divert {
                    target: label_target.clone(),
                    variable: false,
                }
            } else if global_variables.contains(target) {
                RuntimeObject::Divert {
                    target: target.clone(),
                    variable: true,
                }
            } else {
                RuntimeObject::Divert {
                    target: resolve_divert_target(target, path_mode),
                    variable: false,
                }
            };
            content.push(resolved_target);
        }
        DivertTarget::Empty => content.push(RuntimeObject::Divert {
            target: String::new(),
            variable: false,
        }),
    }
}

/// Resolve a divert target path, converting absolute flow names to relative paths
/// when the target is a sibling stitch or child stitch inside a choice container.
fn resolve_divert_target(target: &str, path_mode: &ChoicePathMode) -> String {
    match path_mode {
        ChoicePathMode::Root
        | ChoicePathMode::RootGather { .. }
        | ChoicePathMode::NestedRoot { .. } => target.to_string(),
        ChoicePathMode::Flow {
            parent_flow_name,
            flow_name,
            sibling_stitch_names,
            ..
        } => {
            if let Some((first_part, second_part)) = target.split_once('.') {
                if parent_flow_name.is_none()
                    && first_part == flow_name
                    && sibling_stitch_names.iter().any(|name| name == second_part)
                {
                    return resolve_single_stitch_target(second_part, path_mode);
                }
                return target.to_string();
            }

            // Handle simple target names
            resolve_single_stitch_target(target, path_mode)
        }
    }
}

/// Resolve a single stitch name to a relative path if applicable.
fn resolve_single_stitch_target(target: &str, path_mode: &ChoicePathMode) -> String {
    match path_mode {
        ChoicePathMode::Root
        | ChoicePathMode::RootGather { .. }
        | ChoicePathMode::NestedRoot { .. } => target.to_string(),
        ChoicePathMode::Flow {
            sibling_stitch_names,
            parent_flow_name,
            flow_name,
            self_target_relative,
            ..
        } => {
            if parent_flow_name.is_some() {
                // We're in a stitch - check if target is a sibling stitch
                if sibling_stitch_names.iter().any(|s| s == target) {
                    // From inside a choice container in a stitch, we need 4 levels up:
                    // 1. Named content container (containing c-0, c-1, g-0)
                    // 2. Weave content array
                    // 3. Stitch container
                    // 4. Knot container (where sibling stitches are defined)
                    ".^.^.^.^.".to_string() + target
                } else if Some(target) == parent_flow_name.as_deref() {
                    // Target is the parent knot itself
                    ".^.^.^.^".to_string()
                } else {
                    target.to_string()
                }
            } else {
                // We're in a knot
                if sibling_stitch_names.iter().any(|s| s == target) {
                    // Target is a child stitch - 3 levels up:
                    // 1. Named content container (containing c-0, c-1, g-0)
                    // 2. Weave content array
                    // 3. Knot container (where child stitches are defined)
                    ".^.^.^.".to_string() + target
                } else if *self_target_relative && target == *flow_name {
                    // Match C# CompactPathString behavior: use the relative
                    // self-target only when it is shorter than the global path.
                    compact_relative_path(".^.^.^", target)
                } else {
                    target.to_string()
                }
            }
        }
    }
}

fn compact_relative_path(relative: &str, global: &str) -> String {
    if relative.len() < global.len() {
        relative.to_string()
    } else {
        global.to_string()
    }
}

fn sequence_container_path_for(path_mode: &ChoicePathMode, content_index: usize) -> String {
    match path_mode {
        ChoicePathMode::Root => format!("0.{content_index}"),
        ChoicePathMode::RootGather { gather_name } => {
            format!("0.{gather_name}.{content_index}")
        }
        ChoicePathMode::NestedRoot { container_path, .. }
        | ChoicePathMode::Flow { container_path, .. } => {
            format!("{container_path}.{content_index}")
        }
    }
}

fn choice_point_target(
    path_mode: &ChoicePathMode,
    has_start_content: bool,
    choice_index: usize,
) -> String {
    match path_mode {
        ChoicePathMode::Root => format!("0.c-{choice_index}"),
        ChoicePathMode::RootGather { .. } if has_start_content => format!(".^.^.c-{choice_index}"),
        ChoicePathMode::RootGather { .. } => format!(".^.c-{choice_index}"),
        ChoicePathMode::NestedRoot { .. } if has_start_content => format!(".^.^.c-{choice_index}"),
        ChoicePathMode::NestedRoot { .. } => format!(".^.c-{choice_index}"),
        ChoicePathMode::Flow { .. } if has_start_content => format!(".^.^.c-{choice_index}"),
        ChoicePathMode::Flow { .. } => format!(".^.c-{choice_index}"),
    }
}

fn outer_return_target(path_mode: &ChoicePathMode, choice_point_index: usize) -> String {
    match path_mode {
        ChoicePathMode::Root => format!("0.{choice_point_index}"),
        ChoicePathMode::RootGather { gather_name } => {
            format!("0.{gather_name}.{choice_point_index}")
        }
        ChoicePathMode::NestedRoot { container_path, .. } => {
            format!("{container_path}.{choice_point_index}")
        }
        ChoicePathMode::Flow { container_path, .. } => {
            format!("{container_path}.{choice_point_index}")
        }
    }
}

fn choice_content_return_target(path_mode: &ChoicePathMode, choice_container_name: &str) -> String {
    match path_mode {
        ChoicePathMode::Root => format!("0.{choice_container_name}"),
        ChoicePathMode::RootGather { gather_name } => {
            format!("0.{gather_name}.{choice_container_name}")
        }
        ChoicePathMode::NestedRoot { container_path, .. } => {
            format!("{container_path}.{choice_container_name}")
        }
        ChoicePathMode::Flow { container_path, .. } => {
            format!("{container_path}.{choice_container_name}")
        }
    }
}

fn start_content_target(path_mode: &ChoicePathMode, choice_point_index: usize) -> String {
    match path_mode {
        ChoicePathMode::Root => format!("0.{choice_point_index}.s"),
        ChoicePathMode::RootGather { .. } => format!(".^.^.{choice_point_index}.s"),
        ChoicePathMode::NestedRoot { .. } => format!(".^.^.{choice_point_index}.s"),
        ChoicePathMode::Flow { .. } => format!(".^.^.{choice_point_index}.s"),
    }
}

fn gather_target(
    path_mode: &ChoicePathMode,
    gather_container_name: &str,
    has_following_gather: bool,
) -> String {
    match path_mode {
        ChoicePathMode::Root => format!("0.{gather_container_name}"),
        ChoicePathMode::RootGather { .. } => format!("0.{gather_container_name}"),
        ChoicePathMode::NestedRoot { .. } if has_following_gather => {
            format!(".^.^.{gather_container_name}")
        }
        ChoicePathMode::NestedRoot { gather_target, .. } => gather_target.clone(),
        ChoicePathMode::Flow { container_path, .. } => up_path(
            2 + flow_container_extra_depth(container_path),
            gather_container_name,
        ),
    }
}

fn flow_container_extra_depth(container_path: &str) -> usize {
    let Some((_, rest)) = container_path.split_once(".0") else {
        return 0;
    };
    rest.trim_start_matches('.')
        .split('.')
        .filter(|part| !part.is_empty())
        .count()
}

fn up_path(levels: usize, target: &str) -> String {
    format!("{}.{target}", vec!["^"; levels].join(".")).replacen('^', ".^", 1)
}

fn choice_label_count_target(
    path_mode: &ChoicePathMode,
    choice_container_name: &str,
    has_start_content: bool,
) -> String {
    match path_mode {
        ChoicePathMode::Root
        | ChoicePathMode::RootGather { .. }
        | ChoicePathMode::NestedRoot { .. }
            if has_start_content =>
        {
            format!(".^.^.^.{choice_container_name}")
        }
        ChoicePathMode::Root
        | ChoicePathMode::RootGather { .. }
        | ChoicePathMode::NestedRoot { .. } => {
            format!(".^.^.{choice_container_name}")
        }
        ChoicePathMode::Flow { container_path, .. } => {
            let levels =
                1 + flow_container_extra_depth(container_path) + usize::from(has_start_content);
            up_path(levels, choice_container_name)
        }
    }
}

fn choice_label_divert_target(path_mode: &ChoicePathMode, choice_container_name: &str) -> String {
    match path_mode {
        ChoicePathMode::Root
        | ChoicePathMode::RootGather { .. }
        | ChoicePathMode::NestedRoot { .. } => {
            format!(".^.^.{choice_container_name}")
        }
        ChoicePathMode::Flow { container_path, .. } => {
            let levels = 2 + flow_container_extra_depth(container_path);
            up_path(levels, choice_container_name)
        }
    }
}

fn ends_with_flow_terminator(content: &[RuntimeObject]) -> bool {
    content
        .iter()
        .rev()
        .find(|object| !matches!(object, RuntimeObject::String(text) if text == "\n"))
        .is_some_and(|object| {
            matches!(
                object,
                RuntimeObject::Divert { .. }
                    | RuntimeObject::ControlCommand(ControlCommand::End | ControlCommand::Done)
            )
        })
}

fn done_container(name: &str, count_all_visits: bool) -> Container {
    Container {
        content: vec![RuntimeObject::ControlCommand(ControlCommand::Done)],
        name: Some(name.to_string()),
        flags: count_all_visits.then_some(5),
        merge_tail_metadata: true,
    }
}
