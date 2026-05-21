# CONST Dict Struct Runtime DICT_HAS Failure
Status: solved
Found while: adding the text roguelike room crawl experiment
Scope: crates/ink-experiments/experiments/text-roguelike-room-crawl/story.ink; runtime handling for typed `CONST Dict<int, RoomDef>`

Problem: A typed `CONST rooms: Dict<int, RoomDef>` compiles successfully, but runtime execution fails when the story calls `DICT_HAS(rooms, index)`. The runtime error says `DICT_HAS expected a dict value as its first parameter`.

Why it matters: `docs/SyntaxReference.md` says constants can use maintained value types including `Dict<string, int>`, and experiments need static table data to be usable from runtime logic. If a constant Dict can compile but reaches runtime as a non-Dict value, authors will get delayed runtime failures instead of either working behavior or a clear compiler diagnostic.

Suggested fix: Add a focused fixture that calls `DICT_HAS` and indexes a typed `CONST Dict<int, StructValue>`, then fix constant lowering/runtime value materialization so compiled constants preserve their Dict runtime shape. If constants intentionally cannot support this shape, reject the source with an actionable diagnostic and update the syntax docs.

Evidence: `cargo test -p ink-experiments` fails in `playthrough_snapshots_match_when_present` for `text-roguelike-room-crawl/story.ink` with `RUNTIME ERROR: (game.map_char.13.b.3): DICT_HAS expected a dict value as its first parameter`. The failing source uses `CONST rooms: Dict<int, RoomDef> = %{ ... }` and calls `DICT_HAS(rooms, index)` in `map_char` and `enter_room`.

Fix: Constant references now lower through the expected-type composite literal
path, so typed const arrays, structs, and Dicts can use dynamic value
materialization when static value lowering cannot represent a nested value such
as an enum member in a struct field.

Validation:
- `cargo test -p ink-test --test integration typed_values::const_dicts_with_struct_values_run`
- `cargo test -p ink-experiments`
- `make gate`
