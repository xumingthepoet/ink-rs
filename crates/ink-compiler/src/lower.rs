use std::collections::{BTreeMap, HashMap, HashSet};

mod assignment;
mod composite_literal;
mod conditional;
mod context;
mod divert;
mod expression;
mod flow;
mod indexes;
mod labels;
mod path;
mod value;
mod weave;

use ink_story_json_format::{
    Container, ControlCommand, InternalFunction, NamedContainer, Object as RuntimeObject,
    Program as RuntimeProgram,
};

use crate::{
    analysis::CheckedStory,
    compiler::StageOutput,
    diagnostic::Diagnostic,
    parsed::{Choice, Object},
    source::SourceSpan,
};

use assignment::{
    lower_assignment_initializer_with_context, lower_inc_dec_into, lower_variable_assignment_into,
};
use conditional::lower_conditional_into;
use context::{ChoicePathMode, LoweringContext};
use divert::{
    lower_tail_recursive_return_into, lower_tunnel_onwards_into, push_divert_with_context,
};
use expression::{lower_expression_into, lower_logic_line_into, lower_output_expression_into};
use flow::lower_module_flow;
use indexes::{ConstantValues, LoweringIndexes, RuntimeLenEstimator, StructDefinitions};
use path::{compact_path_strings_in_container, LabelIndex};
use weave::{lower_choice_weave, lower_content_list_into_context};

pub(crate) fn lower(story: &CheckedStory) -> StageOutput<RuntimeProgram> {
    let indexes = LoweringIndexes::build(
        &story.parsed,
        RuntimeLenEstimator {
            choice_content_len: estimated_choice_content_len,
            object_len: estimated_runtime_len_for_label_collection,
        },
    );
    if story.parsed.modules().is_empty() {
        return StageOutput {
            artifact: None,
            diagnostics: vec![Diagnostic::error(
                SourceSpan::new(None, 1, 1),
                "Cannot lower a story without explicit modules",
            )],
        };
    }

    lower_module_story(story, &indexes)
}

fn lower_module_story(
    story: &CheckedStory,
    indexes: &LoweringIndexes<'_>,
) -> StageOutput<RuntimeProgram> {
    let mut named_containers = story
        .parsed
        .modules()
        .iter()
        .filter(|module| story.module_reachability.is_reachable(module.name()))
        .map(|module| named_container(lower_module(module, indexes)))
        .collect::<Vec<_>>();
    if let Some(global_declarations) =
        lower_global_declarations(indexes, Some(&story.module_reachability))
    {
        named_containers.push(named_container(global_declarations));
    }

    let root_content = if let Some(entry_point) = &story.entry_point {
        vec![
            RuntimeObject::Divert {
                target: format!("{}.{}", entry_point.module, entry_point.knot),
                variable: false,
            },
            RuntimeObject::ControlCommand(ControlCommand::Done),
        ]
    } else {
        vec![RuntimeObject::ControlCommand(ControlCommand::Done)]
    };

    let mut root = Container {
        content: root_content,
        named_content: named_containers,
        name: None,
        flags: None,
    };
    compact_path_strings_in_container(&mut root);

    let mut program = RuntimeProgram::new(root);
    program.internal_functions = collect_internal_functions(story);

    StageOutput {
        artifact: Some(program),
        diagnostics: Vec::new(),
    }
}

fn collect_internal_functions(story: &CheckedStory) -> BTreeMap<String, InternalFunction> {
    let mut functions = BTreeMap::new();
    for module in story
        .parsed
        .modules()
        .iter()
        .filter(|module| story.module_reachability.is_reachable(module.name()))
    {
        for flow in module.flows() {
            collect_internal_functions_in_flow(flow, Some(module.name()), None, &mut functions);
        }
    }
    for flow in story.parsed.flows() {
        collect_internal_functions_in_flow(flow, None, None, &mut functions);
    }
    functions
}

fn collect_internal_functions_in_flow(
    flow: &crate::parsed::Flow,
    module_name: Option<&str>,
    parent_flow_path: Option<&str>,
    functions: &mut BTreeMap<String, InternalFunction>,
) {
    let source_flow_path = parent_flow_path
        .map(|parent| format!("{parent}.{}", flow.name()))
        .unwrap_or_else(|| flow.name().to_string());

    if flow.is_internal() {
        let host_name = module_name
            .map(|module| format!("{module}::{source_flow_path}"))
            .unwrap_or_else(|| source_flow_path.clone());
        let runtime_path = module_name
            .map(|module| format!("{module}.{source_flow_path}"))
            .unwrap_or_else(|| source_flow_path.clone());
        let arg_types = flow
            .arguments()
            .iter()
            .map(|argument| {
                argument
                    .declared_type()
                    .expect("INTERNAL parameters are typed during parsing")
                    .to_string()
            })
            .collect();

        functions.insert(
            host_name,
            InternalFunction::new(runtime_path, arg_types, flow.return_type().to_string()),
        );
    }

    for child in flow.child_flows() {
        collect_internal_functions_in_flow(child, module_name, Some(&source_flow_path), functions);
    }
}

fn lower_module(module: &crate::parsed::Module, indexes: &LoweringIndexes<'_>) -> Container {
    let named_content = module
        .flows()
        .iter()
        .map(|flow| named_container(lower_module_flow(module.name(), flow, indexes)))
        .collect::<Vec<_>>();

    Container {
        content: Vec::new(),
        named_content,
        name: Some(module.name().to_string()),
        flags: None,
    }
}

pub(super) fn named_container(container: Container) -> NamedContainer {
    let name = container
        .name
        .clone()
        .expect("named content containers must carry a container name");
    NamedContainer::new(name, container)
}

pub(super) fn named_content(
    name: impl Into<String>,
    content: Vec<RuntimeObject>,
) -> NamedContainer {
    let name = name.into();
    NamedContainer::new(
        name.clone(),
        Container {
            content,
            named_content: Vec::new(),
            name: Some(name),
            flags: None,
        },
    )
}

fn lower_global_declarations(
    indexes: &LoweringIndexes<'_>,
    reachability: Option<&crate::analysis::ModuleReachability>,
) -> Option<Container> {
    let visible_declarations = indexes
        .variable_declarations
        .iter()
        .filter(
            |declaration| match (declaration.module_name(), reachability) {
                (Some(module), Some(reachability)) => reachability.is_reachable(module),
                _ => true,
            },
        )
        .collect::<Vec<_>>();

    if visible_declarations.is_empty() {
        return None;
    }

    let choice_labels = LabelIndex::new();
    let mut content = vec![RuntimeObject::ControlCommand(ControlCommand::EvalStart)];
    for declaration in visible_declarations
        .iter()
        .copied()
        .filter(|declaration| declaration.assignment().is_global())
    {
        let path_mode = declaration
            .module_name()
            .map(|module_name| ChoicePathMode::Module {
                module_name: module_name.to_string(),
            })
            .unwrap_or(ChoicePathMode::Root);
        let context = LoweringContext::new(
            path_mode,
            &choice_labels,
            &indexes.global_labels,
            &indexes.global_variables,
            &indexes.external_signatures,
            &indexes.constants,
            &indexes.struct_definitions,
        );
        if lower_assignment_initializer_with_context(
            &mut content,
            declaration.assignment(),
            &context,
        ) {
            content.push(RuntimeObject::GlobalVariableAssignment(
                declaration.runtime_name().to_string(),
            ));
        }
    }
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    content.push(RuntimeObject::ControlCommand(ControlCommand::End));

    Some(Container {
        content,
        named_content: Vec::new(),
        name: Some("global decl".to_string()),
        flags: None,
    })
}

fn estimated_choice_content_len(
    choice: &Choice,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    global_variables: &HashSet<String>,
) -> usize {
    let mut content = Vec::new();
    let choice_labels = LabelIndex::new();
    let global_labels = LabelIndex::new();
    let external_signatures = HashMap::new();
    let context = LoweringContext::new(
        ChoicePathMode::Root,
        &choice_labels,
        &global_labels,
        global_variables,
        &external_signatures,
        constants,
        struct_definitions,
    );
    lower_content_list_into_context(&mut content, choice.inner_content(), &context);
    content.len()
}

fn estimated_runtime_len_for_label_collection(
    object: &Object,
    constants: &ConstantValues,
    struct_definitions: &StructDefinitions,
    global_variables: &HashSet<String>,
) -> usize {
    if matches!(object, Object::Weave(_)) {
        return 1;
    }

    let mut content = Vec::new();
    let choice_labels = LabelIndex::new();
    let global_labels = LabelIndex::new();
    let external_signatures = HashMap::new();
    let context = LoweringContext::new(
        ChoicePathMode::Root,
        &choice_labels,
        &global_labels,
        global_variables,
        &external_signatures,
        constants,
        struct_definitions,
    );
    lower_object_into_with_context_count(&mut content, object, &context);
    content.len()
}

fn lower_object_into_with_context(
    content: &mut Vec<RuntimeObject>,
    object: &Object,
    context: &LoweringContext<'_>,
) {
    lower_object_into_with_context_count(content, object, context);
}

fn lower_object_into_with_context_count(
    content: &mut Vec<RuntimeObject>,
    object: &Object,
    context: &LoweringContext<'_>,
) {
    let path_mode = context.path_mode();
    match object {
        Object::Text(text) => content.push(RuntimeObject::String(text.text().to_string())),
        Object::AuthorWarning(_) => {}
        Object::ContentList(content_list) => {
            lower_content_list_into_context(content, content_list, context);
        }
        Object::Expression(expression) => {
            lower_output_expression_into(content, expression, context)
        }
        Object::Conditional(conditional) => lower_conditional_into(content, conditional, context),
        Object::LogicLine(expression) => {
            lower_logic_line_into(content, expression, context);
        }
        Object::Glue(_) => content.push(RuntimeObject::Glue),
        Object::Divert(divert) => push_divert_with_context(content, divert, context),
        Object::TunnelOnwards(tunnel_onwards) => {
            lower_tunnel_onwards_into(content, tunnel_onwards, context);
        }
        Object::Choice(_) => {}
        Object::ConstantDeclaration(_) => {}
        Object::EnumDeclaration(_) => {}
        Object::Gather(_) => {} // Handled in lower_choice_weave
        Object::StructDeclaration(_) => {}
        Object::VariableAssignment(assignment) => {
            lower_variable_assignment_into(content, assignment, context);
        }
        Object::IncDec(inc_dec) => {
            lower_inc_dec_into(content, inc_dec, context);
        }
        Object::Return(ret) => {
            if lower_tail_recursive_return_into(content, ret, context) {
                return;
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
            if let Some(expr) = ret.returned_expression() {
                lower_expression_into(content, expr, context, false);
            } else {
                content.push(RuntimeObject::Void);
            }
            content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
            content.push(RuntimeObject::ControlCommand(ControlCommand::PopFunction));
        }
        Object::Tag(tag) => content.push(RuntimeObject::Tag {
            is_start: tag.is_start(),
        }),
        Object::Weave(weave) => {
            let nested_context = context.with_path_mode(path_mode.for_nested_weave(content.len()));
            content.push(RuntimeObject::Container(lower_choice_weave(
                weave,
                &nested_context,
            )));
        }
        Object::ExternalDeclaration(_) => {}
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

fn ends_with_end_or_done(content: &[RuntimeObject]) -> bool {
    content
        .iter()
        .rev()
        .find(|object| !matches!(object, RuntimeObject::String(text) if text == "\n"))
        .is_some_and(|object| {
            matches!(
                object,
                RuntimeObject::ControlCommand(ControlCommand::End | ControlCommand::Done)
            )
        })
}

fn done_container(name: &str) -> Container {
    Container {
        content: vec![RuntimeObject::ControlCommand(ControlCommand::Done)],
        named_content: Vec::new(),
        name: Some(name.to_string()),
        flags: None,
    }
}

#[cfg(test)]
mod tests {
    use ink_story_json_format::Object as RuntimeObject;
    use serde_json::{json, Value};

    use crate::{compiler::Compiler, source::SourceInput};

    use super::lower;

    #[test]
    fn module_lowering_uses_checked_entry_point_and_reachability() {
        let source = concat!(
            "=== module game ===\n",
            "IMPORT helper FROM support\n",
            "== main ==\n",
            "-> support::helper\n",
            "=== module support ===\n",
            "== helper ==\n",
            "-> END\n",
            "=== module unused ===\n",
            "== spare ==\n",
            "-> END\n",
        );
        let compiler = Compiler::default();
        let parsed = compiler
            .parse(SourceInput::new(source))
            .artifact
            .expect("parse should succeed");
        let checked = compiler
            .analyze(parsed)
            .artifact
            .expect("analysis should produce checked story");

        assert!(checked.module_reachability.is_reachable("support"));
        assert!(!checked.module_reachability.is_reachable("unused"));

        let program = lower(&checked)
            .artifact
            .expect("lowering should produce program");

        assert!(matches!(
            program.root.content.first(),
            Some(RuntimeObject::Divert { target, variable: false }) if target == "game.main"
        ));
        let named_modules = program
            .root
            .named_content
            .iter()
            .map(|named| named.name.as_str())
            .collect::<Vec<_>>();
        assert!(named_modules.contains(&"game"));
        assert!(named_modules.contains(&"support"));
        assert!(!named_modules.contains(&"unused"));
    }

    #[test]
    fn module_lowering_json_contains_root_entry_and_reachable_module_containers() {
        let source = concat!(
            "=== module game ===\n",
            "IMPORT helper FROM support\n",
            "== main ==\n",
            "Game.\n",
            "-> END\n",
            "=== module support ===\n",
            "== helper ==\n",
            "Support.\n",
            "-> END\n",
            "=== module unused ===\n",
            "== main_unused ==\n",
            "Unused.\n",
            "-> END\n",
        );
        let compiler = Compiler::default();
        let compiled = compiler.compile(SourceInput::new(source));

        assert!(
            compiled.artifact.is_some(),
            "module story should compile with warnings only: {:#?}",
            compiled.diagnostics
        );
        let json = compiled
            .artifact
            .expect("compiled story")
            .program
            .to_json_value();
        let root = json
            .get("root")
            .and_then(Value::as_array)
            .expect("root should encode as JSON array");

        assert_eq!(root.first(), Some(&json!({"->": "game.main"})));
        let named = root
            .last()
            .and_then(Value::as_object)
            .expect("root should end with named content object");
        assert!(named.contains_key("game"), "{json:#}");
        assert!(named.contains_key("support"), "{json:#}");
        assert!(!named.contains_key("unused"), "{json:#}");
        assert!(container_json_has_named_content(
            named.get("game").expect("game module"),
            "main"
        ));
        assert!(container_json_has_named_content(
            named.get("support").expect("support module"),
            "helper"
        ));
    }

    #[test]
    fn module_lowering_json_excludes_unreachable_module_globals() {
        let source = concat!(
            "=== module game ===\n",
            "IMPORT helper FROM support\n",
            "== main ==\n",
            "-> support::helper\n",
            "=== module support ===\n",
            "VAR shown: int = 1\n",
            "== helper ==\n",
            "-> END\n",
            "=== module unused ===\n",
            "VAR hidden: int = 2\n",
            "== spare ==\n",
            "-> END\n",
        );
        let compiled = Compiler::default().compile(SourceInput::new(source));

        assert!(
            compiled.artifact.is_some(),
            "module story should compile with unreachable warning only: {:#?}",
            compiled.diagnostics
        );
        let json = compiled
            .artifact
            .expect("compiled story")
            .program
            .to_json_value();
        let json_text = json.to_string();

        assert!(json_text.contains("support::shown"), "{json:#}");
        assert!(!json_text.contains("unused"), "{json:#}");
        assert!(!json_text.contains("unused::hidden"), "{json:#}");
    }

    #[test]
    fn internal_modules_are_lowered_as_host_roots_with_import_dependencies() {
        let source = concat!(
            "=== module game ===\n",
            "== main ==\n",
            "-> END\n",
            "=== module host_api ===\n",
            "IMPORT value FROM config\n",
            "== INTERNAL read() => string ==\n",
            "~ return config::value()\n",
            "=== module config ===\n",
            "== function value() => string ==\n",
            "~ return \"ok\"\n",
            "=== module unused ===\n",
            "== spare ==\n",
            "-> END\n",
        );
        let compiled = Compiler::default().compile(SourceInput::new(source));

        assert!(
            compiled.artifact.is_some(),
            "module story should compile with unreachable warning only: {:#?}",
            compiled.diagnostics
        );
        let json = compiled
            .artifact
            .expect("compiled story")
            .program
            .to_json_value();
        let root = json
            .get("root")
            .and_then(Value::as_array)
            .expect("root should encode as JSON array");
        let named = root
            .last()
            .and_then(Value::as_object)
            .expect("root should end with named content object");
        let internal_functions = json
            .get("internalFunctions")
            .and_then(Value::as_object)
            .expect("internalFunctions should be emitted");

        assert!(named.contains_key("game"), "{json:#}");
        assert!(named.contains_key("host_api"), "{json:#}");
        assert!(named.contains_key("config"), "{json:#}");
        assert!(!named.contains_key("unused"), "{json:#}");
        assert_eq!(
            internal_functions.get("host_api::read"),
            Some(&json!({
                "args": 0,
                "argTypes": [],
                "returnType": "string",
                "path": "host_api.read"
            }))
        );
    }

    #[test]
    fn module_global_initializer_lowering_uses_scoped_runtime_names() {
        let source = concat!(
            "=== module game ===\n",
            "IMPORT helper FROM support\n",
            "== main ==\n",
            "-> support::helper\n",
            "=== module support ===\n",
            "VAR shown: int = 7\n",
            "== helper ==\n",
            "-> END\n",
            "=== module unused ===\n",
            "VAR hidden: int = 9\n",
            "== spare ==\n",
            "-> END\n",
        );
        let compiled = Compiler::default().compile(SourceInput::new(source));

        assert!(
            compiled.artifact.is_some(),
            "module story should compile with unreachable warning only: {:#?}",
            compiled.diagnostics
        );
        let json = compiled
            .artifact
            .expect("compiled story")
            .program
            .to_json_value();
        let root = json
            .get("root")
            .and_then(Value::as_array)
            .expect("root should encode as JSON array");
        let named = root
            .last()
            .and_then(Value::as_object)
            .expect("root should end with named content object");
        let global_decl = named
            .get("global decl")
            .and_then(Value::as_array)
            .expect("global declarations should encode as a container");

        assert!(global_decl.contains(&json!(7)), "{json:#}");
        assert!(
            global_decl.contains(&json!({"VAR=": "support::shown"})),
            "{json:#}"
        );
        assert!(
            !global_decl.contains(&json!({"VAR=": "unused::hidden"})),
            "{json:#}"
        );
    }

    #[test]
    fn module_compilation_fails_when_unreachable_module_has_errors() {
        let source = concat!(
            "=== module game ===\n",
            "== main ==\n",
            "-> END\n",
            "=== module unused ===\n",
            "VAR broken: int = \"wrong\"\n",
            "== spare ==\n",
            "-> END\n",
        );
        let compiled = Compiler::default().compile(SourceInput::new(source));

        assert!(compiled.artifact.is_none());
        assert!(
            compiled
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(
                    "Initializer for variable 'broken' has type string but declared type is int"
                )),
            "unreachable module errors should fail compilation: {:#?}",
            compiled.diagnostics
        );
    }

    #[test]
    fn expression_lowering_context_preserves_current_expression_json() {
        let source = concat!(
            "=== module game ===\n",
            "IMPORT add FROM math\n",
            "STRUCT Stats {\n",
            "hp: int\n",
            "}\n",
            "CONST LIMIT: int = 3\n",
            "VAR state: Stats = { hp: 4 }\n",
            "VAR items: int[] = [5, 8]\n",
            "== main ==\n",
            "{math::add(LIMIT, state.hp)}|{items[1]}|{\"HP {state.hp}\"}\n",
            "-> END\n",
            "=== module math ===\n",
            "== function add(left: int, right: int) => int ==\n",
            "~ return left + right\n",
        );
        let compiled = Compiler::default().compile(SourceInput::new(source));

        assert!(
            compiled.artifact.is_some(),
            "module story should compile: {:#?}",
            compiled.diagnostics
        );
        let json = compiled
            .artifact
            .expect("compiled story")
            .program
            .to_json_value();

        assert!(json_contains_sequence(
            &json,
            &[
                json!(3),
                json!({"VAR?": "game::state"}),
                json!("^hp"),
                json!("FIELD")
            ]
        ));
        assert!(json_contains_sequence(
            &json,
            &[json!({"VAR?": "game::items"}), json!(1), json!("INDEX")]
        ));
        assert!(json_contains_sequence(
            &json,
            &[
                json!("str"),
                json!("^HP "),
                json!("ev"),
                json!({"VAR?": "game::state"}),
                json!("^hp"),
                json!("FIELD"),
                json!("out"),
                json!("/ev"),
                json!("/str")
            ]
        ));
        assert!(
            json.to_string().contains("\"f()\":\"math.add\""),
            "{json:#}"
        );
    }

    #[test]
    fn assignment_lowering_context_preserves_current_lvalue_json() {
        let source = concat!(
            "=== module game ===\n",
            "STRUCT Stats {\n",
            "hp: int\n",
            "}\n",
            "VAR score: int = 1\n",
            "VAR stats: Stats = { hp: 2 }\n",
            "VAR items: int[] = [1, 2, 3]\n",
            "== main ==\n",
            "~ temp local: int = 4\n",
            "~ local = 5\n",
            "~ score += local\n",
            "~ stats.hp = score\n",
            "~ items[local - 5] += stats.hp\n",
            "~ ARRAY_REMOVE(items, 1)\n",
            "{local}|{score}|{stats.hp}|{LEN(items)}\n",
            "-> END\n",
        );
        let compiled = Compiler::default().compile(SourceInput::new(source));

        assert!(
            compiled.artifact.is_some(),
            "module story should compile: {:#?}",
            compiled.diagnostics
        );
        let json = compiled
            .artifact
            .expect("compiled story")
            .program
            .to_json_value();

        assert!(json_contains_sequence(
            &json,
            &[
                json!("ev"),
                json!(4),
                json!("/ev"),
                json!({"temp=": "local"})
            ]
        ));
        assert!(json_contains_sequence(
            &json,
            &[
                json!("ev"),
                json!(5),
                json!("/ev"),
                json!({"temp=": "local", "re": true})
            ]
        ));
        assert!(json_contains_sequence(
            &json,
            &[
                json!({"VAR?": "game::score"}),
                json!({"VAR?": "local"}),
                json!("+"),
                json!({"VAR=": "game::score", "re": true})
            ]
        ));
        assert!(json_contains_sequence(
            &json,
            &[
                json!({"VAR?": "game::stats"}),
                json!("^hp"),
                json!({"VAR?": "game::score"}),
                json!("SET_FIELD")
            ]
        ));
        assert!(json_contains_sequence(
            &json,
            &[
                json!({"VAR?": "game::items"}),
                json!({"VAR?": "$lvalue0"}),
                json!("INDEX"),
                json!({"VAR?": "game::stats"}),
                json!("^hp"),
                json!("FIELD"),
                json!("+"),
                json!("SET_INDEX")
            ]
        ));
        assert!(json_contains_sequence(
            &json,
            &[
                json!({"VAR?": "game::items"}),
                json!(1),
                json!("ARRAY_REMOVE"),
                json!({"VAR=": "game::items", "re": true})
            ]
        ));
        assert!(json_contains_sequence(
            &json,
            &[json!({"VAR?": "game::items"}), json!("LEN")]
        ));
    }

    #[test]
    fn divert_lowering_context_preserves_current_control_flow_json() {
        let source = concat!(
            "=== module game ===\n",
            "EXTERNAL ext(value: int) => int\n",
            "VAR next: -> = -> dynamic_target\n",
            "== main ==\n",
            "{ext(1)}|{count_down(2, 0)}\n",
            "-> static_target\n",
            "== static_target ==\n",
            "-> {next}\n",
            "== dynamic_target ==\n",
            "-> tunnel ->\n",
            "Back.\n",
            "-> END\n",
            "== tunnel ==\n",
            "->-> tunnel_exit\n",
            "== tunnel_exit ==\n",
            "Tunnel exit.\n",
            "-> END\n",
            "== function count_down(n: int, acc: int) => int ==\n",
            "{ if n <= 0:\n",
            "    ~ return acc\n",
            "- else:\n",
            "    ~ return count_down(n - 1, acc + 1)\n",
            "}\n",
        );
        let compiled = Compiler::default().compile(SourceInput::new(source));

        assert!(
            compiled.artifact.is_some(),
            "module story should compile: {:#?}",
            compiled.diagnostics
        );
        let json = compiled
            .artifact
            .expect("compiled story")
            .program
            .to_json_value();

        assert!(json_contains_sequence(
            &json,
            &[json!({"->": "game.static_target"})]
        ));
        assert!(json_contains_sequence(
            &json,
            &[
                json!("ev"),
                json!({"VAR?": "game::next"}),
                json!({"temp=": "$divertTarget"}),
                json!("/ev"),
                json!({"->": "$divertTarget", "var": true})
            ]
        ));
        assert!(json_contains_sequence(
            &json,
            &[json!({"->t->": "game.tunnel"})]
        ));
        assert!(json_contains_sequence(
            &json,
            &[
                json!("ev"),
                json!({"^->": "game.tunnel_exit"}),
                json!("/ev"),
                json!("->->")
            ]
        ));
        assert!(
            json.to_string().contains("\"x()\":\"game::ext\""),
            "{json:#}"
        );
        assert!(
            json.to_string().contains("\"f()\":\"game.count_down\""),
            "{json:#}"
        );
        assert!(json_contains_sequence(
            &json,
            &[
                json!({"VAR?": "n"}),
                json!(1),
                json!("-"),
                json!({"VAR?": "acc"}),
                json!(1),
                json!("+"),
                json!({"temp=": "acc"}),
                json!({"temp=": "n"})
            ]
        ));
    }

    #[test]
    fn structured_weave_context_preserves_choice_and_conditional_json() {
        let source = concat!(
            "=== module game ===\n",
            "VAR ready: bool = true\n",
            "== main ==\n",
            "{ if ready:\n",
            "    Ready.\n",
            "- else:\n",
            "    Not ready.\n",
            "}\n",
            "* [Take]\n",
            "    Took.\n",
            "    { if ready:\n",
            "        Still ready.\n",
            "    - else:\n",
            "        Not ready.\n",
            "    }\n",
            "- (after)\n",
            "After.\n",
            "-> END\n",
        );
        let compiled = Compiler::default().compile(SourceInput::new(source));

        assert!(
            compiled.artifact.is_some(),
            "module story should compile: {:#?}",
            compiled.diagnostics
        );
        let json = compiled
            .artifact
            .expect("compiled story")
            .program
            .to_json_value();

        assert!(json_contains_sequence(
            &json,
            &[json!("ev"), json!({"VAR?": "game::ready"}), json!("/ev")]
        ));
        assert!(json_contains_value(
            &json,
            &json!({"->": ".^.b", "c": true})
        ));
        assert!(json_contains_object_key(&json, "c-0"), "{json:#}");
        assert!(json_contains_object_key(&json, "after"), "{json:#}");
    }

    fn container_json_has_named_content(container: &Value, name: &str) -> bool {
        container
            .as_array()
            .and_then(|items| items.last())
            .and_then(Value::as_object)
            .is_some_and(|named| named.contains_key(name))
    }

    fn json_contains_sequence(value: &Value, expected: &[Value]) -> bool {
        match value {
            Value::Array(items) => {
                items
                    .windows(expected.len())
                    .any(|window| window == expected)
                    || items
                        .iter()
                        .any(|item| json_contains_sequence(item, expected))
            }
            Value::Object(object) => object
                .values()
                .any(|item| json_contains_sequence(item, expected)),
            _ => false,
        }
    }

    fn json_contains_value(value: &Value, expected: &Value) -> bool {
        if value == expected {
            return true;
        }

        match value {
            Value::Array(items) => items.iter().any(|item| json_contains_value(item, expected)),
            Value::Object(object) => object
                .values()
                .any(|item| json_contains_value(item, expected)),
            _ => false,
        }
    }

    fn json_contains_object_key(value: &Value, expected_key: &str) -> bool {
        match value {
            Value::Array(items) => items
                .iter()
                .any(|item| json_contains_object_key(item, expected_key)),
            Value::Object(object) => {
                object.contains_key(expected_key)
                    || object
                        .values()
                        .any(|item| json_contains_object_key(item, expected_key))
            }
            _ => false,
        }
    }
}
