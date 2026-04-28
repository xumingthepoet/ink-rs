# Publishable Crates Need Metadata And Package Boundaries
Status: found
Found while: deep library-readiness and legacy-residue review
Scope: Cargo.toml, crates/ink-compiler/Cargo.toml, crates/ink-runtime/Cargo.toml, crates/ink-story-json-format/Cargo.toml
Problem: The facade and library crates are shaped for downstream use, but their manifests lack core package metadata, and the root facade package has no `include` boundary. `cargo package -p ink-rs --list` packages repository process files, plans, issue records, editor assets, and the upstream writing-guide snapshot.
Why it matters: Publishing or consuming the crates from a registry will produce warnings, weak crate discovery, and an unnecessarily large root crate package with internal project material that game projects do not need.
Suggested fix: Add workspace package metadata after the owner chooses license, repository, homepage, documentation, and readme values. Add package-specific descriptions where missing. Add a root `include` list that keeps only the facade source and intended user-facing docs.
Evidence: `Cargo.toml:1` has only a root package description and no license/repository/readme/include. `crates/ink-compiler/Cargo.toml:1`, `crates/ink-runtime/Cargo.toml:1`, and `crates/ink-story-json-format/Cargo.toml:1` lack descriptions and metadata. `cargo package -p ink-rs --list` warned `manifest has no license, license-file, documentation, homepage or repository` and listed `AGENTS.md`, `Notes.md`, `docs/finished_plans/*`, `docs/issues_*/*`, and `editor/vscode-ink-rs/*`.
