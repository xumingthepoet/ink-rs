# LIST Does Not Use Removed-Feature Diagnostic
Status: solved
Found while: deep library-readiness and legacy-residue review
Scope: crates/ink-compiler/src/source.rs, crates/ink-compiler/src/syntax/parser.rs, docs/WritingWithInk-updates.md
Problem: Maintained documentation says `LIST` declarations produce a removed-feature diagnostic, but the compiler does not have an active `LIST` removal check. A module-level `LIST` declaration falls through to the generic module-content diagnostic.
Why it matters: Authors get misleading migration feedback for a deliberately removed feature. It also leaves the documented language-removal contract untested.
Suggested fix: Add a source or parser diagnostic for `LIST` declarations before the module-content fallback, covering root and module scopes. Add the documented `removed_list_declaration_reports_removed_feature_diagnostic` behavior test or update the docs if the intended behavior changed.
Evidence: `docs/WritingWithInk-updates.md:314` documents the removed-feature diagnostic and `docs/WritingWithInk-updates.md:324` references a test, but `rg` only finds a parser recovery test at `crates/ink-compiler/src/syntax/parser.rs:1140`. A temporary compile of `LIST colors = (red, blue)` inside a module returned `artifact=false` with only `Module-level story content is not allowed; put story content inside a knot or stitch`.
Resolution: Added source-stage detection for `LIST` declarations so root-level sources and explicit modules emit a removed-feature diagnostic before generic module or module-content fallback errors. Added compiler unit coverage for root and module scopes, replaced a parser recovery test that used legacy `LIST` input, and added a behavior fixture/test for module-scoped removed `LIST` declarations.
Validation:
- `cargo fmt --all --check`
- `cargo test -p ink-compiler list_declaration_is_removed`
- `cargo test -p ink-compiler invalid_logic_expression_reports_specific_error_and_recovers_next_line`
- `cargo test -p ink-test --test diagnostics removed_list_declaration_reports_removed_feature_diagnostic`
- `cargo test -p ink-test --test integration_policy`
- `make gate TIMEOUT='f(){ shift; "$$@"; }; f'`
