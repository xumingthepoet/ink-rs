# ink-rs

`ink-rs` is a Rust implementation and language fork of Ink for interactive
narrative games. The root crate is a small facade for game projects; lower-level
crates remain available when a project wants tighter dependency control.

## Crates

- `ink-rs`: facade crate for common game integration.
- `ink-runtime`: runtime story engine for compiled story JSON.
- `ink-compiler`: compiler from ink-rs source to compiled story JSON.
- `ink-story-json-format`: typed compiled story JSON wire model and codec.

`ink-test` and `ink-tools` are repository-internal crates and are not published.

## Features

The facade enables the runtime by default.

```toml
[dependencies]
ink-rs = "0.1"
```

Enable source compilation when a game tool, editor integration, build script, or
mod pipeline needs to compile ink-rs source:

```toml
[dependencies]
ink-rs = { version = "0.1", features = ["compiler"] }
```

Use `full` to enable both runtime and compiler APIs explicitly.

## Runtime Use

```rust
use ink_rs::{Story, StoryError};

fn run_story(json: &str) -> Result<(), StoryError> {
    let mut story = Story::new(json)?;

    while story.can_continue() {
        println!("{}", story.cont()?);
    }

    Ok(())
}
```

## Compiler Use

```rust
use ink_rs::{Compiler, SourceInput};

fn compile_story() -> Option<String> {
    let source = r#"
=== module game ===

== main ==
Hello world.
-> DONE
"#;

    let output = Compiler::new().compile(SourceInput::new(source));
    if output.has_errors() {
        for diagnostic in output.diagnostics {
            eprintln!("{diagnostic:?}");
        }
        None
    } else {
        Some(output.artifact.unwrap().json)
    }
}
```

## Documentation

- `docs/WritingWithInk-latest.md`: maintained ink-rs writing guide.
- `docs/WritingWithInk-updates.md`: syntax and semantic changes from upstream
  Ink.
- `docs/Architecture.md`: crate and data-flow architecture notes.
- `docs/ink_JSON_runtime_format.md`: compiled story JSON format notes.
