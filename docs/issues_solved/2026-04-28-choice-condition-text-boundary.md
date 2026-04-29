# Choice Condition And Dynamic Text Boundary Is Ambiguous

Status: solved

Found while: discussing dynamic choices generated from arrays

Scope: `crates/ink-compiler/src/syntax/choice.rs`,
`docs/SyntaxReference.md`, `docs/SyntaxUpdates.md`

Problem: Choice lines treated every leading braced expression after the choice
marker as a condition. This made `* {condition}{text_expression}` parse as
consecutive conditions rather than as a condition followed by dynamic choice
text. Authors had to use invisible glue, such as
`* {condition}<>{text_expression}`, to force the second braced expression into
choice text.

Why it matters: Dynamic choice generation from array data was harder to read
and explain. The glue workaround was semantically incidental and made the
syntax look like it depended on line-joining behavior rather than an explicit
choice condition/text boundary.

Suggested fix: Add an explicit choice condition boundary, for example
`* {condition}: {text_expression}`. Keep existing consecutive-condition syntax
working, such as `* {a} {b} Text`, and avoid making whitespace-sensitive
`* {condition}{text_expression}` carry a different meaning unless that language
break is explicitly chosen. Update parser tests and the maintained writing
guide in the same change.

Evidence: `parse_choice_conditions` in
`crates/ink-compiler/src/syntax/choice.rs` greedily consumed every leading
`{...}` expression as a choice condition. The dynamic-choice documentation in
`docs/SyntaxReference.md` explained the required `<>` workaround before
`{option.text}`.

Resolution: Choice parsing now treats a colon after one or more leading
conditions as the explicit boundary before choice text. Existing adjacent
braces without a colon remain multiple leading conditions.

Validation:

- `cargo test -p ink-compiler syntax::choice -- --nocapture`
- `cargo test -p ink-test --test language choice_condition_colon_boundary_allows_dynamic_choice_text -- --nocapture`
- `make gate`
