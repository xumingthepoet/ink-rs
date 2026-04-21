use std::sync::atomic::{AtomicI32, Ordering};

const EXPECTED_MAX_STACK_DEPTH: usize = 200;
static UNIQUE_ID_COUNTER: AtomicI32 = AtomicI32::new(0);

fn next_unique_id() -> i32 {
    UNIQUE_ID_COUNTER.fetch_add(1, Ordering::SeqCst) + 1
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub character_index: i32,
    pub character_in_line_index: i32,
    pub line_index: i32,
    pub reported_error_in_scope: bool,
    pub unique_id: i32,
    pub custom_flags: u32,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            character_index: 0,
            character_in_line_index: 0,
            line_index: 0,
            reported_error_in_scope: false,
            unique_id: 0,
            custom_flags: 0,
        }
    }
}

impl Element {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn copy_from(&mut self, from_element: &Element) {
        self.unique_id = next_unique_id();
        self.character_index = from_element.character_index;
        self.character_in_line_index = from_element.character_in_line_index;
        self.line_index = from_element.line_index;
        self.custom_flags = from_element.custom_flags;
        self.reported_error_in_scope = false;
    }

    pub fn squash_from(&mut self, from_element: &Element) {
        self.character_index = from_element.character_index;
        self.character_in_line_index = from_element.character_in_line_index;
        self.line_index = from_element.line_index;
        self.reported_error_in_scope = from_element.reported_error_in_scope;
        self.custom_flags = from_element.custom_flags;
    }
}

#[derive(Debug, Clone)]
pub struct StringParserState {
    stack: Vec<Element>,
}

impl StringParserState {
    pub fn new() -> Self {
        let mut stack = Vec::with_capacity(EXPECTED_MAX_STACK_DEPTH);
        stack.push(Element::new());

        Self { stack }
    }

    pub fn line_index(&self) -> i32 {
        self.current_element().line_index
    }

    pub fn set_line_index(&mut self, value: i32) {
        self.current_element_mut().line_index = value;
    }

    pub fn character_index(&self) -> i32 {
        self.current_element().character_index
    }

    pub fn set_character_index(&mut self, value: i32) {
        self.current_element_mut().character_index = value;
    }

    pub fn character_in_line_index(&self) -> i32 {
        self.current_element().character_in_line_index
    }

    pub fn set_character_in_line_index(&mut self, value: i32) {
        self.current_element_mut().character_in_line_index = value;
    }

    pub fn custom_flags(&self) -> u32 {
        self.current_element().custom_flags
    }

    pub fn set_custom_flags(&mut self, value: u32) {
        self.current_element_mut().custom_flags = value;
    }

    pub fn error_reported_already_in_scope(&self) -> bool {
        self.current_element().reported_error_in_scope
    }

    pub fn stack_height(&self) -> usize {
        self.stack.len()
    }

    pub fn push(&mut self) -> i32 {
        if self.stack.len() >= EXPECTED_MAX_STACK_DEPTH {
            panic!("Stack overflow in parser state");
        }

        let mut new_element = Element::new();
        new_element.copy_from(self.current_element());
        self.stack.push(new_element);

        self.current_element().unique_id
    }

    pub fn pop(&mut self, expected_rule_id: i32) {
        if self.stack.len() == 1 {
            panic!("Attempting to remove final stack element is illegal! Mismatched Begin/Succceed/Fail?");
        }

        if self.current_element().unique_id != expected_rule_id {
            panic!("Mismatched rule IDs - do you have mismatched Begin/Succeed/Fail?");
        }

        self.stack.pop();
    }

    pub fn peek(&self, expected_rule_id: i32) -> &Element {
        if self.current_element().unique_id != expected_rule_id {
            panic!("Mismatched rule IDs - do you have mismatched Begin/Succeed/Fail?");
        }

        self.current_element()
    }

    pub fn peek_penultimate(&self) -> Option<&Element> {
        self.stack
            .len()
            .checked_sub(2)
            .and_then(|index| self.stack.get(index))
    }

    pub fn squash(&mut self) {
        if self.stack.len() < 2 {
            panic!("Attempting to remove final stack element is illegal! Mismatched Begin/Succceed/Fail?");
        }

        let last_element = self.stack[self.stack.len() - 1].clone();
        let penultimate_index = self.stack.len() - 2;
        self.stack[penultimate_index].squash_from(&last_element);
        self.stack.pop();
    }

    pub fn note_error_reported(&mut self) {
        for element in &mut self.stack {
            element.reported_error_in_scope = true;
        }
    }

    fn current_element(&self) -> &Element {
        self.stack
            .last()
            .expect("parser state always maintains one stack element")
    }

    fn current_element_mut(&mut self) -> &mut Element {
        self.stack
            .last_mut()
            .expect("parser state always maintains one stack element")
    }
}

impl Default for StringParserState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::StringParserState;

    #[test]
    fn string_parser_state_starts_with_one_element() {
        let state = StringParserState::new();

        assert_eq!(state.stack_height(), 1);
        assert_eq!(state.line_index(), 0);
        assert_eq!(state.character_index(), 0);
        assert_eq!(state.character_in_line_index(), 0);
        assert_eq!(state.custom_flags(), 0);
        assert!(!state.error_reported_already_in_scope());
        assert!(state.peek_penultimate().is_none());
    }

    #[test]
    fn string_parser_state_push_copies_previous_values() {
        let mut state = StringParserState::new();
        state.set_line_index(3);
        state.set_character_index(9);
        state.set_character_in_line_index(4);
        state.set_custom_flags(0x12);

        let prior_id = state
            .peek_penultimate()
            .map(|element| element.unique_id)
            .unwrap_or(0);
        let pushed_id = state.push();

        assert_eq!(state.stack_height(), 2);
        assert_eq!(state.line_index(), 3);
        assert_eq!(state.character_index(), 9);
        assert_eq!(state.character_in_line_index(), 4);
        assert_eq!(state.custom_flags(), 0x12);
        assert_eq!(state.peek(pushed_id).unique_id, pushed_id);
        assert_ne!(state.peek(pushed_id).unique_id, prior_id);
    }

    #[test]
    fn string_parser_state_pop_restores_previous_element() {
        let mut state = StringParserState::new();
        let rule_id = state.push();
        state.set_line_index(7);
        state.set_character_index(11);

        state.pop(rule_id);

        assert_eq!(state.stack_height(), 1);
        assert_eq!(state.line_index(), 0);
        assert_eq!(state.character_index(), 0);
    }

    #[test]
    fn string_parser_state_squash_merges_top_element() {
        let mut state = StringParserState::new();
        state.set_line_index(1);
        state.set_character_index(2);
        state.set_custom_flags(0x10);
        state.push();
        state.set_line_index(4);
        state.set_character_index(8);
        state.set_character_in_line_index(6);
        state.set_custom_flags(0x20);
        state.squash();

        assert_eq!(state.stack_height(), 1);
        assert_eq!(state.line_index(), 4);
        assert_eq!(state.character_index(), 8);
        assert_eq!(state.character_in_line_index(), 6);
        assert_eq!(state.custom_flags(), 0x20);
    }

    #[test]
    fn string_parser_state_marks_error_scope_across_stack() {
        let mut state = StringParserState::new();
        state.push();

        assert!(!state.error_reported_already_in_scope());
        state.note_error_reported();

        assert!(state.error_reported_already_in_scope());
        assert!(state.peek_penultimate().unwrap().reported_error_in_scope);
    }

    #[test]
    #[should_panic(expected = "Mismatched rule IDs")]
    fn string_parser_state_rejects_mismatched_pop_ids() {
        let mut state = StringParserState::new();
        state.push();
        state.pop(999);
    }
}
