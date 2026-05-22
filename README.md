# ink-rs

`ink-rs` is a new domain-specific language for narrative games, implemented in
Rust. It is inspired by ink by inkle, but Ink compatibility is not a language
goal: ink-rs syntax, compiler behavior, runtime behavior, and save contracts are
defined by this repository.

The root crate is a small facade for game projects; lower-level crates remain
available when a project wants tighter dependency control.

## Crates

- `ink-rs`: facade crate for common game integration.
- `ink-runtime`: runtime story engine for compiled story JSON.
- `ink-compiler`: compiler from ink-rs source to compiled story JSON.
- `ink-story-json-format`: typed compiled story JSON wire model and codec.
- `ink-dioxus`: reusable Dioxus UI adapter for ink-rs game crates.
- `text-games-app`: playable web hub for bundled ink-rs text games.

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

## Dioxus Game Adapter

Game crates that want the shared Dioxus UI shell can depend on `ink-dioxus` with
the `web` feature. The game crate owns its `.ink` files and passes embedded
`InkSource` values to `ink_dioxus::web::launch`, or embedded `InkGameSource`
values to `ink_dioxus::web::WebLaunchConfig::new_catalog` for a multi-game hub.

`ink-dioxus` also exposes a build-script helper that generates that embedded
source list from `assets/ink/src`, so adding or renaming `.ink` files does not
require manually editing an `include_str!` array.

The repository app crate `text-games-app` uses the catalog path to serve all
bundled text games from one web entry point.

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

Host tooling can fail builds on warnings by setting a diagnostics policy:

```rust
use ink_rs::{Compiler, CompilerOptions, DiagnosticsPolicy, SourceInput};

let compiler = Compiler::with_options(CompilerOptions {
    source_filename: None,
    diagnostics_policy: DiagnosticsPolicy::DenyWarnings,
});
let result = compiler.compile_sources(vec![SourceInput::new("...")]);
assert!(result.failed());
```

## Compiler Use

```rust
use ink_rs::{compiler::format_diagnostics, Compiler, SourceInput};

fn compile_story() -> Option<String> {
    let source = r#"
=== module game ===

== main ==
Hello world.
-> DONE
"#;

    let output = Compiler::new().compile(SourceInput::new(source));
    if output.failed() {
        eprintln!("{}", format_diagnostics(&output.diagnostics));
        None
    } else {
        Some(output.artifact.unwrap().json)
    }
}
```

## Documentation

- [SyntaxReference.md](docs/SyntaxReference.md): current ink-rs syntax
  reference.
- [Architecture.md](docs/Architecture.md): crate and data-flow architecture
  notes.
- [ink_JSON_runtime_format.md](docs/ink_JSON_runtime_format.md): compiled
  story JSON format notes.
