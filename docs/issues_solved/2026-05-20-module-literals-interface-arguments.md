# Module Literals Are Not Accepted As Interface-Typed Flow Arguments

Status: solved

Found while: Expanding the `event-handler-dict-registry` experiment with a registration API.

Scope: `crates/ink-compiler` analysis for interface values and flow call arguments; `crates/ink-experiments/experiments/event-handler-dict-registry/story.ink`

Problem: A module literal works as an `interface<IEventHandler>` value in typed variable initializers and Dict literals, but the natural registration call `-> register(10, heal_event, false) ->` fails when `register` declares `handler: interface<IEventHandler>`. The compiler reports `Unresolved variable: heal_event` instead of treating `heal_event` as a module literal in the expected interface-typed argument position.

Why it matters: Game-style registries need APIs such as `register(event_id, handler)` where `handler` is an interface-typed module value. Without module literals in interface-typed flow arguments, authors must introduce extra local variables or prebuilt Dict literals, which makes dynamic registration awkward and less reusable.

Suggested fix: Extend argument type checking/lowering so expected interface types are propagated into knot/function call arguments. When an argument expression is a module name and the expected type is `interface<I>`, resolve it as a module literal if the current module imports it and the target module explicitly implements `I`.

Evidence: `cargo test -p ink-experiments` failed for `event-handler-dict-registry/story.ink` with unresolved variables on calls such as `-> register(10, heal_event, false) ->`, despite `FROM heal_event` and `=== module heal_event implements IEventHandler ===` being present.

Validation: `cargo test -p ink-compiler interface_module`; `cargo test -p ink-compiler bare_module_imports`; `cargo test -p ink-test --test integration typed_values`; `cargo test -p ink-experiments`.
