# ink-dioxus

`ink-dioxus` is the reusable Dioxus game engine layer for ink-rs games. It keeps
game state, rules, and content in ink-rs source while providing shared Rust UI
shells for choice-based games.

The adapter understands these ink-rs tags:

- `# title` or `# title:...` consumes output and updates the story title.
- `# prompt` or `# prompt:...` consumes output and updates the choice prompt.
- `# toast` or `# toast:...` consumes output and shows a transient toast.
- `# enabled:false` disables a choice.
- `# menu` or `# numbered` marks a choice as a numbered menu item.
- `# color:...` styles output or choices.

Inline `[style color=...]...[/style]` markup is also supported for text that
needs styling inside a line or choice label.

The web shell auto-saves when a fully revealed choice prompt is waiting for
input. The runtime save stores the pre-choice execution snapshot and the web
shell stores transcript/title UI state alongside it, so loading regenerates the
same prompt without serializing choices into language save-state JSON.

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

## Web Game Catalog

Game hubs can generate a catalog from one directory per game:

```rust
// build.rs
fn main() {
    ink_dioxus::build::generate_ink_game_catalog("assets/ink")
        .expect("ink-rs game catalog should generate");
}
```

Then launch the generated catalog from the web binary:

```rust
// src/main.rs
include!(concat!(env!("OUT_DIR"), "/ink_games.rs"));

fn main() {
    ink_dioxus::web::launch(
        ink_dioxus::web::WebLaunchConfig::new_catalog(INK_GAMES)
            .with_app_label("TEXT GAMES")
            .with_storage_key("text_games.web_save.v1"),
    );
}
```
