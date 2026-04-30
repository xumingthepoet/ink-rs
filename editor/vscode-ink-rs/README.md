# Ink RS Syntax

VS Code syntax highlighting for ink-rs `.ink` and `.ink2` files.

This is an independent TextMate grammar for ink-rs. It covers the ink-rs
language surface directly:

- `STRUCT Name { field: Type }`
- explicit `=== module name ===` declarations
- `IMPORT name FROM module` and `IMPORT { ... } FROM module` declarations
- typed `VAR` and `~ temp` declarations
- typed `== function name(args) => Type ==` declarations
- typed `EXTERNAL name(args) => Type` declarations
- explicit multiline `{ if ...: }`, `{ switch ...: }`, and `- else:` control
  blocks
- `module::symbol` qualified names
- divert target value types such as `: ->` and `: ->[]`
- explicit dynamic diverts such as `-> {next}` and `-> {route.next}(arg)`
- choices, gathers, knots, stitches, diverts, glue, tags, comments, and inline
  logic

Removed `INCLUDE` statements are highlighted as deprecated/invalid syntax
rather than as current import syntax.

## Local Install

```sh
mkdir -p ~/.vscode/extensions
ln -s "$(pwd)/editor/vscode-ink-rs" ~/.vscode/extensions/ink-rs.ink-rs-syntax-0.1.0
```

Reload VS Code after creating or updating the symlink.
