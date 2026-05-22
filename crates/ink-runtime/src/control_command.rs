use std::fmt;

use ink_story_json_format as format;
use strum::Display;

use crate::object::{Object, RTObject};

#[derive(PartialEq, Display, Clone, Copy, Debug)]
pub enum CommandType {
    EvalStart,
    EvalOutput,
    EvalEnd,
    Duplicate,
    PopEvaluatedValue,
    PopFunction,
    PopTunnel,
    BeginString,
    EndString,
    NoOp,
    ChoiceCount,
    Turns,
    TurnsSince,
    ReadCount,
    Random,
    SeedRandom,
    VisitIndex,
    SequenceShuffleIndex,
    LegacyStartThread,
    Done,
    End,
    BeginTag,
    EndTag,
}

impl CommandType {
    fn from_format(command: format::ControlCommand) -> Self {
        match command {
            format::ControlCommand::Done => Self::Done,
            format::ControlCommand::End => Self::End,
            format::ControlCommand::EvalStart => Self::EvalStart,
            format::ControlCommand::EvalOutput => Self::EvalOutput,
            format::ControlCommand::EvalEnd => Self::EvalEnd,
            format::ControlCommand::BeginString => Self::BeginString,
            format::ControlCommand::EndString => Self::EndString,
            format::ControlCommand::VisitIndex => Self::VisitIndex,
            format::ControlCommand::SequenceShuffleIndex => Self::SequenceShuffleIndex,
            format::ControlCommand::Duplicate => Self::Duplicate,
            format::ControlCommand::NoOp => Self::NoOp,
            format::ControlCommand::Pop => Self::PopEvaluatedValue,
            format::ControlCommand::PopFunction => Self::PopFunction,
            format::ControlCommand::PopTunnel => Self::PopTunnel,
            format::ControlCommand::StartThread => Self::LegacyStartThread,
            format::ControlCommand::ChoiceCount => Self::ChoiceCount,
            format::ControlCommand::Turns => Self::Turns,
            format::ControlCommand::TurnsSince => Self::TurnsSince,
            format::ControlCommand::ReadCount => Self::ReadCount,
            format::ControlCommand::Random => Self::Random,
            format::ControlCommand::SeedRandom => Self::SeedRandom,
        }
    }

    fn to_format(self) -> Option<format::ControlCommand> {
        match self {
            Self::Done => Some(format::ControlCommand::Done),
            Self::End => Some(format::ControlCommand::End),
            Self::EvalStart => Some(format::ControlCommand::EvalStart),
            Self::EvalOutput => Some(format::ControlCommand::EvalOutput),
            Self::EvalEnd => Some(format::ControlCommand::EvalEnd),
            Self::BeginString => Some(format::ControlCommand::BeginString),
            Self::EndString => Some(format::ControlCommand::EndString),
            Self::VisitIndex => Some(format::ControlCommand::VisitIndex),
            Self::SequenceShuffleIndex => Some(format::ControlCommand::SequenceShuffleIndex),
            Self::Duplicate => Some(format::ControlCommand::Duplicate),
            Self::NoOp => Some(format::ControlCommand::NoOp),
            Self::PopEvaluatedValue => Some(format::ControlCommand::Pop),
            Self::PopFunction => Some(format::ControlCommand::PopFunction),
            Self::PopTunnel => Some(format::ControlCommand::PopTunnel),
            Self::LegacyStartThread => Some(format::ControlCommand::StartThread),
            Self::ChoiceCount => Some(format::ControlCommand::ChoiceCount),
            Self::Turns => Some(format::ControlCommand::Turns),
            Self::TurnsSince => Some(format::ControlCommand::TurnsSince),
            Self::ReadCount => Some(format::ControlCommand::ReadCount),
            Self::Random => Some(format::ControlCommand::Random),
            Self::SeedRandom => Some(format::ControlCommand::SeedRandom),
            Self::BeginTag | Self::EndTag => None,
        }
    }

    fn from_token(token: &str) -> Option<Self> {
        match token {
            format::TAG_START_TOKEN => Some(Self::BeginTag),
            format::TAG_END_TOKEN => Some(Self::EndTag),
            _ => format::ControlCommand::from_token(token).map(Self::from_format),
        }
    }

    fn token(self) -> &'static str {
        match self {
            Self::BeginTag => format::TAG_START_TOKEN,
            Self::EndTag => format::TAG_END_TOKEN,
            _ => self
                .to_format()
                .expect("non-tag runtime command must map to a format command")
                .token(),
        }
    }
}

pub struct ControlCommand {
    obj: Object,
    pub command_type: CommandType,
}

impl ControlCommand {
    pub fn new_from_name(name: &str) -> Option<Self> {
        CommandType::from_token(name).map(Self::new)
    }

    pub fn get_name(c: CommandType) -> String {
        c.token().to_string()
    }

    pub fn new(command_type: CommandType) -> Self {
        Self {
            obj: Object::new(),
            command_type,
        }
    }
}

impl RTObject for ControlCommand {
    fn get_object(&self) -> &Object {
        &self.obj
    }
}

impl fmt::Display for ControlCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.command_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_command_tokens_round_trip_through_format_tokens() {
        let commands = [
            format::ControlCommand::Done,
            format::ControlCommand::End,
            format::ControlCommand::EvalStart,
            format::ControlCommand::EvalOutput,
            format::ControlCommand::EvalEnd,
            format::ControlCommand::BeginString,
            format::ControlCommand::EndString,
            format::ControlCommand::VisitIndex,
            format::ControlCommand::SequenceShuffleIndex,
            format::ControlCommand::Duplicate,
            format::ControlCommand::NoOp,
            format::ControlCommand::Pop,
            format::ControlCommand::PopFunction,
            format::ControlCommand::PopTunnel,
            format::ControlCommand::StartThread,
            format::ControlCommand::ChoiceCount,
            format::ControlCommand::Turns,
            format::ControlCommand::TurnsSince,
            format::ControlCommand::ReadCount,
            format::ControlCommand::Random,
            format::ControlCommand::SeedRandom,
        ];

        for format_command in commands {
            let runtime_command = ControlCommand::new_from_name(format_command.token())
                .expect("format command token should parse as runtime command");

            assert_eq!(
                ControlCommand::get_name(runtime_command.command_type),
                format_command.token()
            );
        }
    }

    #[test]
    fn runtime_tag_tokens_use_format_tag_tokens() {
        let begin_tag = ControlCommand::new_from_name(format::TAG_START_TOKEN)
            .expect("format start-tag token should parse");
        let end_tag = ControlCommand::new_from_name(format::TAG_END_TOKEN)
            .expect("format end-tag token should parse");

        assert_eq!(begin_tag.command_type, CommandType::BeginTag);
        assert_eq!(end_tag.command_type, CommandType::EndTag);
        assert_eq!(
            ControlCommand::get_name(CommandType::BeginTag),
            format::TAG_START_TOKEN
        );
        assert_eq!(
            ControlCommand::get_name(CommandType::EndTag),
            format::TAG_END_TOKEN
        );
    }
}
