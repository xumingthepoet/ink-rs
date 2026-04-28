# count_all_visits Emits Legacy Flags With No Runtime State
Status: found
Found while: deep library-readiness and legacy-residue review
Scope: crates/ink-compiler/src/compiler.rs, crates/ink-compiler/src/lower.rs, crates/ink-compiler/src/lower/weave.rs, crates/ink-runtime/src/story_state.rs
Problem: `CompilerOptions::count_all_visits` remains public and lowering still emits count flags when it is enabled, but the runtime no longer maintains visit counts. The option can change JSON flags without providing the behavior its name implies.
Why it matters: Library users can enable a public option that appears to request visit-count behavior but is effectively legacy metadata. This creates confusing compiled output and keeps removed visit-count semantics visible in the current API.
Suggested fix: Remove the option from the public compiler API if visit counts are intentionally gone, or rename it to an internal compatibility knob and document that it only affects legacy JSON metadata. If real sequence/state behavior needs counts, implement a maintained state model separately.
Evidence: `crates/ink-compiler/src/compiler.rs:13` exposes `count_all_visits`. `crates/ink-compiler/src/lower.rs:98` and `crates/ink-compiler/src/lower/weave.rs:41` use it to set flags. `crates/ink-runtime/src/story_state.rs:357` always returns `0` for visit counts.
