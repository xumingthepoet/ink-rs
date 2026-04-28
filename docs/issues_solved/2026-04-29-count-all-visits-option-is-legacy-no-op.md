# count_all_visits Emits Legacy Flags With No Runtime State
Status: solved
Found while: deep library-readiness and legacy-residue review
Scope: crates/ink-compiler/src/compiler.rs, crates/ink-compiler/src/lower.rs, crates/ink-compiler/src/lower/weave.rs, crates/ink-runtime/src/story_state.rs
Problem: `CompilerOptions::count_all_visits` remains public and lowering still emits count flags when it is enabled, but the runtime no longer maintains visit counts. The option can change JSON flags without providing the behavior its name implies.
Why it matters: Library users can enable a public option that appears to request visit-count behavior but is effectively legacy metadata. This creates confusing compiled output and keeps removed visit-count semantics visible in the current API.
Suggested fix: Delete `CompilerOptions::count_all_visits` from the public API and remove lowering branches that only exist to emit visit-count flags. Update tests and fixtures that compile with all-visits enabled, or convert them to removed-legacy coverage where appropriate.
Evidence: `crates/ink-compiler/src/compiler.rs:13` exposes `count_all_visits`. `crates/ink-compiler/src/lower.rs:98` and `crates/ink-compiler/src/lower/weave.rs:41` use it to set flags. `crates/ink-runtime/src/story_state.rs:357` always returns `0` for visit counts.
Owner decision: Delete this legacy option; do not keep a deprecated transition.
Resolution: Removed `CompilerOptions::count_all_visits` from the public compiler API and deleted the lowering parameter chain that emitted visit-count container flags. Snapshot tests now compile those fixtures with the default compiler path, and the affected JSON snapshots were regenerated without legacy visit-count flags.
Validation:
- `cargo fmt --all --check`
- `cargo check -p ink-compiler -p ink-test -p ink-tools`
- `cargo test -p ink-test --test compiler_snapshots`
- `cargo test -p ink-test --test compiler_api`
- `make gate TIMEOUT='f(){ shift; "$$@"; }; f'`
