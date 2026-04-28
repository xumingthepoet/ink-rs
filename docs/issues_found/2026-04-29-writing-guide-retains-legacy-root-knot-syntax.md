# Writing Guide Retains Legacy Root-Knot Syntax
Status: found
Found while: deep library-readiness and legacy-residue review
Scope: docs/WritingWithInk-latest.md
Problem: The maintained writing guide states that runnable ink-rs sources must start with explicit modules and that knots inside modules use `==`, but later sections still teach upstream root-knot syntax with `=== knot ===`. The guide also contains stale once-only choice and sequence material.
Why it matters: New authors can copy examples from the current guide that do not match the current module-first language surface or current language semantics. This is especially harmful for library consumers who need reliable examples for build scripts or runtime tooling.
Suggested fix: Do a complete module-first rewrite of `docs/WritingWithInk-latest.md`: update examples that introduce knots, functions, and full runnable snippets to use explicit `=== module ... ===` plus `== knot ==`; remove or rewrite legacy sequence content after sequence removal; remove stale once-only `*` choice semantics; mark fragment-only examples clearly; and remove or rewrite the upstream TODO placeholder text. Keep `docs/WritingWithInk-updates.md` synchronized.
Evidence: `docs/WritingWithInk-latest.md:50` through `57` describe the current explicit-module source shape. `docs/WritingWithInk-latest.md:217` through `223` still introduces a knot with `=== top_knot ===`, and many later examples use the same legacy root-knot form. `docs/WritingWithInk-latest.md:130` still says `TODO: Write this section properly!`.
Owner decision: Perform a full module-first rewrite, not a narrow partial edit.
