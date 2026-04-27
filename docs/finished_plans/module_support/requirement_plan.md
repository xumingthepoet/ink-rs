# Module Support Requirement Plan

Status: implemented requirements captured from the project owner on 2026-04-26,
normalized into the active plan on 2026-04-27, and closed with the
compatibility note below.

This document records first-phase module support requirements for Ink source
files. Keep detailed implementation sequencing in `task_list.md`.

## Goal

Define a module system for reusable Ink story code that replaces the current
"concatenate every source file and compile the result" model without weakening
compiler diagnostics, compiled story JSON compatibility, or runtime loading
guarantees.

The first phase intentionally diverges from upstream Ink by removing `INCLUDE`
and requiring explicit `module` declarations plus explicit `IMPORT`
dependencies.

## Module Syntax

- A module is declared with a header-like line:

  ```ink
  === module items ===
  ```

- Header delimiter levels are semantic:
  - `=== module name ===` declares a module.
  - `== knot_name ==` declares a knot.
  - `== function function_name(...) => T ==` declares a function.
  - `= stitch_name` declares a stitch.
- Module syntax keywords are case-sensitive:
  - `module` is lowercase.
  - `IMPORT` and `FROM` are uppercase.
- Module declarations do not accept parameters.
- Module names are single identifiers. Submodules and hierarchical module names
  are not supported in the first module-support phase.
- Module names are declared in source and are not inferred from, or required to
  match, file names or file paths.
- A source file may contain multiple different module declarations.
- A module block starts at its module header and continues until the next module
  header or the end of the file.
- A module may be declared only once in a compilation. Reopening the same module
  in another file or later in the same file is a compile error.
- There is no implicit empty/root module.
- Content or module-scoped top-level declarations before the first module
  header are compile errors.
- Module-level direct story content is not allowed. All story content must be
  inside a knot or stitch.
- Tags remain content metadata on knots, stitches, choices, and content. Tags do
  not belong to the module namespace and are not importable.

## Source/Input Model

- First-phase module compilation uses an explicit multi-source input API such
  as `Compiler::compile_sources(Vec<SourceInput>)`.
- The compiler must not discover source files by scanning directories. The
  caller is responsible for passing every source file in the compilation unit.
- Source file order does not affect module visibility, import resolution, or
  lowering semantics.
- `Compiler::compile(SourceInput)` remains a single-file convenience path and
  should delegate to the multi-source path with one input.
- A single source input may contain multiple explicit modules.
- The compiler considers every supplied Ink source input for parsing and
  semantic checking.
- `INCLUDE` is removed in the first module-support phase. Source files must use
  explicit modules and `IMPORT`.
- `INCLUDE` should produce a clear unsupported/removed construct diagnostic.
- The old include-expansion `FileHandler` path is no longer a language
  mechanism. Remove or retire the compiler option/API surface that only existed
  to support `INCLUDE`.

## Namespace And Visibility

- Module boundaries are language-level name boundaries.
- The following declarations belong to a module namespace:
  - knots
  - functions
  - constants
  - global variables
  - structs
  - externals
- Knots, functions, constants, global variables, structs, and externals share
  one namespace within a module and may not reuse the same name in that module.
- The same symbol name may appear in different modules.
- Stitches do not belong to the module namespace. Stitch names remain scoped to
  their parent knot, so different knots in the same module may have stitches
  with the same name.
- Same-module `knot.stitch` addressing remains supported.
- Cross-module direct stitch access is not allowed.
- `temp` declarations remain local runtime declarations and do not belong to
  the module namespace.
- A module may reference its own knots, functions, constants, global variables,
  structs, and externals by unqualified name.
- Unqualified references resolve only within the current module.
- Cross-module references must use source-level module qualification with `::`,
  for example `items::take`.
- The module containing `main` has no special visibility privileges. It must
  explicitly import every cross-module symbol it references.

## Imports

- Initial import syntax is single-line only:

  ```ink
  IMPORT knotName, functionName FROM moduleName
  ```

- Imports are declared inside a module.
- All imports in a module must appear before that module's `CONST`, `VAR`,
  `STRUCT`, `EXTERNAL`, function, knot, stitch, tag, and content declarations.
- `IMPORT`, `CONST`, global `VAR`, `STRUCT`, and `EXTERNAL` declarations are
  allowed only at module top level, not inside knots or stitches.
- Import aliases are not supported in the first module-support phase.
- Import names do not carry explicit kind annotations because importable
  declarations share one module namespace.
- The first module-support phase allows importing:
  - knots
  - functions
  - constants
  - global variables
  - structs
  - externals
- Stitches and tags are not importable in this phase.
- An `IMPORT` list is an exact allow-list. A module may only reference imported
  symbols that were explicitly named in its own `IMPORT ... FROM ...`
  declarations.
- Imports do not introduce unqualified symbol visibility.
- Imports are not transitive. A module may not access symbols imported by one of
  its dependencies unless it also imports those symbols directly from their
  declaring module.
- Imported knots, functions, constants, global variables, structs, and externals
  are referenced through qualified names:
  - `moduleName::knotName`
  - `moduleName::functionName`
  - `moduleName::constName`
  - `moduleName::varName`
  - `moduleName::StructName`
  - `moduleName::externalName`
- `IMPORT` declarations are compile-time checked. The source module must exist,
  every imported symbol must exist in that module, and each imported symbol must
  be a supported import kind for the current phase.
- Imported symbols that are never used by the importing module produce warnings.
- Module `VAR` declarations are story-wide runtime state.
- Importing and referencing the same module variable from multiple modules reads
  and writes the same underlying story variable.
- Imported module variables may be read and written through qualified names,
  including assignment and compound assignment forms supported for local or
  global variables.
- `EXTERNAL` declarations are allowed only at module top level and belong to the
  module namespace.
- Host bindings for module `EXTERNAL` declarations use the source-qualified
  name, for example `audio::play`.

## Entry Point And Reachability

- A runnable compiled story must contain exactly one explicit module that
  defines a knot named `main`.
- Library-only compilation units are not part of the first module-support
  phase.
- A compilation with no `main` knot is a compile error.
- Multiple `main` knots are compile errors, even when they are in different
  modules.
- The runnable entry point is the unique `moduleName::main` knot.
- Import resolution is based on module boundaries, not text concatenation order.
- Compilation order is dependency-driven. Modules imported by another module are
  checked and lowered before dependent modules where ordering matters.
- Cyclic module imports are compile errors.
- After dependency analysis, any module that is not reachable from the `main`
  module's transitive dependency path produces a warning.
- Modules that are not reachable from the `main` module's transitive dependency
  path are still parsed and semantically checked.
- Errors in unreachable modules still fail the overall compilation.
- Unreachable modules are not emitted into final runnable compiled story JSON.

## Diagnostics And Warnings

The compiler must report clear diagnostics for:

- `INCLUDE` use after removal.
- Missing module declarations before module-scoped declarations or content.
- Module-level direct story content.
- Module declarations with wrong delimiter levels, wrong keyword case, invalid
  names, hierarchical names, or parameters.
- Duplicate module declarations in a compilation.
- Duplicate module namespace symbols within one module.
- `IMPORT` declarations outside modules, inside knots/stitches, after other
  module declarations, or using unsupported multiline syntax.
- Missing imported modules, missing imported names, unsupported imported kinds,
  and attempts to import stitches or tags.
- Qualified references to modules or symbols that were not explicitly imported.
- Cross-module direct stitch access.
- Missing `main`, multiple `main` knots, import cycles, and unreachable modules.

Warnings must be emitted for:

- Imported symbols that are never used by the importing module.
- Modules that are semantically valid but not reachable from the `main` module's
  transitive dependency path.

Warnings do not suppress errors. Errors in any supplied module fail the
compilation, even if the module is unreachable from `main`.

## Lowering / JSON / Runtime Compatibility

- Preserve the existing compiled story JSON schema unless an explicit,
  documented runtime-format change becomes unavoidable.
- Use existing format-crate `Program`, `Container`, and `Object` structures for
  emitted runnable story JSON.
- The default lowering representation is:
  - root named content contains one container per emitted reachable module
  - each emitted module container owns its reachable knots and functions as
    named child containers
  - source `moduleName::flowName` maps to runtime path `moduleName.flowName`
    for internal diverts, function calls, and divert-target values
  - source same-module unqualified flow references resolve to
    `currentModule.flowName`
  - stitches remain nested under their parent knot using existing runtime path
    conventions
- Root story content should auto-divert to the unique runtime path for
  `moduleName::main` and then complete normally.
- Global variables should use stable module-qualified runtime variable names,
  for example `items::count`, so same-named globals in different modules do not
  collide and imported references share the same underlying state.
- External declarations and external calls should use the source-qualified host
  binding name, for example `audio::play`.
- Constants and struct types are compile-time module namespace symbols. They do
  not require new runtime JSON schema fields unless a later task discovers a
  concrete compatibility need.
- Runtime save-state JSON is out of scope for this phase except for preserving
  module-qualified global variable names in saved state.
- Runtime execution objects remain runtime-owned. The format crate continues to
  own only the compiled story JSON wire model and codec.

## Compatibility Note

This module-support phase documents explicit modules and imports as current
ink-rs language behavior. For compatibility with unchanged upstream fixtures
and C# compatibility coverage, the one-source `Compiler::compile(SourceInput)`
path still accepts legacy no-module stories. That compatibility path does not
provide module imports or current documentation examples, and `INCLUDE` remains
removed. Explicit module sources and multi-source module compilation enforce the
module ownership, entry-point, import, and namespace rules described above.
Removing the legacy no-module path is a separate follow-up language migration,
not part of this closed first phase.

## Acceptance Criteria

- Parser, parsed model, analysis, lowering, format, runtime-loading, tests, and
  maintained documentation all represent the module system consistently.
- Existing text-concatenation and `INCLUDE` semantics are no longer accepted as
  current language behavior.
- Imports are exact, explicit, non-transitive dependencies with no global
  visibility leaks.
- The compiled story JSON remains loadable by the existing runtime through
  `ink-story-json-format`.
- Intentional divergence from upstream Ink behavior is documented in
  `docs/WritingWithInk-updates.md` and reflected in
  `docs/WritingWithInk-latest.md`.
- Focused module tests pass before broad validation.
- `make gate` passes before this plan is moved to `docs/finished_plans/`.

## Requirement Log

- 2026-04-26: Create this requirements document under
  `docs/active_plan/module_support/` as the active tracking location for
  module support requirements.
- 2026-04-26: Redesign include/import semantics around explicit dependencies,
  no default transitive or sibling visibility leakage, module namespaces for
  top-level declarations, and import resolution based on module/file boundaries
  rather than text concatenation order.
- 2026-04-26: The namespace boundary is named `module`; explicit declarations
  use syntax similar to existing function/flow headers.
- 2026-04-26: A source file may contain multiple modules. Module names are
  source declarations such as `=== module items ===` and have no required
  relationship to file names or file paths.
- 2026-04-27: First-phase module compilation uses an explicit multi-source input
  API. The compiler does not discover files by scanning directories.
- 2026-04-27: `Compiler::compile(SourceInput)` remains a single-source
  convenience path for source that contains explicit module declarations.
- 2026-04-27: A module may be declared only once in a compilation. It cannot be
  reopened in another file or later in the same file, though one file may
  contain multiple different modules.
- 2026-04-27: Submodules and hierarchical module names are not supported in the
  first phase. Module names are single identifiers.
- 2026-04-26: `VAR` declarations must be inside a module boundary.
- 2026-04-26: There is no implicit empty/root module. Module-scoped top-level
  declarations must belong to explicit module declarations, and the required
  `main` knot must be inside an explicit module.
- 2026-04-27: Library compilation units are not part of the first
  module-support phase. A valid compilation must have exactly one `main` knot.
  Missing `main` and multiple `main` knots are compile errors.
- 2026-04-27: The runnable entry point is the unique `moduleName::main` knot.
- 2026-04-27: Content or module-scoped top-level declarations before the first
  module header are errors.
- 2026-04-27: Module-level direct story content is not allowed. All story
  content must be inside a knot or stitch.
- 2026-04-27: Header delimiter levels are semantic. Modules use `===`, knots
  and functions use `==`, and stitches use `=`.
- 2026-04-27: Module syntax keywords are case-sensitive: `module` is lowercase,
  while `IMPORT` and `FROM` are uppercase.
- 2026-04-27: Module declarations do not accept parameters.
- 2026-04-27: `CONST` declarations belong to the module namespace.
- 2026-04-27: Knots, functions, constants, global variables, structs, and
  externals share one module namespace and may not reuse the same name within
  the same module. `IMPORT` does not need kind annotations.
- 2026-04-27: `EXTERNAL` declarations are allowed only at module top level,
  belong to the module namespace, and are importable in the first phase.
- 2026-04-27: Host bindings for module `EXTERNAL` declarations use the
  module-qualified name, for example `audio::play`.
- 2026-04-27: Stitches remain scoped to their parent knot and are not part of
  the module namespace. Different knots in one module may reuse stitch names.
- 2026-04-27: Cross-module direct stitch access is not allowed.
- 2026-04-27: Same-module `knot.stitch` addressing remains supported.
- 2026-04-27: Imports are module-local declarations at the start of a module.
  Initial syntax is `IMPORT name, name FROM moduleName`; knots, functions,
  constants, global variables, structs, and externals are importable in the
  first phase, and imported symbols are used via qualified names.
- 2026-04-27: `IMPORT` declarations are single-line only in the first phase.
- 2026-04-27: Import lists are exact allow-lists. Referencing
  `moduleName::symbol` is only valid when that symbol was explicitly named by an
  import declaration in the current module.
- 2026-04-27: The module containing `main` has no special visibility privileges
  and must explicitly import every cross-module symbol it references.
- 2026-04-27: Source-level module qualification semantics use `::`. Compiled
  story JSON and runtime internals use compatible representations while
  preserving the source language contract.
- 2026-04-27: Module `VAR` declarations remain story-wide runtime state.
  Multiple modules that import and reference the same module variable read and
  write the same underlying story variable.
- 2026-04-27: Imported module variables may be read and written through their
  qualified names, including assignment and compound assignment forms supported
  for local variables.
- 2026-04-27: Imports are not transitive. There is no import chain visibility
  and no default re-export behavior.
- 2026-04-27: A module may reference its own knots, functions, constants, global
  variables, structs, and externals by unqualified name. Unqualified references
  resolve only within the current module; cross-module references must use
  `moduleName::symbol`.
- 2026-04-27: Import aliases are not supported in the first module-support
  phase.
- 2026-04-27: Invalid imports are compile errors, including missing source
  modules, missing imported names, and imported names whose kind is not
  supported by the current phase.
- 2026-04-27: Imported symbols that are never used by the importing module
  produce warnings.
- 2026-04-27: All imports in a module must appear before that module's `CONST`,
  `VAR`, `STRUCT`, `EXTERNAL`, function, knot, stitch, tag, and content
  declarations.
- 2026-04-27: `IMPORT`, `CONST`, global `VAR`, `STRUCT`, and `EXTERNAL`
  declarations are allowed only at module top level, not inside knots or
  stitches.
- 2026-04-27: `temp` declarations remain local runtime declarations and do not
  belong to the module namespace.
- 2026-04-27: The compiler considers all supplied Ink files independent of file
  order, compiles imported dependency modules before dependents where ordering
  matters, tracks fully compiled modules in a completed module set, and warns
  for modules not reachable from the `main` module's transitive dependency path.
- 2026-04-27: Modules not reachable from the `main` module's transitive
  dependency path are parsed and semantically checked, but are not emitted into
  final runnable compiled story JSON.
- 2026-04-27: Errors in unreachable modules still fail the overall compilation.
- 2026-04-27: Cyclic module imports are compile errors.
- 2026-04-27: `INCLUDE` is removed in the first module-support phase. Source
  files must use modules and `IMPORT`; `INCLUDE` should be diagnosed as an
  unsupported/removed construct.
- 2026-04-27: No compiled story JSON schema change is planned by default.
  Lowering should use existing containers and runtime paths, with source
  `module::symbol` mapped to compatible internals.
- 2026-04-27: Runtime save-state JSON is out of scope except for preserving
  module-qualified global variable names in saved state.
- 2026-04-27: First-phase closeout keeps the one-source legacy no-module
  compatibility path for unchanged C# and conformance coverage. Current ink-rs
  documentation and module tests use explicit modules; deleting the legacy root
  path is deferred to a separate migration.
