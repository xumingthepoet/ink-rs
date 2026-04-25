use crate::{
    parsed::{ContentList, Object, Sequence, Text},
    source::SourceLine,
};

use super::{parser::Parser, text};

impl Parser {
    pub(super) fn parse_multiline_sequence(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<Vec<Object>> {
        let line = &lines[*index];
        let trimmed = line.text.trim();
        let after_open = trimmed.strip_prefix('{')?.trim();
        let (sequence_type, rest) = text::parse_sequence_type_annotation(after_open)?;
        if !rest.trim().is_empty() {
            return None;
        }

        *index += 1;
        let mut elements = Vec::new();

        while *index < lines.len() {
            let current_line = &lines[*index];
            let current_trimmed = current_line.text.trim();

            if current_trimmed == "}" {
                *index += 1;
                return Some(vec![
                    Object::ContentList(ContentList::new(vec![Object::Sequence(Sequence::new(
                        sequence_type,
                        elements,
                    ))])),
                    Object::Text(Text::new("\n", current_line.span.clone())),
                ]);
            }

            if current_trimmed.is_empty() {
                *index += 1;
                continue;
            }

            elements.push(self.parse_multiline_sequence_element(lines, index)?);
        }

        None
    }

    fn parse_multiline_sequence_element(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<ContentList> {
        let line = &lines[*index];
        let trimmed = line.text.trim_start();
        if trimmed.starts_with("->") {
            return None;
        }

        let remainder = trimmed.strip_prefix('-')?.trim_start();
        let mut objects = Vec::new();
        if !remainder.is_empty() {
            objects.push(Object::Text(Text::new("\n", line.span.clone())));
            let content_line = SourceLine {
                text: remainder.to_string(),
                span: line.span.clone(),
            };
            objects.extend(self.parse_statement(&content_line));
        }

        *index += 1;
        while *index < lines.len() {
            let next_line = &lines[*index];
            let next_trimmed = next_line.text.trim();
            if next_trimmed == "}" || is_multiline_sequence_element_start(next_line) {
                break;
            }
            if !next_trimmed.is_empty() {
                if objects.is_empty() {
                    objects.push(Object::Text(Text::new("\n", next_line.span.clone())));
                }
                objects.extend(self.parse_statement(next_line));
            }
            *index += 1;
        }

        Some(ContentList::new(objects))
    }
}

fn is_multiline_sequence_element_start(line: &SourceLine) -> bool {
    let trimmed = line.text.trim_start();
    trimmed.starts_with('-') && !trimmed.starts_with("->")
}

#[cfg(test)]
mod tests {
    use crate::{parsed::Object, source::SourceInput, syntax::parse};

    #[test]
    fn parses_multiline_sequence_into_content_list() {
        let output = parse(SourceInput::new("{ cycle:\n- one\n- two\n}"));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("expected story");
        let has_sequence = story.root_weave().content().iter().any(|object| {
            matches!(
                object,
                Object::ContentList(content)
                    if matches!(content.objects().first(), Some(Object::Sequence(_)))
            )
        });
        assert!(has_sequence);
    }
}
