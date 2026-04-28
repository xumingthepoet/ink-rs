# Choice Condition And Dynamic Text Boundary Is Ambiguous

Status: found

Found while: discussing dynamic choices generated from arrays

Scope: `crates/ink-compiler/src/syntax/choice.rs`,
`docs/WritingWithInk-latest.md`, `docs/WritingWithInk-updates.md`

Problem: Choice lines currently treat every leading braced expression after the
choice marker as a condition. This makes `* {condition}{text_expression}` parse
as consecutive conditions rather than as a condition followed by dynamic choice
text. Authors must use invisible glue, such as
`* {condition}<>{text_expression}`, to force the second braced expression into
choice text.

Why it matters: Dynamic choice generation from array data becomes harder to
read and explain. The glue workaround is semantically incidental and makes the
syntax look like it depends on line-joining behavior rather than an explicit
choice condition/text boundary.

Suggested fix: Add an explicit choice condition boundary, for example
`* {condition}: {text_expression}`. Keep existing consecutive-condition syntax
working, such as `* {a} {b} Text`, and avoid making whitespace-sensitive
`* {condition}{text_expression}` carry a different meaning unless that language
break is explicitly chosen. Update parser tests and the maintained writing
guide in the same change.

Evidence: `parse_choice_conditions` in
`crates/ink-compiler/src/syntax/choice.rs` greedily consumes every leading
`{...}` expression as a choice condition. The current dynamic-choice
documentation in `docs/WritingWithInk-latest.md` explains the required `<>`
workaround before `{option.text}`.
