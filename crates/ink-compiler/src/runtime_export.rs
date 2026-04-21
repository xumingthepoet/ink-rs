use serde_json::{json, Value};

use crate::{
    error::CompilerError,
    parsed::{ObjectKind, ObjectRef, Story as ParsedStory},
};

pub fn export_story_json(story: &ParsedStory) -> Result<String, CompilerError> {
    let mut main_content = Vec::new();
    for object in story.content() {
        export_object(&object, &mut main_content)?;
    }

    let mut inner_container = main_content;
    inner_container.push(Value::Null);

    let root = json!([Value::Array(inner_container), "done", null]);
    let story_json = json!({
        "inkVersion": 21,
        "root": root,
        "listDefs": {},
    });

    Ok(story_json.to_string())
}

fn export_object(object: &ObjectRef, output: &mut Vec<Value>) -> Result<(), CompilerError> {
    let borrowed = object.borrow();

    match borrowed.kind() {
        ObjectKind::ContentList { .. } => {
            for child in borrowed.content() {
                export_object(&child, output)?;
            }
        }
        ObjectKind::Text { text } => output.push(export_text_token(text)),
        ObjectKind::AuthorWarning { .. }
        | ObjectKind::Tag { .. }
        | ObjectKind::Divert { .. }
        | ObjectKind::Weave { .. }
        | ObjectKind::Choice { .. }
        | ObjectKind::Gather { .. }
        | ObjectKind::Flow { .. }
        | ObjectKind::Generic => {
            return Err(CompilerError::Unsupported(
                "runtime export currently supports plain text stories only",
            ));
        }
    }

    Ok(())
}

fn export_text_token(text: &str) -> Value {
    if text == "\n" {
        json!("\n")
    } else {
        json!(format!("^{}", text))
    }
}

#[cfg(test)]
mod tests {
    use crate::parsed::{ContentList, Divert, Story, Text};

    use super::export_story_json;

    #[test]
    fn exports_minimal_plain_text_story_json() {
        let line = ContentList::new();
        line.add_content(Text::new("Hello").object());
        line.add_content(Text::new("\n").object());
        let story = Story::new(vec![line.object()], false);

        let json = export_story_json(&story).expect("expected plain text story export");
        assert!(json.contains("\"inkVersion\":21"));
        assert!(json.contains("\"root\""));
        assert!(json.contains("\"listDefs\":{}"));
        assert!(json.contains("^Hello"));
        assert!(json.contains("\"done\""));
    }

    #[test]
    fn rejects_non_text_nodes() {
        let story = Story::new(vec![Divert::empty().object()], false);

        assert!(export_story_json(&story).is_err());
    }
}
