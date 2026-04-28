# tags_for_content_at_path Ignores Unresolved Paths
Status: found
Found while: deep library-readiness and legacy-residue review
Scope: crates/ink-runtime/src/story/tags.rs, crates/ink-runtime/src/container.rs, crates/ink-runtime/src/search_result.rs
Problem: `Story::tags_for_content_at_path` asks `content_at_path` for a path and immediately reads the returned container without checking whether the `SearchResult` is approximate. Missing paths can therefore return tags from the nearest resolved container, commonly an empty tag list, instead of reporting an invalid path.
Why it matters: This public API can silently hide bad content-path strings in host integrations. A typo in a path can be interpreted as "this content has no tags" rather than as an integration error.
Suggested fix: Check `SearchResult::correct_obj()` or the `approximate` flag before reading the container, and return `StoryError::BadArgument` or `InvalidStoryState` for unresolved paths. Add tests for missing knot and missing stitch paths.
Evidence: `crates/ink-runtime/src/story/tags.rs:34` calls `self.content_at_path(&path).container().unwrap()` and does not inspect approximation. `crates/ink-runtime/src/container.rs:203` sets `approximate = true` when a path component cannot be resolved, and `crates/ink-runtime/src/search_result.rs:23` exposes this via `correct_obj`. A temporary runner called `tags_for_content_at_path("game.missing")` and printed `missing-tags-result=Ok([])`.
