#[derive(Debug, Default)]
pub struct CommentEliminator {
    chars: Vec<char>,
    index: usize,
}

impl CommentEliminator {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            chars: input.into().chars().collect(),
            index: 0,
        }
    }

    pub fn process(input: impl Into<String>) -> Option<String> {
        let mut eliminator = Self::new(input);
        eliminator.process_inner()
    }

    fn process_inner(&mut self) -> Option<String> {
        let mut output = String::new();

        while let Some(character) = self.current_character() {
            if self.starts_with("//") {
                self.index += 2;
                if let Some(newline) = self.consume_comment_to_line_end() {
                    output.push(newline);
                }
                continue;
            }

            if self.starts_with("/*") {
                self.index += 2;
                let newline_count = self.consume_block_comment();
                output.extend(std::iter::repeat_n('\n', newline_count));
                continue;
            }

            if character == '\r' {
                self.index += 1;
                if self.current_character() == Some('\n') {
                    self.index += 1;
                }
                output.push('\n');
                continue;
            }

            if character == '\n' {
                self.index += 1;
                output.push('\n');
                continue;
            }

            output.push(character);
            self.index += 1;
        }

        if output.is_empty() {
            None
        } else {
            Some(output)
        }
    }

    fn consume_comment_to_line_end(&mut self) -> Option<char> {
        while let Some(character) = self.current_character() {
            match character {
                '\n' => {
                    self.index += 1;
                    return Some('\n');
                }
                '\r' => {
                    self.index += 1;
                    if self.current_character() == Some('\n') {
                        self.index += 1;
                    }
                    return Some('\n');
                }
                _ => self.index += 1,
            }
        }

        None
    }

    fn consume_block_comment(&mut self) -> usize {
        let mut newline_count = 0;

        while let Some(character) = self.current_character() {
            if character == '*' && self.peek_next_character() == Some('/') {
                self.index += 2;
                break;
            }

            match character {
                '\n' => {
                    newline_count += 1;
                    self.index += 1;
                }
                '\r' => {
                    newline_count += 1;
                    self.index += 1;
                    if self.current_character() == Some('\n') {
                        self.index += 1;
                    }
                }
                _ => self.index += 1,
            }
        }

        newline_count
    }

    fn current_character(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn peek_next_character(&self) -> Option<char> {
        self.chars.get(self.index + 1).copied()
    }

    fn starts_with(&self, prefix: &str) -> bool {
        let prefix_chars: Vec<char> = prefix.chars().collect();
        self.chars[self.index..].starts_with(&prefix_chars)
    }
}

#[cfg(test)]
mod tests {
    use super::CommentEliminator;

    #[test]
    fn comment_eliminator_removes_line_comments_and_keeps_newlines() {
        let processed = CommentEliminator::process("hello // comment\r\nworld")
            .expect("expected retained content");

        assert_eq!(processed, "hello \nworld");
    }

    #[test]
    fn comment_eliminator_removes_block_comments_and_preserves_line_count() {
        let processed = CommentEliminator::process("before /* one\n two\r\nthree */ after")
            .expect("expected retained content");

        assert_eq!(processed, "before \n\n after");
    }

    #[test]
    fn comment_eliminator_returns_none_for_comment_only_input() {
        assert_eq!(CommentEliminator::process("// only comment"), None);
        assert_eq!(CommentEliminator::process("/* only comment */"), None);
    }
}
