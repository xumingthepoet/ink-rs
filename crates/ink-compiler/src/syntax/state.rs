#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParserState {
    stack: Vec<StateFrame>,
    next_rule_id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StateFrame {
    rule_id: usize,
    byte_index: usize,
    character_in_line: usize,
    reported_error_in_scope: bool,
}

impl ParserState {
    pub(super) fn new() -> Self {
        Self {
            stack: vec![StateFrame {
                rule_id: 0,
                byte_index: 0,
                character_in_line: 0,
                reported_error_in_scope: false,
            }],
            next_rule_id: 1,
        }
    }

    pub(super) fn begin_rule(&mut self) -> usize {
        let mut frame = self.current().clone();
        frame.rule_id = self.next_rule_id;
        frame.reported_error_in_scope = false;
        self.next_rule_id += 1;
        let rule_id = frame.rule_id;
        self.stack.push(frame);
        rule_id
    }

    pub(super) fn fail_rule(&mut self, expected_rule_id: usize) {
        self.pop(expected_rule_id);
    }

    pub(super) fn succeed_rule(&mut self, expected_rule_id: usize) {
        assert_eq!(
            self.current().rule_id,
            expected_rule_id,
            "mismatched parser rule id"
        );

        let succeeded = self
            .stack
            .pop()
            .expect("parser state stack cannot be empty");
        let parent = self
            .stack
            .last_mut()
            .expect("parser state must retain the root frame");
        parent.byte_index = succeeded.byte_index;
        parent.character_in_line = succeeded.character_in_line;
        parent.reported_error_in_scope = succeeded.reported_error_in_scope;
    }

    pub(super) fn byte_index(&self) -> usize {
        self.current().byte_index
    }

    pub(super) fn character_in_line(&self) -> usize {
        self.current().character_in_line
    }

    pub(super) fn advance(&mut self, byte_len: usize) {
        let current = self.current_mut();
        current.byte_index += byte_len;
        current.character_in_line += 1;
    }

    pub(super) fn set_position(&mut self, byte_index: usize, character_in_line: usize) {
        let current = self.current_mut();
        current.byte_index = byte_index;
        current.character_in_line = character_in_line;
    }

    pub(super) fn error_reported_in_scope(&self) -> bool {
        self.current().reported_error_in_scope
    }

    pub(super) fn note_error_reported(&mut self) {
        for frame in &mut self.stack {
            frame.reported_error_in_scope = true;
        }
    }

    fn pop(&mut self, expected_rule_id: usize) {
        assert!(
            self.stack.len() > 1,
            "attempted to pop the root parser state frame"
        );
        assert_eq!(
            self.current().rule_id,
            expected_rule_id,
            "mismatched parser rule id"
        );
        self.stack.pop();
    }

    fn current(&self) -> &StateFrame {
        self.stack
            .last()
            .expect("parser state must have a current frame")
    }

    fn current_mut(&mut self) -> &mut StateFrame {
        self.stack
            .last_mut()
            .expect("parser state must have a current frame")
    }
}
