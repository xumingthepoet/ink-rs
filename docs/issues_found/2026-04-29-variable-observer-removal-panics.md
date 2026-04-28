# Removing A Non-Matching Variable Observer Panics
Status: found
Found while: deep library-readiness and legacy-residue review
Scope: crates/ink-runtime/src/story/variable_observer.rs
Problem: `Story::remove_variable_observer` returns `Result<(), StoryError>` but uses `unwrap()` after searching for the observer. Removing an observer that is not registered for the specified variable panics. The `None` branch can also panic when the observer is absent from any observed variable list it iterates.
Why it matters: This is a public runtime API footgun for host games. A repeated removal, wrong variable name, or cleanup path for a never-registered observer can abort the host instead of returning an error or no-op result.
Suggested fix: Replace the `unwrap()` calls with explicit handling. Either treat absent observers as `Ok(())`, or return `StoryError::BadArgument` with a clear message. Add runtime API tests for wrong-variable removal, repeated removal, and remove-from-all with mixed observer lists.
Evidence: `crates/ink-runtime/src/story/variable_observer.rs:68` and `crates/ink-runtime/src/story/variable_observer.rs:81` unwrap the result of `position`. A temporary runner registered one observer for `game::observed`, then removed a different observer for that variable, and printed `remove-unregistered-panicked=true` after the panic at line `68`.
