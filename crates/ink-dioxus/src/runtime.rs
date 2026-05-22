use ink_compiler::{format_diagnostics, Compiler, CompilerOptions, DiagnosticsPolicy, SourceInput};
use ink_runtime::{story::Story, value_type::ValueType};
use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InkSource {
    pub filename: &'static str,
    pub source: &'static str,
}

impl InkSource {
    pub const fn new(filename: &'static str, source: &'static str) -> Self {
        Self { filename, source }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InkGameSource {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub sources: &'static [InkSource],
}

impl InkGameSource {
    pub const fn new(
        id: &'static str,
        title: &'static str,
        description: &'static str,
        sources: &'static [InkSource],
    ) -> Self {
        Self {
            id,
            title,
            description,
            sources,
        }
    }
}

fn strip_bom(content: &str) -> &str {
    content.strip_prefix('\u{FEFF}').unwrap_or(content)
}

#[derive(Debug)]
pub enum InkError {
    Compile(String),
    Runtime(String),
}

impl fmt::Display for InkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compile(reason) => write!(f, "ink-rs compile failed: {reason}"),
            Self::Runtime(reason) => write!(f, "ink-rs runtime failed: {reason}"),
        }
    }
}

impl Error for InkError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextItem {
    Plain(String),
    Tagged { text: String, tags: Vec<String> },
    Break,
}

impl TextItem {
    pub fn tagged(text: String, tags: Vec<String>) -> Self {
        if tags.is_empty() {
            Self::Plain(text)
        } else {
            Self::Tagged { text, tags }
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Self::Plain(text) => text,
            Self::Tagged { text, .. } => text,
            Self::Break => "",
        }
    }

    pub fn tags(&self) -> &[String] {
        match self {
            Self::Tagged { tags, .. } => tags,
            Self::Plain(_) | Self::Break => &[],
        }
    }

    pub fn is_break(&self) -> bool {
        matches!(self, Self::Break)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeChoice {
    pub label: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimePause {
    Choice(Vec<RuntimeChoice>),
    Ended,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeStep {
    pub text: Vec<TextItem>,
    pub pause: RuntimePause,
}

pub struct InkRuntime {
    story: Story,
    output_buffer: Vec<TextItem>,
}

impl InkRuntime {
    pub fn load_from_ink_sources(sources: &[InkSource]) -> Result<Option<Self>, InkError> {
        if sources.is_empty() {
            return Ok(None);
        }

        let inputs = sources
            .iter()
            .map(|source| {
                SourceInput::named(
                    strip_bom(source.source).to_string(),
                    source.filename.to_string(),
                )
            })
            .collect();

        Self::compile_source_inputs(inputs)
    }

    pub fn load_from_source_texts(sources: &[(&str, &str)]) -> Result<Option<Self>, InkError> {
        if sources.is_empty() {
            return Ok(None);
        }

        let inputs = sources
            .iter()
            .map(|(filename, raw)| {
                SourceInput::named(strip_bom(raw).to_string(), (*filename).to_string())
            })
            .collect();

        Self::compile_source_inputs(inputs)
    }

    pub fn compile_source_inputs(inputs: Vec<SourceInput>) -> Result<Option<Self>, InkError> {
        if inputs.is_empty() {
            return Ok(None);
        }

        let compiler = Compiler::with_options(CompilerOptions {
            source_filename: None,
            diagnostics_policy: DiagnosticsPolicy::DenyWarnings,
        });
        let result = compiler.compile_sources(inputs);
        if result.failed() {
            return Err(InkError::Compile(format_diagnostics(&result.diagnostics)));
        }
        let Some(compiled) = result.artifact else {
            return Err(InkError::Compile(format_diagnostics(&result.diagnostics)));
        };

        let story = Story::new(&compiled.json).map_err(|error| {
            InkError::Compile(format!("compiled story JSON parse error: {error}"))
        })?;

        Ok(Some(Self {
            story,
            output_buffer: Vec::new(),
        }))
    }

    pub fn run_until_pause(&mut self) -> Result<RuntimeStep, InkError> {
        while self.story.can_continue() {
            let line = self
                .story
                .cont()
                .map_err(|error| InkError::Runtime(format!("continue failed: {error}")))?;
            let line_tags = self.story.get_current_tags().unwrap_or_default();

            for part in line.split('\n') {
                if part.is_empty() {
                    self.output_buffer.push(TextItem::Break);
                } else {
                    self.output_buffer
                        .push(TextItem::tagged(part.to_string(), line_tags.clone()));
                }
            }
        }

        let choices = self.story.get_current_choices();
        if !choices.is_empty() {
            let runtime_choices = choices
                .into_iter()
                .map(|choice| RuntimeChoice {
                    label: choice.text.to_string(),
                    tags: choice.tags.clone(),
                })
                .collect();

            return Ok(RuntimeStep {
                text: std::mem::take(&mut self.output_buffer),
                pause: RuntimePause::Choice(runtime_choices),
            });
        }

        Ok(RuntimeStep {
            text: std::mem::take(&mut self.output_buffer),
            pause: RuntimePause::Ended,
        })
    }

    pub fn select_choice(&mut self, choice_index: usize) -> Result<(), InkError> {
        self.story
            .choose_choice_index(choice_index)
            .map_err(|error| InkError::Runtime(format!("choice selection failed: {error}")))?;
        Ok(())
    }

    pub fn call_string_function(&mut self, function_path: &str) -> Result<String, InkError> {
        match self
            .story
            .call_internal(function_path, None)
            .map_err(|error| {
                InkError::Runtime(format!("internal function {function_path} failed: {error}"))
            })? {
            Some(ValueType::String(value)) => Ok(value.string),
            Some(value) => Err(InkError::Runtime(format!(
                "internal function {function_path} returned {}, expected string",
                value_type_name(&value)
            ))),
            None => Err(InkError::Runtime(format!(
                "internal function {function_path} returned void, expected string"
            ))),
        }
    }

    pub fn save_state(&self) -> Result<String, InkError> {
        self.story
            .save_state()
            .map_err(|error| InkError::Runtime(format!("save state failed: {error}")))
    }

    pub fn load_state(&mut self, state: &str) -> Result<(), InkError> {
        self.story
            .load_state(state)
            .map_err(|error| InkError::Runtime(format!("load state failed: {error}")))?;
        self.output_buffer.clear();
        Ok(())
    }
}

fn value_type_name(value: &ValueType) -> &'static str {
    match value {
        ValueType::Bool(_) => "bool",
        ValueType::Int(_) => "int",
        ValueType::Float(_) => "float",
        ValueType::String(_) => "string",
        ValueType::DivertTarget(_) => "divert target",
        ValueType::VariablePointer(_) => "variable pointer",
        ValueType::Array(_) => "array",
        ValueType::Dict(_) => "dict",
        ValueType::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_test_runtime(source: &str) -> InkRuntime {
        InkRuntime::load_from_source_texts(&[("test.ink", source)])
            .expect("test ink should compile")
            .expect("test ink should create a runtime")
    }

    fn plain_text(step: RuntimeStep) -> Vec<String> {
        step.text
            .into_iter()
            .filter_map(|item| match item {
                TextItem::Plain(text) | TextItem::Tagged { text, .. } => Some(text),
                TextItem::Break => None,
            })
            .collect()
    }

    fn rendered_text_items(step: RuntimeStep) -> String {
        step.text
            .into_iter()
            .map(|item| match item {
                TextItem::Plain(text) | TextItem::Tagged { text, .. } => text,
                TextItem::Break => "\n".to_string(),
            })
            .collect()
    }

    #[test]
    fn story_state_roundtrips_from_choice_prompt() {
        let mut runtime = compile_test_runtime(
            r#"=== module game ===

== main ==
Opening
* Go left
    Left ending

* Go right
    Right ending

"#,
        );

        let first_pause = runtime.run_until_pause().expect("first pause");
        match first_pause.pause {
            RuntimePause::Choice(choices) => {
                assert_eq!(choices.len(), 2);
                assert_eq!(choices[0].label, "Go left");
                assert_eq!(choices[1].label, "Go right");
            }
            RuntimePause::Ended => panic!("expected choices before ending"),
        }

        let saved = runtime.save_state().expect("state should save");
        runtime.select_choice(0).expect("left choice should select");
        let left = plain_text(runtime.run_until_pause().expect("left branch"));
        assert!(left.iter().any(|line| line == "Left ending"));

        runtime.load_state(&saved).expect("state should load");
        let restored_pause = runtime.run_until_pause().expect("restored pause");
        match restored_pause.pause {
            RuntimePause::Choice(choices) => {
                assert_eq!(choices.len(), 2);
                assert_eq!(choices[1].label, "Go right");
            }
            RuntimePause::Ended => panic!("expected restored choices before ending"),
        }

        runtime
            .select_choice(1)
            .expect("right choice should select");
        let right = plain_text(runtime.run_until_pause().expect("right branch"));
        assert!(right.iter().any(|line| line == "Right ending"));
    }

    #[test]
    fn story_output_preserves_line_breaks_between_continues() {
        let mut runtime = compile_test_runtime(
            r#"=== module game ===

== main ==
First line.

Second line.
* Continue

"#,
        );

        let first_pause = runtime.run_until_pause().expect("first pause");
        assert_eq!(
            rendered_text_items(first_pause),
            "First line.\nSecond line.\n"
        );
    }

    #[test]
    fn choice_tags_preserve_enabled_expression_value() {
        let mut runtime = compile_test_runtime(
            r#"=== module game ===

VAR can_select: bool = false

== main ==
* Always available. # enabled:true

* Visible locked choice. # enabled:{can_select}

"#,
        );

        let first_pause = runtime.run_until_pause().expect("first pause");
        match first_pause.pause {
            RuntimePause::Choice(choices) => {
                assert_eq!(choices[0].label, "Always available.");
                assert_eq!(choices[1].label, "Visible locked choice.");
                assert_eq!(choices[0].tags, vec!["enabled:true"]);
                assert_eq!(choices[1].tags, vec!["enabled:false"]);
            }
            RuntimePause::Ended => panic!("expected choices before ending"),
        }
    }

    #[test]
    fn string_internal_function_can_drive_ui_text() {
        let mut runtime = compile_test_runtime(
            r#"=== module game ===

== main ==
* Continue


=== module ui_text ===

CONST default_prompt_title_text: string = "Next choice"

== INTERNAL default_prompt_title() => string ==
~ return default_prompt_title_text
"#,
        );

        assert_eq!(
            runtime
                .call_string_function("ui_text::default_prompt_title")
                .expect("default prompt internal function should return string"),
            "Next choice"
        );
    }

    #[test]
    fn text_tags_are_preserved_on_output_items() {
        let mut runtime = compile_test_runtime(
            r#"=== module game ===

== main ==
Blue line. # color:blue
* Continue.

"#,
        );

        let first_pause = runtime.run_until_pause().expect("first pause");
        assert!(first_pause.text.iter().any(|item| {
            item.text() == "Blue line." && item.tags().iter().map(String::as_str).eq(["color:blue"])
        }));
    }
}
