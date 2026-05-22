use crate::{
    app::{InkApp, InkAppInteraction, InkAppOptions, InkChoicePrompt, InkToast},
    runtime::{InkRuntime, InkSource},
    styled_text::{
        parse_style_markup, parse_style_markup_with_tags, style_from_tags, StyledSegment, TextStyle,
    },
    transcript::TranscriptBuffer,
};
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, sync::OnceLock};

const DEFAULT_WEB_CSS: &str = include_str!("web.css");
const DEFAULT_TEXT_REVEAL_CHARS_PER_SECOND: u32 = 48;
const DEFAULT_TEXT_REVEAL_TICK_MS: u32 = 50;
const DEFAULT_TOAST_DURATION_TICKS: u32 = 40;
const DEFAULT_WEB_SAVE_STORAGE_KEY: &str = "ink_dioxus.web_save.v1";
const DEFAULT_WEB_SAVE_SCHEMA_VERSION: u32 = 1;

static WEB_CONFIG: OnceLock<WebLaunchConfig> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct WebLaunchConfig {
    pub sources: &'static [InkSource],
    pub app_label: &'static str,
    pub storage_key: &'static str,
    pub save_schema_version: u32,
    pub default_story_title: &'static str,
    pub default_prompt_title: &'static str,
    pub default_prompt_title_function: Option<&'static str>,
    pub text_reveal_chars_per_second: u32,
    pub text_reveal_tick_ms: u32,
    pub toast_duration_ticks: u32,
    pub css: &'static str,
    pub messages: WebMessages,
}

impl WebLaunchConfig {
    pub const fn new(sources: &'static [InkSource]) -> Self {
        Self {
            sources,
            app_label: "ink-rs",
            storage_key: DEFAULT_WEB_SAVE_STORAGE_KEY,
            save_schema_version: DEFAULT_WEB_SAVE_SCHEMA_VERSION,
            default_story_title: "ink-rs Story",
            default_prompt_title: "Choices",
            default_prompt_title_function: None,
            text_reveal_chars_per_second: DEFAULT_TEXT_REVEAL_CHARS_PER_SECOND,
            text_reveal_tick_ms: DEFAULT_TEXT_REVEAL_TICK_MS,
            toast_duration_ticks: DEFAULT_TOAST_DURATION_TICKS,
            css: DEFAULT_WEB_CSS,
            messages: WebMessages::DEFAULT,
        }
    }

    pub fn with_app_label(mut self, value: &'static str) -> Self {
        self.app_label = value;
        self
    }

    pub fn with_storage_key(mut self, value: &'static str) -> Self {
        self.storage_key = value;
        self
    }

    pub fn with_save_schema_version(mut self, value: u32) -> Self {
        self.save_schema_version = value;
        self
    }

    pub fn with_default_story_title(mut self, value: &'static str) -> Self {
        self.default_story_title = value;
        self
    }

    pub fn with_default_prompt_title(mut self, value: &'static str) -> Self {
        self.default_prompt_title = value;
        self
    }

    pub fn with_default_prompt_title_function(mut self, value: &'static str) -> Self {
        self.default_prompt_title_function = Some(value);
        self
    }

    pub fn with_text_reveal(mut self, chars_per_second: u32, tick_ms: u32) -> Self {
        self.text_reveal_chars_per_second = chars_per_second;
        self.text_reveal_tick_ms = tick_ms;
        self
    }

    pub fn with_toast_duration_ticks(mut self, value: u32) -> Self {
        self.toast_duration_ticks = value;
        self
    }

    pub fn with_css(mut self, value: &'static str) -> Self {
        self.css = value;
        self
    }

    pub fn with_messages(mut self, value: WebMessages) -> Self {
        self.messages = value;
        self
    }

    fn app_options(&self) -> InkAppOptions {
        let mut options = InkAppOptions::new()
            .with_default_story_title(self.default_story_title)
            .with_default_prompt_title(self.default_prompt_title);

        if let Some(function_path) = self.default_prompt_title_function {
            options = options.with_default_prompt_title_function(function_path);
        }

        options
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WebMessages {
    pub choices_kicker: &'static str,
    pub error_title: &'static str,
    pub error_hint: &'static str,
    pub no_sources_text: &'static str,
    pub storage_loading_text: &'static str,
    pub initializing_text: &'static str,
    pub no_choices_text: &'static str,
    pub ended_title: &'static str,
    pub ended_text: &'static str,
    pub speed_up_title: &'static str,
}

impl WebMessages {
    pub const DEFAULT: Self = Self {
        choices_kicker: "CHOICES",
        error_title: "Could not start",
        error_hint: "Check ink compiler diagnostics and rebuild.",
        no_sources_text: "No embedded ink-rs source files.",
        storage_loading_text: "Loading local save.",
        initializing_text: "Story is initializing.",
        no_choices_text: "No available choices.",
        ended_title: "Status",
        ended_text: "This story branch has ended.",
        speed_up_title: "Speed up text reveal",
    };
}

pub fn launch(config: WebLaunchConfig) {
    assert!(
        WEB_CONFIG.set(config).is_ok(),
        "ink-dioxus web config was already initialized"
    );
    dioxus::launch(web_app);
}

fn config() -> &'static WebLaunchConfig {
    WEB_CONFIG
        .get()
        .expect("ink-dioxus web config must be initialized before launching")
}

fn web_app() -> Element {
    let mut game = use_signal(WebGameState::new);
    use_future(move || async move {
        let saved_state = read_web_save().await;
        game.write().finish_storage_restore(saved_state);
    });
    use_future(move || async move {
        loop {
            TimeoutFuture::new(config().text_reveal_tick_ms).await;
            let should_tick = {
                let game = game.read();
                game.can_advance_transcript_reveal() || game.has_visible_toast()
            };
            if should_tick {
                let mut game = game.write();
                if game.can_advance_transcript_reveal() {
                    game.advance_transcript_reveal();
                    game.maybe_auto_save();
                }
                game.advance_toast_tick();
            }
        }
    });
    use_effect(move || {
        sync_browser_scroll(game.read().scroll_sync());
    });

    let view = game.read().view();
    let toast = view.toast.clone();
    let css = config().css;
    let app_label = config().app_label;
    let choices_kicker = config().messages.choices_kicker;
    let speed_up_title = config().messages.speed_up_title;

    rsx! {
        style { "{css}" }
        main {
            class: "web-shell",
            section { class: "story-panel",
                header { class: "story-header",
                    div { class: "brand-block",
                        span { class: "kicker", "{app_label}" }
                        h1 { "{view.story_title}" }
                    }
                }
                div { class: "transcript-frame",
                    div { class: "transcript-scroll",
                        for paragraph in view.paragraphs {
                            p { class: "story-line",
                                for segment in paragraph.segments {
                                    span { style: "{segment.style_attr}", "{segment.text}" }
                                }
                            }
                        }
                    }
                }
            }

            aside { class: "choice-panel",
                div { class: "choice-panel-header",
                    if !view.choice_title.is_empty() {
                        span { class: "kicker", "{choices_kicker}" }
                        h2 { "{view.choice_title}" }
                    }
                }

                if view.is_revealing {
                    div {
                        class: "choice-read-surface",
                        title: "{speed_up_title}",
                        onclick: move |_| game.write().accelerate_transcript_reveal(),
                    }
                } else if view.choices.is_empty() {
                    div { class: "empty-choices", "{view.empty_choice_text}" }
                } else {
                    div { class: "choice-list",
                        for choice in view.choices {
                            button {
                                key: "{choice.index}",
                                class: "{choice.class_name}",
                                disabled: !choice.enabled,
                                onpointerdown: move |_| game.write().press_choice(choice.index),
                                onpointerup: move |_| game.write().release_choice(),
                                onpointercancel: move |_| game.write().release_choice(),
                                onpointerleave: move |_| game.write().release_choice(),
                                onblur: move |_| game.write().release_choice(),
                                onclick: move |_| game.write().submit_choice(choice.index),
                                span {
                                    class: "choice-number",
                                    style: "{choice.base_style_attr}",
                                    "{choice.display_number}"
                                }
                                span { class: "choice-label",
                                    for segment in choice.segments {
                                        span { style: "{segment.style_attr}", "{segment.text}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(toast) = toast {
                div { class: "toast-layer",
                    div { class: "toast-card", role: "status", aria_live: "polite",
                        for segment in toast.segments {
                            span { style: "{segment.style_attr}", "{segment.text}" }
                        }
                    }
                }
            }
        }
    }
}

struct WebGameState {
    app: Option<InkApp>,
    transcript: TranscriptBuffer,
    target_transcript_text: String,
    visible_transcript_text: String,
    target_transcript_chars: usize,
    visible_transcript_chars: usize,
    visible_transcript_revision: u64,
    reveal_carry_units: u32,
    last_saved_prompt_revision: u64,
    pressed_choice_index: Option<usize>,
    toast: Option<WebToastState>,
    toast_queue: VecDeque<WebToastState>,
    storage_ready: bool,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WebSaveState {
    schema_version: u32,
    content_fingerprint: String,
    story_state: String,
    transcript_text: String,
    story_title: String,
    prompt_title: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScrollSync {
    transcript_revision: u64,
}

#[derive(Debug)]
struct WebView {
    story_title: String,
    paragraphs: Vec<WebParagraphView>,
    choice_title: String,
    choices: Vec<WebChoiceView>,
    empty_choice_text: String,
    is_revealing: bool,
    toast: Option<WebToastView>,
}

#[derive(Debug, Clone)]
struct WebToastState {
    text: String,
    tags: Vec<String>,
    ticks_remaining: u32,
}

#[derive(Debug, Clone)]
struct WebToastView {
    segments: Vec<WebTextSegment>,
}

#[derive(Debug)]
struct WebChoiceView {
    index: usize,
    display_number: String,
    segments: Vec<WebTextSegment>,
    class_name: String,
    enabled: bool,
    base_style_attr: String,
}

#[derive(Debug)]
struct WebParagraphView {
    segments: Vec<WebTextSegment>,
}

#[derive(Debug, Clone)]
struct WebTextSegment {
    text: String,
    style_attr: String,
}

impl WebGameState {
    fn new() -> Self {
        let mut state = Self {
            app: None,
            transcript: TranscriptBuffer::default(),
            target_transcript_text: String::new(),
            visible_transcript_text: String::new(),
            target_transcript_chars: 0,
            visible_transcript_chars: 0,
            visible_transcript_revision: 0,
            reveal_carry_units: 0,
            last_saved_prompt_revision: 0,
            pressed_choice_index: None,
            toast: None,
            toast_queue: VecDeque::new(),
            storage_ready: false,
            error: None,
        };

        let runtime = match InkRuntime::load_from_ink_sources(config().sources) {
            Ok(Some(runtime)) => runtime,
            Ok(None) => {
                state.error = Some(config().messages.no_sources_text.to_string());
                return state;
            }
            Err(error) => {
                state.error = Some(error.to_string());
                return state;
            }
        };

        match InkApp::with_options(runtime, config().app_options()) {
            Ok(app) => state.app = Some(app),
            Err(error) => {
                state.error = Some(error.to_string());
                return state;
            }
        }

        state.consume_output();
        state
    }

    fn finish_storage_restore(&mut self, saved_state: Option<String>) {
        if self.error.is_none() {
            if let Some(saved_state) = saved_state {
                if self.restore_from_web_save(&saved_state).is_err() {
                    clear_web_save();
                }
            }
        }

        self.storage_ready = true;
        self.maybe_auto_save();
    }

    fn restore_from_web_save(&mut self, saved_state: &str) -> Result<(), String> {
        let save: WebSaveState = serde_json::from_str(saved_state)
            .map_err(|error| format!("save parse failed: {error}"))?;

        if save.schema_version != config().save_schema_version {
            return Err("save version is stale".to_string());
        }

        if save.content_fingerprint != embedded_ink_fingerprint() {
            return Err("save content version does not match current story".to_string());
        }

        let runtime = InkRuntime::load_from_ink_sources(config().sources)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| config().messages.no_sources_text.to_string())?;
        let mut app = InkApp::from_saved_state_with_options(
            runtime,
            &save.story_state,
            config().app_options(),
        )
        .map_err(|error| error.to_string())?;
        app.restore_ui_titles(save.story_title, save.prompt_title);

        self.app = Some(app);
        self.transcript = TranscriptBuffer::from_text(save.transcript_text.clone());
        self.target_transcript_text = save.transcript_text;
        self.target_transcript_chars = self.target_transcript_text.chars().count();
        self.visible_transcript_text = self.target_transcript_text.clone();
        self.visible_transcript_chars = self.target_transcript_chars;
        self.visible_transcript_revision = self.visible_transcript_revision.wrapping_add(1);
        self.reveal_carry_units = 0;
        self.last_saved_prompt_revision = 0;
        self.toast = None;
        self.toast_queue.clear();
        Ok(())
    }

    fn view(&self) -> WebView {
        let messages = config().messages;
        if let Some(error) = &self.error {
            return WebView {
                story_title: messages.error_title.to_string(),
                paragraphs: vec![plain_paragraph(error)],
                choice_title: messages.error_title.to_string(),
                choices: Vec::new(),
                empty_choice_text: messages.error_hint.to_string(),
                is_revealing: false,
                toast: self.toast_view(),
            };
        }

        if !self.storage_ready {
            return WebView {
                story_title: self.story_title().to_string(),
                paragraphs: vec![plain_paragraph(messages.storage_loading_text)],
                choice_title: messages.ended_title.to_string(),
                choices: Vec::new(),
                empty_choice_text: String::new(),
                is_revealing: false,
                toast: self.toast_view(),
            };
        }

        let is_revealing = self.has_pending_transcript_reveal();
        let interaction = self
            .app
            .as_ref()
            .map(InkApp::current_interaction)
            .unwrap_or(InkAppInteraction::Ended);
        let paragraphs = self.transcript_paragraphs();

        if is_revealing {
            return WebView {
                story_title: self.story_title().to_string(),
                paragraphs,
                choice_title: self.default_prompt_title().to_string(),
                choices: Vec::new(),
                empty_choice_text: String::new(),
                is_revealing,
                toast: self.toast_view(),
            };
        }

        match interaction {
            InkAppInteraction::Choice(prompt) => WebView {
                story_title: self.story_title().to_string(),
                paragraphs,
                choice_title: choice_title(&prompt),
                choices: choice_views(&prompt, self.pressed_choice_index),
                empty_choice_text: messages.no_choices_text.to_string(),
                is_revealing,
                toast: self.toast_view(),
            },
            InkAppInteraction::Ended => WebView {
                story_title: self.story_title().to_string(),
                paragraphs,
                choice_title: messages.ended_title.to_string(),
                choices: Vec::new(),
                empty_choice_text: messages.ended_text.to_string(),
                is_revealing,
                toast: self.toast_view(),
            },
        }
    }

    fn submit_choice(&mut self, index: usize) {
        if self.has_pending_transcript_reveal() {
            return;
        }

        self.release_choice();

        let Some(prompt) = self.current_prompt() else {
            return;
        };

        if prompt.enabled_option_index(index).is_none() {
            return;
        }

        let Some(app) = &mut self.app else {
            return;
        };

        match app.submit_choice(index) {
            Ok(()) => {
                self.consume_output();
                self.maybe_auto_save();
            }
            Err(error) => {
                self.error = Some(error.to_string());
            }
        }
    }

    fn press_choice(&mut self, index: usize) {
        if self.has_pending_transcript_reveal() {
            return;
        }

        let Some(prompt) = self.current_prompt() else {
            return;
        };

        if prompt.enabled_option_index(index).is_some() {
            self.pressed_choice_index = Some(index);
        }
    }

    fn release_choice(&mut self) {
        self.pressed_choice_index = None;
    }

    fn consume_output(&mut self) {
        let Some((toasts, output)) = self
            .app
            .as_mut()
            .map(|app| (app.drain_toasts(), app.drain_output()))
        else {
            return;
        };

        self.show_toasts(toasts);
        if !output.is_empty() {
            self.transcript.append(&output);
            self.target_transcript_text = self.transcript.text();
            self.target_transcript_chars = self.target_transcript_text.chars().count();
        }
    }

    fn has_pending_transcript_reveal(&self) -> bool {
        self.visible_transcript_chars < self.target_transcript_chars
    }

    fn can_advance_transcript_reveal(&self) -> bool {
        self.storage_ready && self.has_pending_transcript_reveal()
    }

    fn has_visible_toast(&self) -> bool {
        self.toast.is_some() || !self.toast_queue.is_empty()
    }

    fn advance_toast_tick(&mut self) {
        if let Some(toast) = &mut self.toast {
            toast.ticks_remaining = toast.ticks_remaining.saturating_sub(1);
            if toast.ticks_remaining == 0 {
                self.toast = None;
            }
        }
        self.promote_next_toast();
    }

    fn show_toasts(&mut self, toasts: Vec<InkToast>) {
        for toast in toasts {
            self.toast_queue.push_back(WebToastState {
                text: toast.text,
                tags: toast.tags,
                ticks_remaining: config().toast_duration_ticks,
            });
        }
        self.promote_next_toast();
    }

    fn promote_next_toast(&mut self) {
        if self.toast.is_none() {
            self.toast = self.toast_queue.pop_front();
        }
    }

    fn advance_transcript_reveal(&mut self) {
        if !self.has_pending_transcript_reveal() {
            self.reveal_carry_units = 0;
            return;
        }

        self.reveal_carry_units +=
            config().text_reveal_chars_per_second * config().text_reveal_tick_ms;
        let chars_to_reveal = (self.reveal_carry_units / 1000) as usize;
        self.reveal_carry_units %= 1000;

        if chars_to_reveal == 0 {
            return;
        }

        let hidden_text = &self.target_transcript_text[self.visible_transcript_text.len()..];
        let reveal_chunk = hidden_text
            .chars()
            .take(chars_to_reveal)
            .collect::<String>();
        let revealed_chars = reveal_chunk.chars().count();

        if revealed_chars > 0 {
            self.visible_transcript_text.push_str(&reveal_chunk);
            self.visible_transcript_chars = self
                .visible_transcript_chars
                .saturating_add(revealed_chars)
                .min(self.target_transcript_chars);
            self.visible_transcript_revision = self.visible_transcript_revision.wrapping_add(1);
        }
    }

    fn accelerate_transcript_reveal(&mut self) {
        if !self.has_pending_transcript_reveal() {
            return;
        }

        let current_byte = self.visible_transcript_text.len();
        let next_byte = next_paragraph_boundary(&self.target_transcript_text, current_byte);
        if next_byte <= current_byte {
            return;
        }

        let reveal_chunk = &self.target_transcript_text[current_byte..next_byte];
        let revealed_chars = reveal_chunk.chars().count();
        self.visible_transcript_text.push_str(reveal_chunk);
        self.visible_transcript_chars = self
            .visible_transcript_chars
            .saturating_add(revealed_chars)
            .min(self.target_transcript_chars);
        self.reveal_carry_units = 0;
        self.visible_transcript_revision = self.visible_transcript_revision.wrapping_add(1);
        self.maybe_auto_save();
    }

    fn maybe_auto_save(&mut self) {
        if !self.storage_ready || self.error.is_some() || self.has_pending_transcript_reveal() {
            return;
        }

        let Some(prompt) = self.current_prompt() else {
            return;
        };

        if self.last_saved_prompt_revision == prompt.revision {
            return;
        }

        let Some(app) = &self.app else {
            return;
        };

        let Ok(story_state) = app.save_state() else {
            return;
        };

        let save_state = WebSaveState {
            schema_version: config().save_schema_version,
            content_fingerprint: embedded_ink_fingerprint(),
            story_state,
            transcript_text: self.target_transcript_text.clone(),
            story_title: app.story_title().to_string(),
            prompt_title: app.dynamic_prompt_title().map(str::to_string),
        };

        if let Ok(serialized) = serde_json::to_string(&save_state) {
            write_web_save(&serialized);
            self.last_saved_prompt_revision = prompt.revision;
        }
    }

    fn scroll_sync(&self) -> ScrollSync {
        ScrollSync {
            transcript_revision: self.visible_transcript_revision,
        }
    }

    fn current_prompt(&self) -> Option<InkChoicePrompt> {
        let interaction = self.app.as_ref()?.current_interaction();
        match interaction {
            InkAppInteraction::Choice(prompt) => Some(prompt),
            InkAppInteraction::Ended => None,
        }
    }

    fn story_title(&self) -> &str {
        self.app
            .as_ref()
            .map(InkApp::story_title)
            .unwrap_or(config().default_story_title)
    }

    fn default_prompt_title(&self) -> &str {
        self.app
            .as_ref()
            .map(InkApp::default_prompt_title)
            .unwrap_or(config().default_prompt_title)
    }

    fn transcript_paragraphs(&self) -> Vec<WebParagraphView> {
        let transcript = &self.visible_transcript_text;
        if transcript.trim().is_empty() {
            return vec![plain_paragraph(config().messages.initializing_text)];
        }

        transcript
            .split("\n\n")
            .filter_map(|paragraph| {
                let paragraph = paragraph.trim();
                (!paragraph.is_empty()).then(|| styled_paragraph(paragraph))
            })
            .collect()
    }

    fn toast_view(&self) -> Option<WebToastView> {
        let toast = self.toast.as_ref()?;
        Some(WebToastView {
            segments: parse_style_markup_with_tags(&toast.text, &toast.tags, true)
                .into_iter()
                .map(web_text_segment)
                .collect(),
        })
    }
}

async fn read_web_save() -> Option<String> {
    let storage_key = js_string_literal(config().storage_key);
    let script = format!(
        r#"
try {{
  return window.localStorage.getItem({storage_key});
}} catch (_) {{
  return null;
}}
"#
    );

    document::eval(&script)
        .join::<Option<String>>()
        .await
        .ok()
        .flatten()
}

fn write_web_save(serialized_save: &str) {
    let storage_key = js_string_literal(config().storage_key);
    let serialized_save = js_string_literal(serialized_save);
    let script = format!(
        r#"
try {{
  window.localStorage.setItem({storage_key}, {serialized_save});
}} catch (_) {{}}
"#
    );

    let _ = document::eval(&script);
}

fn clear_web_save() {
    let storage_key = js_string_literal(config().storage_key);
    let script = format!(
        r#"
try {{
  window.localStorage.removeItem({storage_key});
}} catch (_) {{}}
"#
    );

    let _ = document::eval(&script);
}

fn js_string_literal(value: &str) -> String {
    serde_json::to_string(value).expect("string should serialize to a JavaScript literal")
}

fn embedded_ink_fingerprint() -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for source in config().sources {
        update_fnv1a(&mut hash, source.filename.as_bytes());
        update_fnv1a(&mut hash, &[0]);
        update_fnv1a(&mut hash, source.source.as_bytes());
        update_fnv1a(&mut hash, &[0xff]);
    }

    format!("{hash:016x}")
}

fn update_fnv1a(hash: &mut u64, bytes: &[u8]) {
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}

fn choice_title(prompt: &InkChoicePrompt) -> String {
    prompt.title.clone()
}

fn choice_views(
    prompt: &InkChoicePrompt,
    pressed_choice_index: Option<usize>,
) -> Vec<WebChoiceView> {
    prompt
        .options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let is_tagged = option.is_numbered_choice();
            let mut class_name = match (option.enabled, is_tagged) {
                (false, true) => "choice-card is-disabled is-tagged",
                (false, false) => "choice-card is-disabled",
                (true, true) => "choice-card is-tagged",
                (true, false) => "choice-card",
            }
            .to_string();
            if option.enabled && pressed_choice_index == Some(index) {
                class_name.push_str(" is-pressed");
            }

            WebChoiceView {
                index,
                display_number: (index + 1).to_string(),
                segments: parse_style_markup_with_tags(&option.label, &option.tags, option.enabled)
                    .into_iter()
                    .map(web_text_segment)
                    .collect(),
                class_name,
                enabled: option.enabled,
                base_style_attr: web_style_attr(&style_from_tags(&option.tags, option.enabled)),
            }
        })
        .collect()
}

fn next_paragraph_boundary(text: &str, current_byte: usize) -> usize {
    if current_byte >= text.len() {
        return text.len();
    }

    let hidden_text = &text[current_byte..];
    let visible_paragraph_start = hidden_text
        .char_indices()
        .find_map(|(offset, character)| (character != '\n').then_some(offset))
        .unwrap_or(hidden_text.len());

    if visible_paragraph_start >= hidden_text.len() {
        return text.len();
    }

    let paragraph_text = &hidden_text[visible_paragraph_start..];
    paragraph_text
        .find("\n\n")
        .map(|boundary| current_byte + visible_paragraph_start + boundary)
        .unwrap_or(text.len())
}

fn plain_paragraph(text: &str) -> WebParagraphView {
    styled_paragraph(text)
}

fn styled_paragraph(text: &str) -> WebParagraphView {
    WebParagraphView {
        segments: parse_style_markup(text)
            .into_iter()
            .map(web_text_segment)
            .collect(),
    }
}

fn web_text_segment(segment: StyledSegment) -> WebTextSegment {
    WebTextSegment {
        text: segment.text,
        style_attr: web_style_attr(&segment.style),
    }
}

fn web_style_attr(style: &TextStyle) -> String {
    let Some(color) = style.color.as_deref().and_then(css_color_value) else {
        return String::new();
    };

    format!("color: {color};")
}

fn css_color_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    if let Some(hex) = value.strip_prefix('#') {
        if (hex.len() == 3 || hex.len() == 6)
            && hex.chars().all(|character| character.is_ascii_hexdigit())
        {
            return Some(format!("#{hex}"));
        }
    }

    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return Some(value.to_ascii_lowercase());
    }

    None
}

fn sync_browser_scroll(sync: ScrollSync) {
    let script = format!(
        r#"
const transcriptRevision = "{transcript_revision}";

requestAnimationFrame(() => {{
  const transcript = document.querySelector(".transcript-scroll");
  if (transcript && transcript.dataset.scrollRevision !== transcriptRevision) {{
    transcript.scrollTop = transcript.scrollHeight;
    transcript.dataset.scrollRevision = transcriptRevision;
  }}
}});
"#,
        transcript_revision = sync.transcript_revision,
    );

    let _ = document::eval(&script);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_paragraph_boundary_reveals_until_next_blank_line() {
        let text = "First paragraph.\n\nSecond paragraph.";

        assert_eq!(next_paragraph_boundary(text, 0), "First paragraph.".len());
        assert_eq!(
            next_paragraph_boundary(text, "First paragraph.".len()),
            text.len()
        );
    }

    #[test]
    fn css_color_value_accepts_safe_color_values() {
        assert_eq!(css_color_value("red").as_deref(), Some("red"));
        assert_eq!(css_color_value("#0af").as_deref(), Some("#0af"));
        assert_eq!(css_color_value("bad;color"), None);
    }
}
