pub mod state;

use std::fmt;

use crate::{
    error::{Diagnostic, DiagnosticSeverity},
    parsed::DebugMetadata,
};

use super::character_set::CharacterSet;

pub use state::{Element, StringParserState};

pub type ErrorHandler = Box<dyn FnMut(&Diagnostic)>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ParseSuccessStruct;

pub const PARSE_SUCCESS: ParseSuccessStruct = ParseSuccessStruct;

pub fn is_newline(character: char) -> bool {
    matches!(character, '\n' | '\r')
}

pub fn is_whitespace(character: char) -> bool {
    matches!(character, '\n' | '\r' | '\t' | ' ')
}

pub fn is_inline_whitespace(character: char) -> bool {
    matches!(character, '\t' | ' ')
}

pub fn is_digit(character: char) -> bool {
    character.is_ascii_digit()
}

pub fn is_letter(character: char) -> bool {
    character.is_alphabetic()
}

pub fn is_identifier_character(character: char) -> bool {
    is_letter(character) || is_digit(character) || matches!(character, '_')
}

pub fn newline_characters() -> CharacterSet {
    CharacterSet::from("\n\r")
}

pub fn inline_whitespace_characters() -> CharacterSet {
    CharacterSet::from(" \t")
}

pub fn digit_characters() -> CharacterSet {
    CharacterSet::from("0123456789")
}

#[derive(Default)]
pub struct StringParser {
    chars: Vec<char>,
    input_string: String,
    source_filename: Option<String>,
    state: StringParserState,
    error_handler: Option<ErrorHandler>,
    diagnostics: Vec<Diagnostic>,
    had_error: bool,
}

impl fmt::Debug for StringParser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StringParser")
            .field("input_string", &self.input_string)
            .field("source_filename", &self.source_filename)
            .field("state", &self.state)
            .field("diagnostics", &self.diagnostics)
            .field("had_error", &self.had_error)
            .finish()
    }
}

impl StringParser {
    pub fn new(str: impl Into<String>) -> Self {
        let input_string = Self::pre_process_input_string(str.into());
        let chars = input_string.chars().collect();

        Self {
            chars,
            input_string,
            source_filename: None,
            state: StringParserState::new(),
            error_handler: None,
            diagnostics: Vec::new(),
            had_error: false,
        }
    }

    pub fn new_with_source_filename(
        str: impl Into<String>,
        source_filename: Option<impl Into<String>>,
    ) -> Self {
        let mut parser = Self::new(str);
        parser.source_filename = source_filename.map(Into::into);
        parser
    }

    pub fn with_source_filename(mut self, source_filename: impl Into<String>) -> Self {
        self.source_filename = Some(source_filename.into());
        self
    }

    pub fn set_error_handler<F>(&mut self, error_handler: F)
    where
        F: FnMut(&Diagnostic) + 'static,
    {
        self.error_handler = Some(Box::new(error_handler));
    }

    pub fn input_string(&self) -> &str {
        &self.input_string
    }

    pub fn source_filename(&self) -> Option<&str> {
        self.source_filename.as_deref()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn had_error(&self) -> bool {
        self.had_error
    }

    pub fn state(&self) -> &StringParserState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut StringParserState {
        &mut self.state
    }

    pub fn current_character(&self) -> char {
        if self.index() < self.chars.len() && self.remaining_length() > 0 {
            self.chars[self.index()]
        } else {
            '\0'
        }
    }

    pub fn end_of_input(&self) -> bool {
        self.index() >= self.chars.len()
    }

    pub fn remaining_string(&self) -> String {
        self.chars[self.index()..].iter().collect()
    }

    pub fn line_remainder(&mut self) -> Option<String> {
        self.peek(|parser| parser.parse_until_characters_from_string("\n\r", None))
    }

    pub fn remaining_length(&self) -> usize {
        self.chars.len().saturating_sub(self.index())
    }

    pub fn line_index(&self) -> i32 {
        self.state.line_index()
    }

    pub fn set_line_index(&mut self, value: i32) {
        self.state.set_line_index(value);
    }

    pub fn character_in_line_index(&self) -> i32 {
        self.state.character_in_line_index()
    }

    pub fn set_character_in_line_index(&mut self, value: i32) {
        self.state.set_character_in_line_index(value);
    }

    pub fn index(&self) -> usize {
        self.state.character_index().max(0) as usize
    }

    fn set_index(&mut self, value: usize) {
        self.state.set_character_index(value as i32);
    }

    pub fn set_flag(&mut self, flag: u32, true_or_false: bool) {
        let current_flags = self.state.custom_flags();
        let next_flags = if true_or_false {
            current_flags | flag
        } else {
            current_flags & !flag
        };

        self.state.set_custom_flags(next_flags);
    }

    pub fn get_flag(&self, flag: u32) -> bool {
        (self.state.custom_flags() & flag) != 0
    }

    pub fn begin_rule(&mut self) -> i32 {
        self.state.push()
    }

    pub fn fail_rule(&mut self, expected_rule_id: i32) {
        self.state.pop(expected_rule_id);
    }

    pub fn cancel_rule(&mut self, expected_rule_id: i32) {
        self.state.pop(expected_rule_id);
    }

    pub fn succeed_rule<T>(&mut self, expected_rule_id: i32, result: T) -> T {
        let end_state = self.state.peek(expected_rule_id).clone();
        let start_state = self.state.peek_penultimate().cloned();

        self.rule_did_succeed(&result, start_state.as_ref(), &end_state);
        self.state.squash();

        result
    }

    fn rule_did_succeed<T>(
        &mut self,
        _result: &T,
        _state_at_start: Option<&Element>,
        _state_at_end: &Element,
    ) {
    }

    pub fn parse_object<T, F>(&mut self, mut rule: F) -> Option<T>
    where
        F: FnMut(&mut Self) -> Option<T>,
    {
        let rule_id = self.begin_rule();
        let stack_height_before = self.state.stack_height();
        let result = rule(self);

        if stack_height_before != self.state.stack_height() {
            panic!("Mismatched Begin/Fail/Succeed rules");
        }

        match result {
            Some(value) => Some(self.succeed_rule(rule_id, value)),
            None => {
                self.fail_rule(rule_id);
                None
            }
        }
    }

    pub fn parse<T, F>(&mut self, rule: F) -> Option<T>
    where
        F: FnMut(&mut Self) -> Option<T>,
    {
        self.parse_object(rule)
    }

    pub fn peek<T, F>(&mut self, mut rule: F) -> Option<T>
    where
        F: FnMut(&mut Self) -> Option<T>,
    {
        let rule_id = self.begin_rule();
        let result = rule(self);
        self.cancel_rule(rule_id);
        result
    }

    pub fn parse_string(&mut self, str: &str) -> Option<String> {
        let expected_length = str.chars().count();
        if expected_length > self.remaining_length() {
            return None;
        }

        let rule_id = self.begin_rule();
        let start_index = self.index();

        let mut i = self.index();
        let mut cli = self.character_in_line_index();
        let mut li = self.line_index();

        let mut success = true;
        for expected_character in str.chars() {
            let current_character = self.chars[i];
            if current_character != expected_character {
                success = false;
                break;
            }

            if current_character == '\n' {
                li += 1;
                cli = -1;
            }

            i += 1;
            cli += 1;
        }

        self.set_index(i);
        self.set_character_in_line_index(cli);
        self.set_line_index(li);

        if success {
            Some(self.succeed_rule(rule_id, str.to_string()))
        } else {
            self.fail_rule(rule_id);
            let _ = start_index;
            None
        }
    }

    pub fn parse_single_character(&mut self) -> Option<char> {
        if self.remaining_length() == 0 {
            return None;
        }

        let character = self.current_character();
        self.consume_character(character);
        Some(character)
    }

    pub fn parse_until_characters_from_string(
        &mut self,
        str: &str,
        max_count: Option<usize>,
    ) -> Option<String> {
        self.parse_characters_from_char_set(&CharacterSet::from(str), false, max_count)
    }

    pub fn parse_until_characters_from_char_set(
        &mut self,
        char_set: &CharacterSet,
        max_count: Option<usize>,
    ) -> Option<String> {
        self.parse_characters_from_char_set(char_set, false, max_count)
    }

    pub fn parse_characters_from_string(
        &mut self,
        str: &str,
        should_include_str_chars: bool,
        max_count: Option<usize>,
    ) -> Option<String> {
        self.parse_characters_from_char_set(
            &CharacterSet::from(str),
            should_include_str_chars,
            max_count,
        )
    }

    pub fn parse_characters_from_char_set(
        &mut self,
        char_set: &CharacterSet,
        should_include_chars: bool,
        max_count: Option<usize>,
    ) -> Option<String> {
        let max_count = max_count.unwrap_or(usize::MAX);
        let start_index = self.index();

        let mut i = self.index();
        let mut cli = self.character_in_line_index();
        let mut li = self.line_index();
        let mut count = 0;

        while i < self.chars.len()
            && char_set.contains(&self.chars[i]) == should_include_chars
            && count < max_count
        {
            let character = self.chars[i];
            if character == '\n' {
                li += 1;
                cli = -1;
            }

            i += 1;
            cli += 1;
            count += 1;
        }

        self.set_index(i);
        self.set_character_in_line_index(cli);
        self.set_line_index(li);

        if i > start_index {
            Some(self.chars[start_index..i].iter().collect())
        } else {
            None
        }
    }

    pub fn parse_until<T, F>(
        &mut self,
        mut stop_rule: F,
        pause_characters: Option<&CharacterSet>,
        end_characters: Option<&CharacterSet>,
    ) -> Option<String>
    where
        F: FnMut(&mut Self) -> Option<T>,
    {
        let rule_id = self.begin_rule();

        let mut pause_and_end = CharacterSet::new();
        if let Some(pause_characters) = pause_characters {
            pause_and_end.union_with(pause_characters);
        }
        if let Some(end_characters) = end_characters {
            pause_and_end.union_with(end_characters);
        }

        let mut parsed_string = String::new();

        loop {
            if let Some(partial_parsed_string) =
                self.parse_until_characters_from_char_set(&pause_and_end, None)
            {
                parsed_string.push_str(&partial_parsed_string);
            }

            let rule_result_at_pause = self.peek(|parser| stop_rule(parser));

            if rule_result_at_pause.is_some() {
                break;
            }

            if self.end_of_input() {
                break;
            }

            let pause_character = self.current_character();
            if pause_characters
                .map(|characters| characters.contains(&pause_character))
                .unwrap_or(false)
            {
                parsed_string.push(pause_character);
                self.consume_character(pause_character);
                continue;
            } else {
                break;
            }
        }

        if parsed_string.is_empty() {
            self.fail_rule(rule_id);
            None
        } else {
            Some(self.succeed_rule(rule_id, parsed_string))
        }
    }

    pub fn expect<T, F>(&mut self, mut rule: F, message: Option<&str>) -> Option<T>
    where
        F: FnMut(&mut Self) -> Option<T>,
    {
        let result = self.parse_object(|parser| rule(parser));
        if result.is_none() {
            let message = message.unwrap_or("expected rule");
            let line_remainder = self.line_remainder();
            let but_saw = match line_remainder.as_ref() {
                Some(line_remainder) if !line_remainder.is_empty() => {
                    format!("'{}'", line_remainder)
                }
                _ => "end of line".to_string(),
            };
            self.error(format!("Expected {message} but saw {but_saw}"), false);
        }

        result
    }

    pub fn error(&mut self, message: impl Into<String>, is_warning: bool) {
        self.error_on_line(message.into(), self.line_index() + 1, is_warning);
    }

    pub fn error_with_debug_metadata(
        &mut self,
        message: impl Into<String>,
        debug_metadata: &DebugMetadata,
        is_warning: bool,
    ) {
        self.error_on_line(
            message.into(),
            debug_metadata.start_line_number as i32,
            is_warning,
        );
    }

    pub fn warning(&mut self, message: impl Into<String>) {
        self.error(message, true);
    }

    pub fn create_debug_metadata(
        &self,
        state_at_start: &Element,
        state_at_end: &Element,
    ) -> DebugMetadata {
        DebugMetadata {
            source_name: self.source_filename.clone(),
            start_line_number: (state_at_start.line_index + 1).max(1) as usize,
            end_line_number: (state_at_end.line_index + 1).max(1) as usize,
            start_character_number: (state_at_start.character_in_line_index + 1).max(1) as usize,
            end_character_number: (state_at_end.character_in_line_index + 1).max(1) as usize,
        }
    }

    pub fn parse_newline(&mut self) -> Option<String> {
        let rule_id = self.begin_rule();

        let _ = self.parse_string("\r");

        if self.parse_string("\n").is_none() {
            self.fail_rule(rule_id);
            None
        } else {
            Some(self.succeed_rule(rule_id, "\n".to_string()))
        }
    }

    fn error_on_line(&mut self, message: String, line_number: i32, is_warning: bool) {
        if !self.state.error_reported_already_in_scope() {
            let diagnostic = Diagnostic::new(
                if is_warning {
                    DiagnosticSeverity::Warning
                } else {
                    DiagnosticSeverity::Error
                },
                self.source_filename.clone(),
                line_number.max(1) as usize,
                self.character_in_line_index().max(0) as usize + 1,
                message,
            );

            self.diagnostics.push(diagnostic.clone());
            if let Some(error_handler) = self.error_handler.as_mut() {
                error_handler(&diagnostic);
            }

            self.state.note_error_reported();
        }

        if !is_warning {
            self.had_error = true;
        }
    }

    fn consume_character(&mut self, character: char) {
        if character == '\n' {
            self.set_line_index(self.line_index() + 1);
            self.set_character_in_line_index(-1);
        }

        self.set_index(self.index() + 1);
        self.set_character_in_line_index(self.character_in_line_index() + 1);
    }

    fn pre_process_input_string(str: String) -> String {
        str
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{parsed::DebugMetadata, DiagnosticSeverity};

    use super::{
        digit_characters, inline_whitespace_characters, newline_characters, CharacterSet,
        StringParser,
    };

    #[test]
    fn string_parser_tracks_cursor_and_newlines() {
        let mut parser = StringParser::new("a\nbc");

        assert_eq!(parser.current_character(), 'a');
        assert_eq!(parser.remaining_length(), 4);
        assert_eq!(parser.parse_single_character(), Some('a'));
        assert_eq!(parser.line_index(), 0);
        assert_eq!(parser.character_in_line_index(), 1);

        assert_eq!(parser.parse_newline(), Some("\n".to_string()));
        assert_eq!(parser.line_index(), 1);
        assert_eq!(parser.character_in_line_index(), 0);
        assert_eq!(parser.current_character(), 'b');
    }

    #[test]
    fn string_parser_parses_character_sets_and_line_remainder() {
        let mut parser = StringParser::new("abc def\nghi");

        let letters = parser
            .parse_characters_from_char_set(&CharacterSet::from("abc"), true, None)
            .expect("expected prefix");
        assert_eq!(letters, "abc");
        assert_eq!(parser.current_character(), ' ');
        assert_eq!(parser.line_remainder(), Some(" def".to_string()));
        assert!(newline_characters().contains(&'\n'));

        let whitespace = parser
            .parse_characters_from_char_set(&inline_whitespace_characters(), true, None)
            .expect("expected whitespace");
        assert_eq!(whitespace, " ");
        let digits = parser.parse_characters_from_char_set(&digit_characters(), true, None);
        assert!(digits.is_none());
    }

    #[test]
    fn string_parser_parse_and_peek_restore_or_commit_state() {
        let mut parser = StringParser::new("hello");
        let peeked = parser.peek(|parser| parser.parse_string("he"));
        assert_eq!(peeked, Some("he".to_string()));
        assert_eq!(parser.current_character(), 'h');

        let parsed = parser.parse_object(|parser| parser.parse_string("he"));
        assert_eq!(parsed, Some("he".to_string()));
        assert_eq!(parser.current_character(), 'l');
    }

    #[test]
    fn string_parser_emits_line_based_diagnostics_and_debug_metadata() {
        let diagnostics = Rc::new(RefCell::new(Vec::new()));
        let mut parser = StringParser::new("alpha");
        let sink = diagnostics.clone();
        parser.set_error_handler(move |diagnostic| {
            sink.borrow_mut().push(diagnostic.clone());
        });

        parser.parse_single_character();
        parser.parse_single_character();
        parser.warning("careful");

        let collected = diagnostics.borrow();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].severity, DiagnosticSeverity::Warning);
        assert_eq!(collected[0].line, 1);
        assert_eq!(collected[0].column, 3);
        assert!(!parser.had_error());

        let start = super::state::Element {
            character_index: 1,
            character_in_line_index: 1,
            line_index: 2,
            reported_error_in_scope: false,
            unique_id: 10,
            custom_flags: 0,
        };
        let end = super::state::Element {
            character_index: 3,
            character_in_line_index: 4,
            line_index: 5,
            reported_error_in_scope: false,
            unique_id: 11,
            custom_flags: 0,
        };

        let metadata = parser.create_debug_metadata(&start, &end);
        assert_eq!(
            metadata,
            DebugMetadata {
                source_name: None,
                start_line_number: 3,
                end_line_number: 6,
                start_character_number: 2,
                end_character_number: 5,
            }
        );
    }
}
