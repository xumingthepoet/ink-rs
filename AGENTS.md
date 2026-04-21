# AGENTS.md

## Project Goal

This repository is a Rust port of the official ink compiler layer from the C#
implementation in `ink-csharp/`.

The runtime layer must be reused from the local `blade-ink-rs/lib` crate. Do not
rewrite runtime behavior unless the compiler layer needs a small integration
adapter.

## Source References

- `ink-csharp/`: official C# ink implementation. Use `ink-csharp/compiler` as
  the primary architecture and naming reference for the compiler port.
- `blade-ink-rs/`: unofficial Rust implementation. It currently covers the
  runtime layer only and is used as the runtime dependency.

Both directories are local reference trees and are intentionally ignored by the
root `.gitignore`.

## Architecture Direction

- Implement the Rust compiler layer under `crates/ink-compiler`.
- Keep compiler data structure names close to the C# compiler where practical:
  `Compiler`, `InkParser`, `StringParser`, `Parsed::Story`, `Parsed::Object`,
  `Choice`, `Divert`, `FlowBase`, `Weave`, and related parsed hierarchy types.
- Prefer Rust module naming conventions while preserving recognizable type
  names. For example, use `parsed::Story` and `parser::InkParser`.
- Treat `ink-csharp/compiler/ParsedHierarchy` as the source of truth for parsed
  AST structure and runtime export behavior.
- Treat `ink-csharp/compiler/InkParser` and `ink-csharp/compiler/StringParser`
  as the source of truth for parsing behavior.
- Export runtime JSON compatible with `bladeink::story::Story::new`.

## Constraints

- Do not vendor or track `ink-csharp/` or `blade-ink-rs/` in this repository.
- Do not duplicate runtime implementation from `blade-ink-rs/lib`.
- Keep initial ports small and testable. Port one compiler concept at a time
  instead of creating large unverified translations.
- Preserve behavior over idiomatic rewrites when porting from C#; idiomatic Rust
  is secondary to compatibility.
- Add tests with representative `.ink` snippets and expected compiled JSON or
  runtime behavior as soon as a feature is implemented.

## Useful Commands

```sh
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

These commands require the ignored local `blade-ink-rs/` directory to be
present because `crates/ink-compiler` depends on `blade-ink-rs/lib` by path.

