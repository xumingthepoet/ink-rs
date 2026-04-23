use crate::{
    ast::{AstNode, ParsedStory},
    compiler::StageOutput,
    diagnostic::Diagnostic,
    source::{SourceFile, SourceInput, SourceLine},
};

pub(crate) fn parse(input: SourceInput) -> StageOutput<ParsedStory> {
    let source = SourceFile::from_input(input);
    let mut parser = Parser::new(source);
    let story = parser.parse_story();

    StageOutput {
        artifact: Some(story),
        diagnostics: parser.diagnostics,
    }
}

struct Parser {
    source: SourceFile,
    diagnostics: Vec<Diagnostic>,
}

impl Parser {
    fn new(source: SourceFile) -> Self {
        Self {
            source,
            diagnostics: Vec::new(),
        }
    }

    fn parse_story(&mut self) -> ParsedStory {
        let mut nodes = Vec::new();

        for line in self.source.lines.clone() {
            if let Some(node) = self.parse_statement(&line) {
                nodes.push(node);
            }
        }

        ParsedStory { nodes }
    }

    fn parse_statement(&mut self, line: &SourceLine) -> Option<AstNode> {
        if line.text.trim().is_empty() {
            return None;
        }

        if let Some(choice) = self.parse_choice(line) {
            return Some(choice);
        }

        if let Some(divert) = self.parse_divert(line) {
            return Some(divert);
        }

        if let Some(diagnostic) = self.try_unsupported_statement(line) {
            self.diagnostics.push(diagnostic);
            return None;
        }

        Some(AstNode::TextLine {
            text: line.text.trim().to_string(),
            span: line.span.clone(),
        })
    }

    fn try_unsupported_statement(&self, line: &SourceLine) -> Option<Diagnostic> {
        let trimmed = line.text.trim_start();

        let feature = if trimmed.starts_with("INCLUDE ") {
            Some("include")
        } else if trimmed.starts_with("VAR ") {
            Some("global variable declaration")
        } else if trimmed.starts_with("LIST ") {
            Some("list declaration")
        } else if trimmed.starts_with("CONST ") {
            Some("constant declaration")
        } else if trimmed.starts_with("EXTERNAL ") {
            Some("external declaration")
        } else if trimmed.starts_with("===") {
            Some("knot declaration")
        } else if trimmed.starts_with('=') {
            Some("stitch declaration")
        } else if trimmed.starts_with('+') {
            Some("choice")
        } else if trimmed.starts_with('-') && !trimmed.starts_with("->") {
            Some("gather")
        } else if trimmed.starts_with('~') {
            Some("logic line")
        } else if trimmed.contains('{') || trimmed.contains('}') {
            Some("inline logic")
        } else if trimmed.contains("<>") {
            Some("glue")
        } else if trimmed.contains('#') {
            Some("tag")
        } else {
            None
        };

        feature.map(|feature| Diagnostic::unsupported(line.span.clone(), feature))
    }

    fn parse_choice(&self, line: &SourceLine) -> Option<AstNode> {
        let trimmed = line.text.trim_start();
        let choice_text = trimmed.strip_prefix('*')?.trim_start();
        let inline = choice_text.contains('[') || choice_text.contains(']');
        let text = choice_text
            .trim_matches(|ch| ch == '[' || ch == ']')
            .trim()
            .to_string();

        Some(AstNode::Choice {
            text,
            inline,
            span: line.span.clone(),
        })
    }

    fn parse_divert(&self, line: &SourceLine) -> Option<AstNode> {
        let trimmed = line.text.trim_start();
        let target = trimmed.strip_prefix("->")?.trim().to_string();

        Some(AstNode::Divert {
            target,
            span: line.span.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_text_lines() {
        let output = parse(SourceInput::new("Line.\nOther line."));
        assert!(output.diagnostics.is_empty());
        let story = output.artifact.unwrap();
        assert_eq!(story.nodes.len(), 2);
    }

    #[test]
    fn parses_choice() {
        let output = parse(SourceInput::new("* Choice"));
        assert!(output.diagnostics.is_empty());
        let story = output.artifact.unwrap();
        assert_eq!(story.nodes.len(), 1);
    }

    #[test]
    fn reports_unsupported_sticky_choice() {
        let output = parse(SourceInput::new("+ Choice"));
        assert_eq!(output.diagnostics.len(), 1);
        assert!(output.artifact.unwrap().nodes.is_empty());
    }
}
