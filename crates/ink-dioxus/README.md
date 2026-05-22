# ink-dioxus

`ink-dioxus` is the reusable Dioxus adapter for ink-rs games. It keeps the game
state, rules, and content in ink-rs source while providing a shared Rust UI
shell for choice-based games.

The adapter understands these ink-rs tags:

- `# title` or `# title:...` consumes output and updates the story title.
- `# prompt` or `# prompt:...` consumes output and updates the choice prompt.
- `# toast` or `# toast:...` consumes output and shows a transient toast.
- `# enabled:false` disables a choice.
- `# menu` or `# numbered` marks a choice as a numbered menu item.
- `# color:...` styles output or choices.

Inline `[style color=...]...[/style]` markup is also supported for text that
needs styling inside a line or choice label.

## Web Game Skeleton

Add the adapter as both a normal dependency and a build dependency:

```toml
[dependencies]
ink-dioxus = { path = "../ink-rs/crates/ink-dioxus", features = ["web"] }

[build-dependencies]
ink-dioxus = { path = "../ink-rs/crates/ink-dioxus" }
```

Use the build helper once in a game crate so the `.ink` source list is generated
from `assets/ink/src`:

```rust
// build.rs
fn main() {
    ink_dioxus::build::generate_ink_source_list("assets/ink/src")
        .expect("ink-rs source list should generate");
}
```

Then launch the generated source list from the web binary:

```rust
// src/main.rs
include!(concat!(env!("OUT_DIR"), "/ink_sources.rs"));

fn main() {
    ink_dioxus::web::launch(
        ink_dioxus::web::WebLaunchConfig::new(INK_SOURCES)
            .with_app_label("TEXT RPG")
            .with_storage_key("text_rpg.web_save.v1")
            .with_default_story_title("ink-rs Story")
            .with_default_prompt_title("Choices"),
    );
}
```

After that, ordinary game content changes can live in `.ink` files.
