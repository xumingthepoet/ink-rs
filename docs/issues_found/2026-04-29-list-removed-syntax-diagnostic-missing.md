# LIST Does Not Use Removed-Feature Diagnostic
Status: found
Found while: deep library-readiness and legacy-residue review
Scope: crates/ink-compiler/src/source.rs, crates/ink-compiler/src/syntax/parser.rs, docs/WritingWithInk-updates.md
Problem: Maintained documentation says `LIST` declarations produce a removed-feature diagnostic, but the compiler does not have an active `LIST` removal check. A module-level `LIST` declaration falls through to the generic module-content diagnostic.
Why it matters: Authors get misleading migration feedback for a deliberately removed feature. It also leaves the documented language-removal contract untested.
Suggested fix: Add a source or parser diagnostic for `LIST` declarations before the module-content fallback, covering root and module scopes. Add the documented `removed_list_declaration_reports_removed_feature_diagnostic` behavior test or update the docs if the intended behavior changed.
Evidence: `docs/WritingWithInk-updates.md:314` documents the removed-feature diagnostic and `docs/WritingWithInk-updates.md:324` references a test, but `rg` only finds a parser recovery test at `crates/ink-compiler/src/syntax/parser.rs:1140`. A temporary compile of `LIST colors = (red, blue)` inside a module returned `artifact=false` with only `Module-level story content is not allowed; put story content inside a knot or stitch`.
