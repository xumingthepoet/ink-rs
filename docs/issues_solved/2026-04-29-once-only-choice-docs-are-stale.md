# Writing Guide Still Documents Once-Only Star Choices
Status: solved
Found while: deep library-readiness and legacy-residue review
Scope: docs/SyntaxReference.md, docs/SyntaxUpdates.md, crates/ink-test/tests/choices.rs
Problem: The maintained writing guide still contains the upstream "Choices can only be used once" section and examples where `*` choices disappear. Later in the same guide and in the change log, ink-rs says both `*` and `+` choices are repeatable.
Why it matters: Users learning ink-rs from the current guide will write against removed upstream choice semantics and may add unnecessary fallback logic or misdiagnose repeatable choices as a runtime bug.
Suggested fix: Rewrite the once-only choice section to describe ink-rs repeatable choices and explicit variable-based disappearance. Keep the existing repeatable-choice section, or merge the sections so the guide has one consistent rule.
Evidence: `docs/SyntaxReference.md:499` says "Choices can only be used once" and line `501` says every choice can only be chosen once by default. `docs/SyntaxReference.md:581` says both `*` and `+` are repeatable. `docs/SyntaxUpdates.md:179` records repeatable `*`/`+`, and `crates/ink-test/tests/choices.rs:185` tests `star_and_plus_choices_are_repeatable`.
Resolution: Rewrote the maintained writing guide around repeatable choices and explicit variable-based hiding, removed compiler/runtime once-only choice flag state, and renamed the legacy sticky-choice fixture/test to plus-choice repeatability wording.
Validation: `cargo test -p ink-compiler`; `cargo test -p ink-runtime`; `cargo test -p ink-test --test choices`; `cargo test -p ink-test --test integration_policy`; `make gate TIMEOUT='f(){ shift; "$$@"; }; f'`.
