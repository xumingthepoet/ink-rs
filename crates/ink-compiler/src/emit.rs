use ink_story_json_format::Program as RuntimeProgram;

use crate::{compiler::StageOutput, diagnostic::Diagnostic, source::SourceSpan};

pub(crate) fn emit_json(program: RuntimeProgram) -> StageOutput<String> {
    match program
        .to_json_string()
        .map_err(|error| format!("failed to serialize compiled story JSON: {error}"))
    {
        Ok(json) => StageOutput {
            artifact: Some(json),
            diagnostics: Vec::new(),
        },
        Err(error) => StageOutput {
            artifact: None,
            diagnostics: vec![Diagnostic::error(SourceSpan::new(None, 1, 1), error)],
        },
    }
}

#[cfg(test)]
mod tests {
    use ink_story_json_format as format;
    use serde_json::json;

    use super::*;

    #[test]
    fn emits_text_string_tokens() {
        let program =
            RuntimeProgram::new(format::Container::unnamed(vec![format::Object::String(
                "Line.".to_string(),
            )]));

        let json = emit_json(program).artifact.expect("expected emitted json");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(
            value,
            json!({
                "inkVersion": 1,
                "root": ["^Line.", null],
                "listDefs": {}
            })
        );
    }

    #[test]
    fn emits_void_token() {
        let program = RuntimeProgram::new(format::Container::unnamed(vec![format::Object::Void]));

        let json = emit_json(program).artifact.expect("expected emitted json");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["root"][0], json!("void"));
    }
}
