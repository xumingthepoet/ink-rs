# Dynamic Interface Calls Missing In Typed Initializer Contexts

Status: solved

Found while: Task 10 dynamic interface function call diagnostics

Scope: `crates/ink-compiler/src/analysis/initializers.rs`, `assignments.rs`, `array_literals.rs`, `struct_literals.rs`, `field_access.rs`, `index_access.rs`

Problem: Dynamic interface function calls type-check through the Task 10 expression inference path when the caller supplies an `InterfaceMemberIndex`, but several typed-value analysis passes still call plain `infer_expression_type`. A valid expression such as `{route}::score(1)` can therefore be rejected in a `temp` or global initializer before lowering is reached.

Why it matters: Interface methods are normal expressions from the language user's perspective. Leaving typed initializer and assignment contexts on the old inference path makes the feature feel inconsistent and can block common usage patterns.

Suggested fix: Thread `InterfaceMemberIndex` into the typed-value analysis passes and make interface-aware expression inference the normal `infer_expression_type` path wherever expression return types are compared to declared or expected types. Update `infer_expected_interface_expression_type` callers at the same time so dynamic calls returning interface values work consistently.

Evidence: `diagnostics/interface-dynamic-function-lowering.ink` originally used `~ temp value: int = {route}::score(1)` and produced initializer diagnostics before the intended dynamic function lowering diagnostic.

Resolution: `infer_expression_type` now requires `InterfaceMemberIndex`, typed-value analysis passes pass that index through their initializer, assignment, array, struct, field, and index checks, and the lowering/runtime fixtures again use direct typed initializers with dynamic interface function calls.

Validation:

- `cargo test -p ink-compiler --lib dynamic_interface_function -- --nocapture`
- `cargo test -p ink-compiler --lib`
- `cargo test -p ink-test interface_dynamic_function -- --nocapture`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `make gate`
