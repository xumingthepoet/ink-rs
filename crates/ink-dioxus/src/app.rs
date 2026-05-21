use crate::{
    runtime::{InkError, InkRuntime, RuntimePause, TextItem},
    tags,
};

const DEFAULT_STORY_TITLE: &str = "Ink Story";
const DEFAULT_PROMPT_TITLE: &str = "Choices";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InkAppOptions {
    pub default_story_title: String,
    pub default_prompt_title: String,
    pub default_prompt_title_function: Option<String>,
}

impl Default for InkAppOptions {
    fn default() -> Self {
        Self {
            default_story_title: DEFAULT_STORY_TITLE.to_string(),
            default_prompt_title: DEFAULT_PROMPT_TITLE.to_string(),
            default_prompt_title_function: None,
        }
    }
}

impl InkAppOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_default_story_title(mut self, title: impl Into<String>) -> Self {
        self.default_story_title = title.into();
        self
    }

    pub fn with_default_prompt_title(mut self, title: impl Into<String>) -> Self {
        self.default_prompt_title = title.into();
        self
    }

    pub fn with_default_prompt_title_function(mut self, function_path: impl Into<String>) -> Self {
        self.default_prompt_title_function = Some(function_path.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InkChoiceOption {
    pub label: String,
    pub tags: Vec<String>,
    pub enabled: bool,
}

impl InkChoiceOption {
    fn new(label: String, tags: Vec<String>) -> Self {
        let enabled = parse_choice_enabled(&tags);
        Self {
            label,
            tags,
            enabled,
        }
    }

    pub fn is_numbered_choice(&self) -> bool {
        self.tags
            .iter()
            .any(|tag| tag == "menu" || tag == "numbered")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InkChoicePrompt {
    pub revision: u64,
    pub title: String,
    pub options: Vec<InkChoiceOption>,
}

impl InkChoicePrompt {
    pub fn enabled_option_index(&self, option_index: usize) -> Option<usize> {
        self.options
            .get(option_index)
            .filter(|option| option.enabled)
            .map(|_| option_index)
    }

    pub fn first_enabled_index(&self) -> Option<usize> {
        self.options.iter().position(|option| option.enabled)
    }

    pub fn step_enabled_index(&self, selected_index: usize, step: isize) -> Option<usize> {
        if self.options.is_empty() {
            return None;
        }

        if step == 0 {
            return self.enabled_option_index(selected_index);
        }

        let item_count = self.options.len() as isize;
        let start_index = if selected_index < self.options.len() {
            selected_index
        } else {
            self.first_enabled_index()?
        } as isize;

        for distance in 1..=self.options.len() {
            let index = (start_index + step * distance as isize).rem_euclid(item_count) as usize;
            if self.options[index].enabled {
                return Some(index);
            }
        }

        None
    }

    pub fn sync_selected_index(&self, selected_index: usize, reset_selection: bool) -> usize {
        if !reset_selection && self.enabled_option_index(selected_index).is_some() {
            return selected_index;
        }

        self.first_enabled_index().unwrap_or(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InkToast {
    pub text: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InkAppInteraction {
    Choice(InkChoicePrompt),
    Ended,
}

#[derive(Debug, Clone)]
enum InkAppMode {
    Running,
    Choice {
        revision: u64,
        title: String,
        options: Vec<InkChoiceOption>,
    },
    Ended,
}

pub struct InkApp {
    runtime: InkRuntime,
    mode: InkAppMode,
    output_buffer: Vec<TextItem>,
    toast_buffer: Vec<InkToast>,
    interaction_revision: u64,
    default_story_title: String,
    story_title: String,
    default_prompt_title: String,
    prompt_title: Option<String>,
}

impl InkApp {
    pub fn new(runtime: InkRuntime) -> Result<Self, InkError> {
        Self::with_options(runtime, InkAppOptions::default())
    }

    pub fn with_options(mut runtime: InkRuntime, options: InkAppOptions) -> Result<Self, InkError> {
        let default_prompt_title = resolve_default_prompt_title(&mut runtime, &options)?;
        let mut app = Self {
            runtime,
            mode: InkAppMode::Running,
            output_buffer: Vec::new(),
            toast_buffer: Vec::new(),
            interaction_revision: 0,
            default_story_title: options.default_story_title.clone(),
            story_title: options.default_story_title,
            default_prompt_title,
            prompt_title: None,
        };

        app.drive_story()?;
        Ok(app)
    }

    pub fn from_saved_state(runtime: InkRuntime, saved_state: &str) -> Result<Self, InkError> {
        Self::from_saved_state_with_options(runtime, saved_state, InkAppOptions::default())
    }

    pub fn from_saved_state_with_options(
        mut runtime: InkRuntime,
        saved_state: &str,
        options: InkAppOptions,
    ) -> Result<Self, InkError> {
        runtime.load_state(saved_state)?;
        Self::with_options(runtime, options)
    }

    fn drive_story(&mut self) -> Result<(), InkError> {
        let run = self.runtime.run_until_pause()?;
        self.consume_runtime_output(run.text);

        match run.pause {
            RuntimePause::Choice(choices) => {
                self.interaction_revision = self.interaction_revision.wrapping_add(1);
                self.mode = InkAppMode::Choice {
                    revision: self.interaction_revision,
                    title: resolved_prompt_title(&self.prompt_title, &self.default_prompt_title),
                    options: choices
                        .into_iter()
                        .map(|choice| InkChoiceOption::new(choice.label, choice.tags))
                        .collect(),
                };
            }
            RuntimePause::Ended => {
                self.mode = InkAppMode::Ended;
            }
        }

        Ok(())
    }

    pub fn submit_choice(&mut self, option_index: usize) -> Result<(), InkError> {
        if let InkAppMode::Choice { options, .. } = &self.mode {
            if options
                .get(option_index)
                .is_none_or(|option| !option.enabled)
            {
                return Ok(());
            }

            self.mode = InkAppMode::Running;
            self.prompt_title = None;
            self.runtime.select_choice(option_index)?;
            self.drive_story()?;
        }
        Ok(())
    }

    pub fn current_interaction(&self) -> InkAppInteraction {
        match &self.mode {
            InkAppMode::Choice {
                revision,
                title,
                options,
            } => InkAppInteraction::Choice(InkChoicePrompt {
                revision: *revision,
                title: title.clone(),
                options: options.clone(),
            }),
            InkAppMode::Ended => InkAppInteraction::Ended,
            InkAppMode::Running => InkAppInteraction::Ended,
        }
    }

    pub fn drain_output(&mut self) -> Vec<TextItem> {
        std::mem::take(&mut self.output_buffer)
    }

    pub fn drain_toasts(&mut self) -> Vec<InkToast> {
        std::mem::take(&mut self.toast_buffer)
    }

    pub fn story_title(&self) -> &str {
        &self.story_title
    }

    pub fn default_prompt_title(&self) -> &str {
        &self.default_prompt_title
    }

    pub fn dynamic_prompt_title(&self) -> Option<&str> {
        self.prompt_title.as_deref()
    }

    pub fn restore_ui_titles(&mut self, story_title: String, prompt_title: Option<String>) {
        self.story_title =
            normalize_ui_title(story_title).unwrap_or_else(|| self.default_story_title.clone());
        self.prompt_title = prompt_title.and_then(normalize_ui_title);

        let title = resolved_prompt_title(&self.prompt_title, &self.default_prompt_title);
        if let InkAppMode::Choice {
            title: mode_title, ..
        } = &mut self.mode
        {
            *mode_title = title;
        }
    }

    pub fn save_state(&self) -> Result<String, InkError> {
        self.runtime.save_state()
    }

    fn consume_runtime_output(&mut self, output: Vec<TextItem>) {
        for item in output {
            if apply_output_control_tags(
                &mut self.story_title,
                &mut self.prompt_title,
                &mut self.toast_buffer,
                &item,
            ) {
                continue;
            }

            self.output_buffer.push(item);
        }
    }
}

fn resolve_default_prompt_title(
    runtime: &mut InkRuntime,
    options: &InkAppOptions,
) -> Result<String, InkError> {
    match options.default_prompt_title_function.as_deref() {
        Some(function_path) => runtime.call_string_function(function_path),
        None => Ok(options.default_prompt_title.clone()),
    }
}

fn parse_choice_enabled(tags: &[String]) -> bool {
    tags::tag_bool(tags, "enabled").unwrap_or(true)
}

fn apply_output_control_tags(
    story_title: &mut String,
    prompt_title: &mut Option<String>,
    toast_buffer: &mut Vec<InkToast>,
    item: &TextItem,
) -> bool {
    let tags = item.tags();
    let mut consumed = false;

    if let Some(title) = control_tag_value(tags, "title", item.text()) {
        if let Some(title) = normalize_ui_title(title) {
            *story_title = title;
        }
        consumed = true;
    }

    if let Some(prompt) = control_tag_value(tags, "prompt", item.text()) {
        *prompt_title = normalize_ui_title(prompt);
        consumed = true;
    }

    if let Some(toast) = control_tag_value(tags, "toast", item.text()) {
        if let Some(text) = normalize_control_text(toast) {
            toast_buffer.push(InkToast {
                text,
                tags: tags.to_vec(),
            });
        }
        consumed = true;
    }

    consumed
}

fn control_tag_value(tags: &[String], key: &str, text: &str) -> Option<String> {
    tags::tag_value(tags, key).or_else(|| tags::tag_marker(tags, key).then(|| text.to_string()))
}

fn normalize_ui_title(title: String) -> Option<String> {
    normalize_control_text(title)
}

fn normalize_control_text(text: String) -> Option<String> {
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

fn resolved_prompt_title(prompt_title: &Option<String>, default_prompt_title: &str) -> String {
    prompt_title
        .clone()
        .unwrap_or_else(|| default_prompt_title.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::InkRuntime;

    fn option(label: &str, enabled: bool) -> InkChoiceOption {
        InkChoiceOption {
            label: label.to_string(),
            tags: Vec::new(),
            enabled,
        }
    }

    fn runtime(source: &str) -> InkRuntime {
        InkRuntime::load_from_source_texts(&[("test.ink", source)])
            .expect("test ink should compile")
            .expect("test ink should create a runtime")
    }

    #[test]
    fn enabled_tag_disables_choice() {
        assert!(!parse_choice_enabled(&["enabled:false".to_string()]));
        assert!(!parse_choice_enabled(&["# enabled:{0}".to_string()]));
        assert!(parse_choice_enabled(&["enabled:true".to_string()]));
        assert!(parse_choice_enabled(&["menu".to_string()]));
    }

    #[test]
    fn prompt_selection_skips_disabled_choices() {
        let prompt = InkChoicePrompt {
            revision: 1,
            title: "Choose".to_string(),
            options: vec![
                option("locked", false),
                option("open", true),
                option("later", false),
            ],
        };

        assert_eq!(prompt.sync_selected_index(0, true), 1);
        assert_eq!(prompt.step_enabled_index(1, 1), Some(1));
        assert_eq!(prompt.enabled_option_index(0), None);
        assert_eq!(prompt.enabled_option_index(1), Some(1));
    }

    #[test]
    fn output_control_tags_update_titles_and_are_consumed() {
        let mut story_title = DEFAULT_STORY_TITLE.to_string();
        let mut prompt_title = None;

        assert!(apply_output_control_tags(
            &mut story_title,
            &mut prompt_title,
            &mut Vec::new(),
            &TextItem::Tagged {
                text: "Location: Border Village - Square".to_string(),
                tags: vec!["title".to_string()],
            },
        ));
        assert_eq!(story_title, "Location: Border Village - Square");

        assert!(apply_output_control_tags(
            &mut story_title,
            &mut prompt_title,
            &mut Vec::new(),
            &TextItem::Tagged {
                text: "Next action".to_string(),
                tags: vec!["prompt".to_string()],
            },
        ));
        assert_eq!(prompt_title.as_deref(), Some("Next action"));
    }

    #[test]
    fn output_control_tags_accept_key_values() {
        let mut story_title = DEFAULT_STORY_TITLE.to_string();
        let mut prompt_title = None;

        assert!(apply_output_control_tags(
            &mut story_title,
            &mut prompt_title,
            &mut Vec::new(),
            &TextItem::Tagged {
                text: "ignored".to_string(),
                tags: vec![
                    "title:\"Location: Border Village - Square\" prompt:Choose action".to_string()
                ],
            },
        ));

        assert_eq!(story_title, "Location: Border Village - Square");
        assert_eq!(prompt_title.as_deref(), Some("Choose action"));
    }

    #[test]
    fn toast_output_tag_is_consumed_as_transient_ui_event() {
        let mut story_title = DEFAULT_STORY_TITLE.to_string();
        let mut prompt_title = None;
        let mut toast_buffer = Vec::new();

        assert!(apply_output_control_tags(
            &mut story_title,
            &mut prompt_title,
            &mut toast_buffer,
            &TextItem::Tagged {
                text: "Gained herb x1".to_string(),
                tags: vec!["toast color:green".to_string()],
            },
        ));

        assert_eq!(
            toast_buffer,
            vec![InkToast {
                text: "Gained herb x1".to_string(),
                tags: vec!["toast color:green".to_string()],
            }]
        );
    }

    #[test]
    fn toast_choice_tag_does_not_change_choice_behavior() {
        let choice = InkChoiceOption::new("Gain herb".to_string(), vec!["toast".to_string()]);

        assert!(choice.enabled);
        assert!(!choice.is_numbered_choice());
    }

    #[test]
    fn app_uses_fallback_prompt_title_without_internal_function() {
        let app = InkApp::new(runtime(
            r#"=== module game ===

== main ==
Line.
* Continue
    -> END
"#,
        ))
        .expect("app should start");

        assert_eq!(app.default_prompt_title(), DEFAULT_PROMPT_TITLE);
    }

    #[test]
    fn app_can_read_default_prompt_title_from_internal_function() {
        let app = InkApp::with_options(
            runtime(
                r#"=== module game ===

== main ==
* Continue
    -> END

=== module ui_text ===

CONST default_prompt_title_text: string = "Next choice"

== INTERNAL default_prompt_title() => string ==
~ return default_prompt_title_text
"#,
            ),
            InkAppOptions::new()
                .with_default_prompt_title_function("ui_text::default_prompt_title"),
        )
        .expect("app should start");

        assert_eq!(app.default_prompt_title(), "Next choice");
    }
}
