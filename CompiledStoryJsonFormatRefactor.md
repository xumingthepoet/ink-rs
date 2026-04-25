# Compiled Story JSON Format Refactor

## Purpose

The compiler and runtime currently both know the compiled story JSON wire
format.

Current shape:

```text
compiler parsed model
  -> compiler lower::ir::RuntimeProgram / RuntimeObject
  -> compiler emit.rs
  -> JSON
  -> runtime json_read.rs / json_read_stream.rs
  -> runtime Container / RTObject execution graph
```

This duplicates the format in two places:

- `crates/ink-compiler/src/lower/ir.rs` defines runtime-shaped compiler IR.
- `crates/ink-compiler/src/emit.rs` writes JSON tokens.
- `crates/ink-runtime/src/json/json_read.rs` and
  `crates/ink-runtime/src/json/json_write.rs` read and write many of the same
  tokens again.
- `crates/ink-runtime/src/control_command.rs`,
  `crates/ink-runtime/src/native_function_call.rs`, and related runtime object
  types also duplicate command and token names.

The long-term target is to move the compiled story JSON schema into one crate:

```text
crates/ink-story-json-format
```

Both `ink-compiler` and `ink-runtime` should depend on this crate directly.

## Final Architecture

Target shape:

```text
compiler parsed model
  -> ink_story_json_format::Program
  -> JSON
```

```text
JSON
  -> ink_story_json_format::Program
  -> runtime executable Container / RTObject graph
```

`ink-story-json-format` is only the typed compiled story format. It owns:

- data structures for the compiled story JSON format
- token definitions for the JSON wire format
- memory-to-JSON serialization
- JSON-to-memory deserialization
- format version constants
- focused format tests and roundtrip tests

It must not own:

- parser structures
- analysis structures
- compiler lowering state
- runtime execution state
- callstack state
- variable state
- `Rc<dyn RTObject>` graphs
- runtime parent pointers
- runtime navigation or evaluation behavior
- save-state-only runtime objects unless they are explicitly split into a
  separate save-state format module

The final state should not contain a permanent adapter or wrapper layer around
the new crate. During migration, temporary conversion code is acceptable, but it
must be treated as scaffolding and deleted before the refactor is considered
complete.

Final ownership rules:

- The compiler lowers directly into `ink_story_json_format::Program`.
- The compiler does not keep a parallel `RuntimeProgram`, `RuntimeObject`, or
  `ControlCommand` schema for compiled JSON output.
- The runtime reads compiled story JSON through `ink_story_json_format` and
  consumes the resulting data directly when building its executable graph.
- The runtime does not keep a parallel story JSON reader/writer that redefines
  the same compiled story wire tokens.
- Runtime execution types remain runtime-owned, but they are not the JSON schema.

## Format Crate Scope

The new crate should be small and stable:

```text
crates/ink-story-json-format/
  Cargo.toml
  src/
    lib.rs
    program.rs
    container.rs
    object.rs
    command.rs
    native_function.rs
    json.rs
    token.rs
    error.rs
```

Suggested public concepts:

- `Program`
- `Container`
- `Object`
- `ControlCommand`
- `NativeFunction`
- `Divert`
- `DivertKind`
- `ChoicePoint`
- `VariableAssignment`
- `VariableReference`
- `VariablePointer`
- `Value`
- `FormatError`

Suggested public JSON API:

- `Program::from_json_str(&str) -> Result<Program, FormatError>`
- `Program::to_json_string(&self) -> Result<String, FormatError>`
- `Program::from_json_value(serde_json::Value) -> Result<Program, FormatError>`
- `Program::to_json_value(&self) -> Result<serde_json::Value, FormatError>`

The exact API can change, but the crate should expose data plus JSON codecs, not
compiler or runtime behavior.

## Data Model Notes

The format data model should represent the JSON story shape, not a compiler
convenience shape.

Important distinctions:

- Compiler-only fields like `merge_tail_metadata` should not become permanent
  format data.
- Compiler-only variants like `NamedContent(Vec<Container>)` should be replaced
  by first-class container named content.
- Runtime-only parent pointers and `named_content` indexes should not be stored
  in the format crate.
- Control command token mapping should live in one place.
- Native function token mapping should live in one place, including the `"^"` /
  `"L^"` collision rule.
- `listDefs` should remain represented even while the current compiler emits an
  empty object.
- Unsupported list value tokens should have explicit format errors or explicit
  unsupported-state handling, not accidental runtime parse failures.

A likely core shape:

```rust
pub struct Program {
    pub ink_version: i32,
    pub root: Container,
    pub list_defs: ListDefinitions,
}

pub struct Container {
    pub content: Vec<Object>,
    pub named_content: Vec<NamedContainer>,
    pub name: Option<String>,
    pub flags: Option<i32>,
}

pub struct NamedContainer {
    pub name: String,
    pub container: Container,
}
```

Use map storage only if deterministic serialization remains guaranteed.

## Migration Plan

### Step 1: Add the crate

Add `crates/ink-story-json-format` to the workspace.

Initial contents:

- data structures for the currently emitted compiled story JSON
- JSON writer equivalent to compiler `emit.rs`
- JSON reader equivalent to runtime `json_read.rs` for compiled story objects
- unit tests for each token family
- roundtrip tests for representative story JSON snippets

Do not change compiler or runtime behavior in this step beyond adding the crate
and proving the crate can read and write the existing format.

Validation:

```text
cargo test -p ink-story-json-format
cargo fmt --all --check
```

### Step 2: Move token definitions

Move compiled story token names into the format crate.

Examples:

- `"ev"`, `"out"`, `"/ev"`
- `"str"`, `"/str"`
- `"done"`, `"end"`
- `"visit"`, `"seq"`, `"du"`, `"nop"`, `"pop"`
- `"choiceCnt"`, `"turn"`, `"turns"`, `"readc"`
- `"rnd"`, `"srnd"`, `"range"`, `"lrnd"`
- `"->"`, `"f()"`, `"->t->"`, `"x()"`
- `"^->"`, `"CNT?"`, `"VAR?"`, `"VAR="`, `"temp="`, `"^var"`
- `"*"`, `"flg"`, `"#f"`, `"#n"`
- `"<>"`, `"void"`, `"#"`, `"/#"`

The runtime can still have execution enums, but the JSON spelling must come
from the format crate.

Validation:

```text
cargo test -p ink-story-json-format
cargo test -p ink-runtime json
```

### Step 3: Make compiler emit through the format crate

Temporarily convert compiler `lower::ir::RuntimeProgram` into
`ink_story_json_format::Program` inside compiler code, then call the format
crate JSON writer.

This is a migration-only bridge. It exists to keep the change reviewable while
proving that the format crate emits byte-equivalent JSON.

Validation:

```text
cargo test -p ink-compiler
cargo test -p ink-test --test conformance
cargo test --workspace
```

### Step 4: Lower compiler output directly into format data

Replace compiler `lower::ir::RuntimeProgram`, `Container`, `RuntimeObject`, and
`ControlCommand` with `ink_story_json_format` types.

After this step:

- `Compiler::lower` returns `ink_story_json_format::Program`.
- `CompiledStory::program` stores `ink_story_json_format::Program`.
- `ink-compiler/src/emit.rs` is deleted or reduced to a tiny public-stage
  wrapper around `Program::to_json_string`.
- The temporary compiler conversion bridge from Step 3 is deleted.

Validation:

```text
cargo test -p ink-compiler
cargo test -p ink-test --test conformance
cargo test --workspace
```

### Step 5: Make runtime load through the format crate

Change runtime story loading so compiled story JSON is parsed by
`ink_story_json_format::Program::from_json_str`.

Then build the runtime executable graph directly from the format `Program`.

This graph-building code belongs in `ink-runtime` because it creates runtime
execution objects, parent pointers, named-content indexes, and `Rc` ownership.
It must not redefine JSON token parsing.

After this step:

- `json_read.rs` no longer owns compiled story JSON token parsing.
- runtime compiled-story loading depends on `ink_story_json_format`.
- version checks use format crate constants where appropriate.
- runtime execution object construction remains runtime-owned.

Validation:

```text
cargo test -p ink-runtime
cargo test --workspace
```

### Step 6: Remove runtime compiled-story JSON writer duplication

Separate compiled-story JSON writing from runtime save-state writing.

`json_write.rs` currently writes runtime objects for save state as well as
story-shaped objects. The compiled story writer should come from
`ink-story-json-format`; save-state writing can remain runtime-owned until a
separate save-state format is designed.

After this step:

- compiled story JSON writing lives only in `ink-story-json-format`
- runtime save-state JSON code does not pretend to be the compiled story schema
- shared token spelling still comes from the format crate where save-state uses
  compiled-story object tokens

Validation:

```text
cargo test -p ink-runtime
cargo test --workspace
```

### Step 7: Delete temporary adapters and wrappers

Remove all migration scaffolding.

The refactor is not complete while any of these remain:

- compiler-local `RuntimeProgram` or `RuntimeObject` mirrors
- compiler conversion from local runtime-shaped IR to format IR
- runtime conversion from duplicated JSON token enums to format enums
- wrapper APIs that hide the format crate behind another story JSON schema
- two independent command-token or native-function-token lookup tables

Allowed final conversion:

- runtime code that consumes `ink_story_json_format::Program` and constructs
  runtime execution objects

Not allowed final conversion:

- a separate adapter layer whose only job is translating one compiled story
  schema into another compiled story schema

Validation:

```text
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
make gate
```

## Completion Criteria

The work is complete when:

- `ink-compiler` depends directly on `ink-story-json-format`.
- `ink-runtime` depends directly on `ink-story-json-format`.
- the format crate owns compiled story data structures and JSON codecs.
- compiler lowering outputs format crate data directly.
- compiler JSON emit uses the format crate writer.
- runtime compiled-story load uses the format crate reader.
- runtime execution objects remain runtime-owned.
- there is no permanent adapter or wrapper layer around the format crate.
- compiled story JSON token names are not maintained independently in compiler
  and runtime.
- `make gate` passes.

## Non-Goals

This refactor should not change the Ink language by itself.

It should not:

- change parser behavior
- change analysis behavior
- change runtime execution semantics
- change the JSON format unless a separate documented format change requires it
- merge compiler and runtime object models
- make compiler depend on runtime execution objects
- make runtime depend on compiler lowering internals

Any intentional format change discovered during this work must be documented in
`docs/ink_JSON_runtime_format.md` and covered by focused tests.
