# Remove Variable Observer API
Status: found
Found while: deep library-readiness and legacy-residue review
Scope: crates/ink-runtime/src/story/variable_observer.rs
Problem: `Story::remove_variable_observer` returns `Result<(), StoryError>` but uses `unwrap()` after searching for the observer. Removing an observer that is not registered for the specified variable panics. The `None` branch can also panic when the observer is absent from any observed variable list it iterates.
Why it matters: This is a public runtime API footgun for host games, and the owner confirmed variable observing is not needed in the current host API surface.
Suggested fix: Delete the variable observer feature instead of patching the panic. Remove the `VariableObserver` trait, `Story::observe_variable`, `Story::remove_variable_observer`, `Story::notify_variable_changed`, the `variable_observers` field, and runtime API tests/docs that present observer support. Keep direct `get_variable`/`set_variable` support.
Evidence: `crates/ink-runtime/src/story/variable_observer.rs:68` and `crates/ink-runtime/src/story/variable_observer.rs:81` unwrap the result of `position`. A temporary runner registered one observer for `game::observed`, then removed a different observer for that variable, and printed `remove-unregistered-panicked=true` after the panic at line `68`.
Owner decision: Remove variable observing from the runtime API; this feature is not currently needed.
