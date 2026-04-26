# Ink RS Syntax

VS Code syntax highlighting for ink-rs `.ink` and `.ink2` files.

This is an independent TextMate grammar for ink-rs. It is intentionally kept
outside `ink-csharp/` and covers the ink-rs language surface directly:

- `STRUCT Name { field: Type }`
- typed `VAR` and `~ temp` declarations
- typed `== function name(args) => Type ==` declarations
- typed `EXTERNAL name(args) => Type` declarations
- divert target value types such as `: ->` and `: ->[]`
- explicit dynamic diverts such as `-> {next}` and `-> {route.next}(arg)`
- choices, gathers, knots, stitches, diverts, glue, tags, comments, and inline
  logic

## Local Install

```sh
mkdir -p ~/.vscode/extensions
ln -s "$(pwd)/editor/vscode-ink-rs" ~/.vscode/extensions/ink-rs.ink-rs-syntax-0.1.0
```

Reload VS Code after creating or updating the symlink.
