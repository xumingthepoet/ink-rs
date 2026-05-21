use crate::{runtime::TextItem, styled_text::tagged_style_markup};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TranscriptBuffer {
    paragraphs: Vec<String>,
    pending_paragraph_break: bool,
}

impl TranscriptBuffer {
    pub fn from_text(text: String) -> Self {
        let paragraphs = if text.is_empty() {
            Vec::new()
        } else {
            text.split("\n\n").map(str::to_string).collect()
        };

        Self {
            paragraphs,
            pending_paragraph_break: false,
        }
    }

    pub fn append(&mut self, items: &[TextItem]) {
        for item in items {
            if item.is_break() {
                self.pending_paragraph_break = true;
            } else {
                self.append_message(item.text(), item.tags());
            }
        }
    }

    pub fn text(&self) -> String {
        self.paragraphs.join("\n\n")
    }

    fn append_message(&mut self, text: &str, tags: &[String]) {
        let text = tagged_style_markup(text, tags, true);
        if self.pending_paragraph_break || self.paragraphs.is_empty() {
            self.paragraphs.push(text);
        } else if let Some(last_paragraph) = self.paragraphs.last_mut() {
            if !last_paragraph.is_empty() && !text.is_empty() {
                last_paragraph.push(' ');
            }
            last_paragraph.push_str(&text);
        } else {
            self.paragraphs.push(text);
        }

        self.pending_paragraph_break = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transcript_groups_text_until_break() {
        let mut transcript = TranscriptBuffer::default();
        transcript.append(&[
            TextItem::Plain("First".to_string()),
            TextItem::Plain("line".to_string()),
            TextItem::Break,
            TextItem::Plain("Second".to_string()),
        ]);

        assert_eq!(transcript.text(), "First line\n\nSecond");
    }

    #[test]
    fn transcript_preserves_tag_style_markup() {
        let mut transcript = TranscriptBuffer::default();
        transcript.append(&[TextItem::Tagged {
            text: "Blue".to_string(),
            tags: vec!["color:blue".to_string()],
        }]);

        assert_eq!(transcript.text(), "[style color=blue]Blue[/style]");
    }
}
