use super::StoryState;

impl StoryState {
    pub fn has_error(&self) -> bool {
        !self.current_errors.is_empty()
    }

    pub fn has_warning(&self) -> bool {
        !self.current_warnings.is_empty()
    }

    pub fn get_current_errors(&self) -> &Vec<String> {
        &self.current_errors
    }

    pub fn get_current_warnings(&self) -> &Vec<String> {
        &self.current_warnings
    }

    pub(crate) fn add_error(&mut self, message: String, is_warning: bool) {
        if !is_warning {
            self.current_errors.push(message);
        } else {
            self.current_warnings.push(message);
        }
    }

    pub(crate) fn reset_errors(&mut self) {
        self.current_errors.clear();
    }
}
