# Dict Collection Helpers Needed For Natural Registries
Status: solved
Found while: reviewing `crates/ink-experiments` for workaround-shaped examples after the 100 experiment run
Scope: current ink-rs Dict builtins and registry/table patterns in `crates/ink-experiments/experiments/`
Problem: Dict V1 has typed reads and writes but no `contains`, `remove`, length, key enumeration, or iteration helpers. Many experiments therefore keep a parallel array of keys or ids solely so story code can iterate, report, or batch-process Dict entries. Registry-like examples also use boolean `active` maps instead of deleting entries.
Why it matters: Game systems commonly model abilities, quests, mail, bounties, rooms, NPCs, event handlers, and runtime registries as keyed tables. Without minimal Dict collection helpers, authors duplicate key lists and must manually keep those lists synchronized with the Dict contents. That weakens the value of Dicts and can hide missing-key and stale-entry errors inside local bookkeeping.
Fix: Added typed Dict helpers `DICT_HAS(dict, key) => bool`, `DICT_SIZE(dict) => int`, `DICT_REMOVE(dict, key) => void`, and `DICT_KEYS(dict) => K[]`. `DICT_REMOVE` requires a mutable lvalue and treats a missing key as a no-op. `DICT_KEYS` returns string keys in lexical order and int keys in ascending order.
Evidence: `job-002-cooldown-table-by-ability-id` keeps `ability_ids` next to several `Dict<int, ...>` tables; `job-004-quest-objective-progress-table` keeps `quest_ids` next to `quest_progress` and `quest_goal`; `job-023-room-template-assembly` keeps `room_template_ids` and `room_hazard_ids` next to Dict lookups; `job-024-mail-inbox-quest-hooks` keeps `incoming_mail_ids` next to Dict state; `job-029-disease-spread-quarantine` keeps `npc_ids` next to Dict state; `job-066-bounty-board-expiration` keeps `board_ids` next to Dict state; `event-handler-dict-registry` uses `active[event_id] = false` rather than removing handler entries.
Validation:
- `cargo test -p ink-story-json-format native_function`
- `cargo test -p ink-runtime native_function_call`
- `cargo test -p ink-compiler dict_collection`
- `cargo test -p ink-compiler analysis`
- `cargo test -p ink-compiler targets`
- `cargo test -p ink-compiler modules`
- `cargo test -p ink-compiler array_literals`
- `cargo test -p ink-compiler dict_literals`
- `cargo test -p ink-compiler struct_literals`
- `cargo test -p ink-test --test integration diagnostics::dict_builtin`
- `cargo test -p ink-test --test integration typed_values::dict_typed_values_run_at_runtime`
- `cargo test -p ink-test --test integration typed_values`
- `cargo test -p ink-experiments`
- `make gate`
