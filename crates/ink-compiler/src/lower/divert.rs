use std::collections::HashSet;

use ink_story_json_format::{ControlCommand, Object as RuntimeObject};

use crate::parsed::{Divert, DivertTarget, Expression, FlowArgument, Return, TunnelOnwards};

use super::context::{ChoicePathMode, LoweringContext};
use super::expression::{
    dynamic_interface_knot_signature, lower_expression_into, lower_function_arg_into,
};
use super::indexes::CallSignature;

pub(super) fn lower_tail_recursive_return_into(
    content: &mut Vec<RuntimeObject>,
    ret: &Return,
    context: &LoweringContext<'_>,
) -> bool {
    let path_mode = context.path_mode();
    let Some(flow_name) = path_mode.current_flow_name() else {
        return false;
    };
    let external_signatures = context.external_signatures();
    let signature_name = path_mode
        .current_module_name()
        .map(|module_name| format!("{module_name}::{flow_name}"))
        .filter(|qualified_name| external_signatures.contains_key(qualified_name))
        .unwrap_or_else(|| flow_name.to_string());
    let Some(tail_args) = ret.direct_self_tail_call_args(flow_name) else {
        return false;
    };
    let Some(CallSignature::Ink {
        args: expected_args,
        ..
    }) = external_signatures.get(&signature_name)
    else {
        return false;
    };
    if expected_args.len() != tail_args.len() {
        return false;
    }
    let Some(body_start_target) = path_mode.current_flow_body_start_target(expected_args.len())
    else {
        return false;
    };

    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    let mut visiting_constants = HashSet::new();
    for (index, arg) in tail_args.iter().enumerate() {
        lower_function_arg_into(
            content,
            arg,
            expected_args.get(index),
            context,
            false,
            &mut visiting_constants,
        );
    }
    for argument in expected_args.iter().rev() {
        content.push(RuntimeObject::VariableAssignment(
            argument.name().to_string(),
        ));
    }
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    content.push(RuntimeObject::Divert {
        target: body_start_target,
        variable: false,
    });

    true
}

pub(super) fn push_divert_with_context(
    content: &mut Vec<RuntimeObject>,
    divert: &Divert,
    context: &LoweringContext<'_>,
) {
    if let DivertTarget::Dynamic(expression) = divert.target() {
        push_dynamic_divert_with_context(
            content,
            divert,
            DynamicDivertTarget {
                expression: expression.clone(),
                divert_arguments: divert.arguments().to_vec(),
            },
            context,
        );
        return;
    }

    if !divert.arguments().is_empty() {
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
        lower_static_divert_arguments_into(content, divert.target(), divert.arguments(), context);
        content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    }

    if divert.is_thread() {
        content.push(RuntimeObject::ControlCommand(ControlCommand::StartThread));
    }

    match divert.target() {
        DivertTarget::Done => content.push(RuntimeObject::ControlCommand(ControlCommand::Done)),
        DivertTarget::End => content.push(RuntimeObject::ControlCommand(ControlCommand::End)),
        DivertTarget::Dynamic(_) => {
            unreachable!("dynamic divert targets return before static lowering")
        }
        DivertTarget::Path(target) => {
            let resolved_target = if let Some(choice_target) = context.choice_labels().get(target) {
                runtime_divert(choice_target.to_string(), false, divert.is_tunnel())
            } else if let Some(label_target) = context
                .path_mode()
                .scoped_label_target(target, context.global_labels())
                .filter(|label_target| *label_target != target)
            {
                runtime_divert(
                    context.path_mode().resolve_label_target(label_target),
                    false,
                    divert.is_tunnel(),
                )
            } else {
                runtime_divert(
                    context.path_mode().resolve_divert_target(target),
                    false,
                    divert.is_tunnel(),
                )
            };
            content.push(resolved_target);
        }
        DivertTarget::QualifiedPath(target) => {
            let target = target.as_str();
            let resolved_target = if let Some(choice_target) = context.choice_labels().get(target) {
                runtime_divert(choice_target.to_string(), false, divert.is_tunnel())
            } else if let Some(label_target) = context
                .path_mode()
                .scoped_label_target(target, context.global_labels())
                .filter(|label_target| *label_target != target)
            {
                runtime_divert(
                    context.path_mode().resolve_label_target(label_target),
                    false,
                    divert.is_tunnel(),
                )
            } else {
                runtime_divert(
                    context.path_mode().resolve_divert_target(target),
                    false,
                    divert.is_tunnel(),
                )
            };
            content.push(resolved_target);
        }
        DivertTarget::Empty => {
            content.push(runtime_divert(String::new(), false, divert.is_tunnel()))
        }
    }
}

struct DynamicDivertTarget {
    expression: Expression,
    divert_arguments: Vec<Expression>,
}

fn push_dynamic_divert_with_context(
    content: &mut Vec<RuntimeObject>,
    divert: &Divert,
    dynamic_target: DynamicDivertTarget,
    context: &LoweringContext<'_>,
) {
    const DYNAMIC_DIVERT_TARGET_TEMP: &str = "$divertTarget";

    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    lower_dynamic_divert_arguments_into(
        content,
        &dynamic_target.expression,
        &dynamic_target.divert_arguments,
        context,
    );
    lower_expression_into(content, &dynamic_target.expression, context, false);
    content.push(RuntimeObject::VariableAssignment(
        DYNAMIC_DIVERT_TARGET_TEMP.to_string(),
    ));
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));

    if divert.is_thread() {
        content.push(RuntimeObject::ControlCommand(ControlCommand::StartThread));
    }
    content.push(runtime_divert(
        DYNAMIC_DIVERT_TARGET_TEMP.to_string(),
        true,
        divert.is_tunnel(),
    ));
}

fn runtime_divert(target: String, variable: bool, is_tunnel: bool) -> RuntimeObject {
    if is_tunnel {
        RuntimeObject::TunnelDivert { target, variable }
    } else {
        RuntimeObject::Divert { target, variable }
    }
}

pub(super) fn lower_tunnel_onwards_into(
    content: &mut Vec<RuntimeObject>,
    tunnel_onwards: &TunnelOnwards,
    context: &LoweringContext<'_>,
) {
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalStart));
    let dynamic_override = tunnel_onwards
        .override_target()
        .and_then(|target| match target {
            DivertTarget::Dynamic(expression) => Some(expression),
            _ => None,
        });
    if let Some(expression) = dynamic_override {
        lower_dynamic_divert_arguments_into(
            content,
            expression,
            tunnel_onwards.arguments(),
            context,
        );
    } else if let Some(target) = tunnel_onwards.override_target() {
        lower_static_divert_arguments_into(content, target, tunnel_onwards.arguments(), context);
    } else {
        lower_plain_divert_arguments_into(content, tunnel_onwards.arguments(), context);
    }
    if let Some(target) = tunnel_onwards.override_target() {
        match target {
            DivertTarget::Dynamic(expression) => {
                lower_expression_into(content, expression, context, false);
            }
            DivertTarget::Path(target) => {
                lower_tunnel_onwards_path_target_into(content, target, context);
            }
            DivertTarget::QualifiedPath(target) => {
                lower_tunnel_onwards_path_target_into(content, target.as_str(), context);
            }
            DivertTarget::Done => content.push(RuntimeObject::DivertTarget("DONE".to_string())),
            DivertTarget::End => content.push(RuntimeObject::DivertTarget("END".to_string())),
            DivertTarget::Empty => content.push(RuntimeObject::Void),
        }
    } else {
        content.push(RuntimeObject::Void);
    }
    content.push(RuntimeObject::ControlCommand(ControlCommand::EvalEnd));
    content.push(RuntimeObject::ControlCommand(ControlCommand::PopTunnel));
}

fn lower_dynamic_divert_arguments_into(
    content: &mut Vec<RuntimeObject>,
    target: &Expression,
    arguments: &[Expression],
    context: &LoweringContext<'_>,
) {
    let Some((_, signature)) = dynamic_interface_knot_signature(target, context) else {
        lower_plain_divert_arguments_into(content, arguments, context);
        return;
    };

    let mut visiting_constants = HashSet::new();
    for (index, argument) in arguments.iter().enumerate() {
        lower_function_arg_into(
            content,
            argument,
            signature.arguments().get(index),
            context,
            false,
            &mut visiting_constants,
        );
    }
}

fn lower_plain_divert_arguments_into(
    content: &mut Vec<RuntimeObject>,
    arguments: &[Expression],
    context: &LoweringContext<'_>,
) {
    for argument in arguments {
        lower_expression_into(content, argument, context, false);
    }
}

fn lower_static_divert_arguments_into(
    content: &mut Vec<RuntimeObject>,
    target: &DivertTarget,
    arguments: &[Expression],
    context: &LoweringContext<'_>,
) {
    let expected_args = static_divert_expected_args(target, context);
    let mut visiting_constants = HashSet::new();
    for (index, argument) in arguments.iter().enumerate() {
        lower_function_arg_into(
            content,
            argument,
            expected_args.and_then(|args| args.get(index)),
            context,
            false,
            &mut visiting_constants,
        );
    }
}

fn static_divert_expected_args<'a>(
    target: &DivertTarget,
    context: &'a LoweringContext<'_>,
) -> Option<&'a [FlowArgument]> {
    let target = static_divert_target_name(target)?;
    let signature_name = resolve_static_signature_name(target, context)?;
    let Some(CallSignature::Ink { args, .. }) = context.external_signatures().get(&signature_name)
    else {
        return None;
    };
    Some(args)
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

fn resolve_static_signature_name(target: &str, context: &LoweringContext<'_>) -> Option<String> {
    let signatures = context.external_signatures();
    if target.contains("::") {
        return signatures.contains_key(target).then(|| target.to_string());
    }

    let module = context.path_mode().current_module_name();
    let scoped = |name: &str| {
        module
            .map(|module| format!("{module}::{name}"))
            .unwrap_or_else(|| name.to_string())
    };

    if target.contains('.') {
        let candidate = scoped(target);
        return signatures.contains_key(&candidate).then_some(candidate);
    }

    let mut candidates = Vec::new();
    if let ChoicePathMode::Flow {
        flow_name,
        parent_flow_name,
        ..
    } = context.path_mode()
    {
        candidates.push(scoped(&format!("{flow_name}.{target}")));
        if let Some(parent_flow_name) = parent_flow_name {
            candidates.push(scoped(&format!("{parent_flow_name}.{target}")));
        }
    }
    candidates.push(scoped(target));

    candidates
        .into_iter()
        .find(|candidate| signatures.contains_key(candidate))
}

fn lower_tunnel_onwards_path_target_into(
    content: &mut Vec<RuntimeObject>,
    target: &str,
    context: &LoweringContext<'_>,
) {
    if let Some(choice_target) = context.choice_labels().get(target) {
        content.push(RuntimeObject::DivertTarget(choice_target.to_string()));
    } else if let Some(label_target) = context
        .path_mode()
        .scoped_label_target(target, context.global_labels())
        .filter(|label_target| *label_target != target)
    {
        content.push(RuntimeObject::DivertTarget(
            context.path_mode().resolve_label_target(label_target),
        ));
    } else if context.path_mode().is_local_variable(target)
        || context.global_variables().contains(target)
    {
        content.push(RuntimeObject::VariableReference(target.to_string()));
    } else {
        content.push(RuntimeObject::DivertTarget(
            context.path_mode().resolve_divert_target(target),
        ));
    }
}
