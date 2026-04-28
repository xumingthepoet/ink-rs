# Remove Path-Based Tag Lookup API
Status: solved
Found while: deep library-readiness and legacy-residue review
Scope: crates/ink-runtime/src/story/tags.rs, crates/ink-runtime/src/container.rs, crates/ink-runtime/src/search_result.rs
Problem: `Story::tags_for_content_at_path` asks `content_at_path` for a path and immediately reads the returned container without checking whether the `SearchResult` is approximate. Missing paths can therefore return tags from the nearest resolved container, commonly an empty tag list, instead of reporting an invalid path.
Why it matters: This public API can silently hide bad content-path strings in host integrations, and path-based tag lookup is not needed for the current host API surface. Current-line tags remain important and must stay supported.
Suggested fix: Delete `Story::tags_for_content_at_path` and related tests/docs for arbitrary path-based tag lookup instead of repairing its missing-path behavior. Keep `Story::get_current_tags()` as an important runtime API, and preserve choice tags on generated choices.
Evidence: `crates/ink-runtime/src/story/tags.rs:34` calls `self.content_at_path(&path).container().unwrap()` and does not inspect approximation. `crates/ink-runtime/src/container.rs:203` sets `approximate = true` when a path component cannot be resolved, and `crates/ink-runtime/src/search_result.rs:23` exposes this via `correct_obj`. A temporary runner called `tags_for_content_at_path("game.missing")` and printed `missing-tags-result=Ok([])`.
Owner decision: Remove the path-based tag lookup API. Keep `story.get_current_tags()`; it is an important feature.
Resolution: Removed the public `Story::tags_for_content_at_path` API and the test support wrapper/assertions for arbitrary path-based tag lookup. Kept `Story::get_current_tags()`, global tags, and choice tags supported.
Validation:
- `cargo fmt --all --check`
- `cargo check -p ink-runtime -p ink-test`
- `cargo test -p ink-test --test tags`
- `cargo test -p ink-runtime`
- `make gate TIMEOUT='f(){ shift; "$$@"; }; f'`
