# Architecture

## Overview

`ink-rs` is split into two conceptual layers:

- Compiler layer: implemented in this repository under `crates/ink-compiler`.
- Runtime layer: reused from the ignored local `blade-ink-rs/lib` crate.

The compiler layer parses `.ink` source, builds a parsed hierarchy, resolves
references, and exports runtime JSON. The runtime layer loads that JSON and
executes the story.

```text
.ink source
  -> InkParser
  -> parsed hierarchy
  -> reference resolution
  -> runtime JSON export
  -> bladeink::story::Story
```

## Compiler Crate Modules

- `compiler`: public orchestration API, equivalent to C# `Compiler.cs`.
- `parser`: ink parser entry point and future parser submodules.
- `parsed`: parsed hierarchy types, equivalent to C#
  `compiler/ParsedHierarchy`.
- `error`: compiler diagnostics and error types.

## C# Architecture Mapping

| C# area | Rust target | Purpose |
| --- | --- | --- |
| `Compiler.cs` | `compiler` | Parse and compile orchestration |
| `StringParser/` | `parser::string_parser` | Low-level parser state and rules |
| `InkParser/` | `parser::ink_parser` | Ink language grammar |
| `ParsedHierarchy/` | `parsed` | AST-like parsed object model |
| `ink-engine-runtime/` | `bladeink` dependency | Runtime story execution |

## Parsed Hierarchy Design

The C# compiler uses inheritance heavily. Rust should preserve the conceptual
model while using explicit enums, traits, owned structs, or reference-counted
nodes where appropriate.

Guidelines:

- Keep type names recognizable.
- Preserve behavior before optimizing representation.
- Document ownership choices when they differ from C# parent pointers.
- Avoid using a single large enum if it makes incremental porting harder.
- Prefer small tests for each parsed object behavior before integrating with
  the full parser.

### Current Ownership Model

The first parsed-hierarchy slice uses reference-counted tree nodes:

- `parsed::Object` is wrapped in `Rc<RefCell<_>>` when stored in a tree.
- Child nodes keep a `Weak` parent pointer to avoid ownership cycles.
- Tree traversal helpers operate on `ObjectRef` values and walk depth-first.
- Debug metadata inherits from ancestors when a node does not define its own
  metadata.
- Flow-level concepts are represented with `FlowLevel`, `NamedContent`, and
  `FlowBase` traits so future knot/stitch types can share a common interface.
- `Knot` and `Stitch` wrappers now build on the flow traits so line-based
  parser output has named flow nodes instead of anonymous placeholders.
- `Weave`, `Choice`, and `Gather` wrappers now model the C# weave-point layer
  with indentation-based grouping and local weave-point naming lookup, while
  the fuller parser/runtime wiring remains a separate slice.
- `Path::resolve_from_context` now resolves flow names, nested stitches, and
  weave points from the parsed tree using the same name-search structure as the
  C# compiler's path resolution rules.
- Content-node leaf types are represented as `ObjectKind`-backed wrappers so
  `ContentList`, `Text`, `AuthorWarning`, `Tag`, and `Divert` can live in the
  same tree model without changing traversal behavior.
- `InkParser::new` runs a comment-elimination pre-pass before the grammar
  layer sees the input, mirroring the C# compiler's preprocessing step.
- Whitespace handling lives in parser helpers that mirror the C# parser rules
  for newline, end-of-file, and spacing combinators.
- The current parser entry point has a temporary plain-text fallback that
  produces `ContentList`/`Text` nodes and basic flow nodes while the fuller
  grammar is still being ported.
- Basic knot and stitch headers are currently recognized by line prefix and
  attached to their nested body lines; this is intentionally a temporary
  parsing slice, not the final full grammar.
- Simple divert lines are now recognized by line prefix and become parsed
  `Divert` nodes; the full divert grammar is still pending.
- `INCLUDE` lines are now resolved through the compiler-owned `FileHandler`
  abstraction, recursively parsed, and flattened so non-flow content appears at
  the include site while flow content is appended to the owning story.

## Runtime Export

The compiler owns the JSON export path. It generates JSON compatible with
`bladeink::story::Story::new`.

The runtime reader's minimal accepted shape is:

- Top-level object keys: `inkVersion`, `root`, and `listDefs`.
- `inkVersion` is the story format version and must be numeric.
- `root` is a container array. Its final entry is either `null` or a trailing
  object containing named-content metadata such as `#n` and `#f`.
- `listDefs` is an object mapping list names to item dictionaries.
- Runtime tokens are encoded as compact JSON values:
  - string commands like `done`, `<>`, `^text`, `\n`, `->`, `->t->`, `f()`,
    and `x()`
  - object encodings like `{"^->":"path"}`, `{"*":"path"}`,
    `{"VAR?":"name"}`, `{"VAR=":"name"}`, and `{"#":"tag"}`

The current Rust exporter covers the minimal plain-text story slice and
collects parsed list declarations into top-level `listDefs` metadata, but
broader grammar export is still pending. It is compiler-owned and already
validated against the runtime loader.

Do not copy the runtime implementation into `ink-compiler`. If runtime internals
are private, prefer generating serialized JSON directly from compiler-owned
structures rather than making broad runtime visibility changes.

## Plugins

The official C# compiler supports plugin discovery and PreParse/PostParse/
PostExport hooks. This Rust port currently defers that feature entirely: the
compiler crate does not load plugins, expose plugin hook traits, or mutate the
story through plugin callbacks. Keep plugin support documented as an explicit
future scope item unless a compatibility requirement appears.

## Diagnostics

Diagnostics should carry:

- Severity: error or warning.
- Message.
- Source filename when available.
- Line and column.
- Optional debug metadata range.

The user-facing API should support collecting diagnostics even when compilation
cannot continue.

## Long-Horizon State

The codebase state is externalized through Markdown:

- `AGENTS.md`: target, constraints, and agent rules.
- `PLAN.md`: checkpointed milestones.
- `IMPLEMENT.md`: execution loop.
- `DOCUMENTATION.md`: current status and decisions.

This keeps repeated `继续` prompts grounded in repository state rather than chat
history.

## Architecture Risks

- JSON export compatibility depends on matching the runtime reader's expected
  shape, not just matching C# object names.
- Parser rollback must match C# closely because grammar rules depend on
  speculative parsing.
- Rust ownership for parsed parent/child relationships needs explicit design;
  do not hide it behind ad-hoc cloning.
- Reference resolution should be tested feature-by-feature to avoid late
  all-at-once failures.

## Demo Path

The final architecture should support this minimal demonstration:

1. Compile plain text ink to JSON.
2. Load it with `bladeink::story::Story::new`.
3. Continue the runtime story and print expected output.
4. Repeat with a story containing choices and diverts.
5. Repeat with variables, expressions, and conditions.
