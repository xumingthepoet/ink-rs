use ink_story_json_format::{Container, ControlCommand, NativeFunction, Object as RuntimeObject};

use crate::parsed::{Sequence, SequenceType, Weave};

use super::context::LoweringContext;
use super::named_container;
use super::path::compact_relative_path;
use super::weave::{
    content_list_has_choice, lower_choice_weave_with_initial_content,
    lower_content_list_into_context,
};

pub(super) fn lower_sequence(
    sequence: &Sequence,
    context: &LoweringContext<'_>,
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
        content.push(RuntimeObject::NativeFunction(NativeFunction::Min));
    } else if cycle {
        content.push(RuntimeObject::Int(sequence.elements().len() as i32));
        content.push(RuntimeObject::NativeFunction(NativeFunction::Mod));
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
                RuntimeObject::NativeFunction(NativeFunction::Equal),
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
            RuntimeObject::NativeFunction(NativeFunction::Equal),
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
            let branch_path_mode = context
                .path_mode()
                .for_sequence_branch(sequence_container_path, &branch_name);
            let branch_context = context.with_path_mode(branch_path_mode);
            let mut branch_content = vec![RuntimeObject::ControlCommand(ControlCommand::Pop)];
            let mut branch_named_content = Vec::new();
            if let Some(element) = element {
                if content_list_has_choice(element) {
                    let element_weave = Weave::new(element.objects().to_vec(), 0);
                    let lowered_branch = lower_choice_weave_with_initial_content(
                        &element_weave,
                        &branch_context,
                        false,
                        branch_content,
                    );
                    branch_content = lowered_branch.content;
                    branch_named_content = lowered_branch.named_content;
                } else {
                    lower_content_list_into_context(&mut branch_content, element, &branch_context);
                }
            }
            let relative_return_target = format!(".^.^.{post_sequence_index}");
            let global_return_target = format!("{sequence_container_path}.{post_sequence_index}");
            branch_content.push(RuntimeObject::Divert {
                target: compact_relative_path(&relative_return_target, &global_return_target),
                variable: false,
            });
            Container {
                content: branch_content,
                named_content: branch_named_content,
                name: Some(branch_name),
                flags: None,
            }
        })
        .map(named_container)
        .collect::<Vec<_>>();

    Container {
        content,
        named_content: branch_containers,
        name: None,
        flags: Some(5),
    }
}
