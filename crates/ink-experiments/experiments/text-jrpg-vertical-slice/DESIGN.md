# Text JRPG Vertical Slice Design

## Goal

Build a multi-file JRPG experiment that exercises cross-system state in ink-rs:
location changes, main and side quests, party join/leave, inventory, equipment,
character growth, shop flow, a three-slot ink-level save system, encounters,
loot tables, battle, status effects, rewards, dialogue, event flags, and a gate
puzzle.

This is a vertical slice, not a complete RPG. The finished experiment should
play through one compact chapter from village setup to dungeon clear and return.

## Language Surfaces To Explore

- Multi-file modules with imports between world, party, quests, items,
  equipment, shop, save, encounters, loot, battle, puzzle, dialogue, and event
  systems.
- `STRUCT` declarations with nested structs, struct arrays, and struct values in
  `Dict<int, T>` tables.
- `ENUM` state machines for quest states, location ids, battle phases, target
  kinds, status kinds, equipment slots, and event kinds.
- `Dict<int, Struct>` for static data tables: locations, NPCs, allies, enemies,
  skills, items, equipment, shop stocks, encounter tables, loot tables, quests,
  objectives, and puzzle nodes.
- `Dict<int, int>` for mutable counters: inventory, equipment by slot, quest
  objective progress, event flags, status duration, save snapshot fields,
  encounter progress, and enemy HP by battle slot.
- `interface<T>` values for behavior dispatch where it is natural, especially
  skill effects, enemy AI, and event handlers.
- `for` loops over arrays and Dicts for rendering lists, scanning party members,
  applying status ticks, resolving rewards, and checking quest objectives.
- Dynamic choice helper patterns for data-driven menus: travel destinations,
  dialogue topics, battle skills, battle targets, item use, shop inventory, save
  point actions, encounter choices, and puzzle choices.

## Planned File Layout

```text
text-jrpg-vertical-slice/
  DESIGN.md
  story.ink
  world.ink
  events.ink
  quests.ink
  party.ink
  items.ink
  equipment.ink
  shop.ink
  save.ink
  encounters.ink
  loot.ink
  skills.ink
  battle.ink
  dialogue.ink
  puzzle.ink
  story.ink.playthrough.json
```

`story.ink` owns the top-level route and imports every subsystem. It should keep
the playable path readable and delegate subsystem mechanics to the other files.

## Chapter Route

The chapter should be deterministic so that `story.ink.playthrough.json` can pin
the full interaction.

1. Start in Brindle Village with one hero, low gold, and no key item.
2. Talk to Elder Mara to start the main quest: find the missing caravan in the
   old mine.
3. Talk to Apothecary Senn to start a side quest: gather moonleaf in the forest.
4. Visit the village shop, buy one potion, fail one unaffordable purchase, and
   render shop stock after inventory changes.
5. Use the shrine save point to write Slot 1, mutate state, then load Slot 1
   back through the save-point menu.
6. Recruit Guard Ren after accepting the main quest.
7. Travel from village to forest.
8. Trigger a deterministic forest encounter from an encounter table, win it, and
   resolve a loot table drop.
9. Gather moonleaf, update the side quest objective, and receive an antidote.
10. Trigger a short story scene where Ren leaves to scout the mine gate, proving
    temporary party removal before he rejoins.
11. Travel to mine gate.
12. Use the mine gate camp save point to write Slot 2.
13. Solve a three-rune gate puzzle. Puzzle success sets a world flag and opens
    the mine.
14. Enter the mine and use the lift camp save point to write Slot 3.
15. Fight a scripted battle against two enemies.
16. Resolve battle rewards: EXP, gold, item drop, and one new skill if a level
    threshold is reached.
17. Trigger rescue dialogue and party banter.
18. Return to village, complete the main quest and side quest, render all three
    save slots, then end.

The playthrough should cover at least one incorrect puzzle input, one shop
purchase, one failed shop purchase, one save action, one deterministic encounter,
one loot table resolution, one item use or inventory check, one skill use, one
enemy AI turn, one status tick, one party join, one story leave/rejoin, and one
quest objective update.

## Subsystem Design

### World And Locations

Locations are static `LocationDef` structs in `Dict<int, LocationDef>`. Mutable
world state should be held in flags and quest state, not by mutating location
defs.

Required locations:

- Brindle Village: hub, NPC dialogue, quest turn-in, shop access.
- Moonlit Forest: side quest collection and optional encounter text.
- Old Mine Gate: puzzle gate before dungeon entry.
- Old Mine: battle and caravan rescue.

Travel should be data-driven where possible, but choices may use a dynamic choice
thread helper if ordinary `for` blocks cannot naturally produce choices.

### Events And Flags

Use an event/flag registry as the glue between systems.

Minimum flags:

- main quest accepted
- side quest accepted
- Ren joined
- Ren scouting
- Ren rejoined
- moonleaf gathered
- village save written
- forest encounter cleared
- gate opened
- mine battle won
- mine loot resolved
- caravan rescued
- main quest complete
- side quest complete

Represent flags as `Dict<int, bool>` or `Dict<int, int>` and expose helper
functions such as `set_flag`, `has_flag`, and `flag_label`. Event handlers may
use `interface<IEventHandler>` if it stays readable.

### Quests And Objectives

Use `QuestDef`, `ObjectiveDef`, and mutable objective progress counters.

Minimum quests:

- Main: Missing Caravan
  - accept quest
  - recruit Ren
  - open mine gate
  - win mine battle
  - rescue caravan
  - return to elder
- Side: Moonleaf Remedy
  - accept quest
  - gather moonleaf
  - return to apothecary

Quest rendering should iterate objective tables and expose the awkwardness of
conditional objective visibility if current syntax makes it clumsy.

### Party And Growth

Use static ally definitions and mutable runtime stats.

Actors:

- Hero: always present, balanced physical skill.
- Ren: joins after main quest accept, guard role, temporarily leaves to scout
  the mine gate, then rejoins before the gate puzzle.
- Optional NPC-only healer can be represented by dialogue, not active combat.

Track level, EXP, HP, MP, base stats, learned skills, and active party ids. The
reward system should grant EXP and call a level-up helper that can learn one
skill at a threshold.

Party join and leave must update the same active party list used by dialogue,
inventory targeting, and battle initialization. Avoid a separate story-only
party flag that can drift from battle state.

### Items And Inventory

Use `Dict<int, ItemDef>` for item data and `Dict<int, int>` for inventory counts.

Minimum items:

- Potion: heals one ally in battle or out of battle.
- Antidote: clears poison.
- Moonleaf: quest item.
- Mine Charm: reward or key item.

Inventory rendering should use Dict iteration and count lookup. Item use should
exercise target selection and inventory mutation.

### Shop And Economy

Use `ShopDef` and `ShopStock` tables keyed by shop id and item id. The village
shop is enough for v1.

Required behavior:

- Render stock with item names, prices, and current inventory counts.
- Buy one affordable potion and decrement gold.
- Attempt one unaffordable purchase and preserve state.
- Optionally unlock one extra stock item after the forest encounter.

Shop stock should be data-driven, not a hardcoded list of text branches. If
dynamic stock choices become too noisy, keep the awkward source shape visible.

### Equipment

Keep v1 equipment small but real.

Slots:

- weapon
- armor
- charm

Use `EquipmentDef` with stat bonuses and slot enum. Active equipment can be
tracked as `Dict<int, int>` keyed by actor id plus slot-specific helper, or as a
small `Loadout` struct per actor. Prefer whichever form exposes fewer
workarounds in current ink.

The playthrough only needs one equipment grant or display, not a full shop UI.

### Save Slots

Use three save slots owned by `save.ink`. This is an ink-level checkpoint system,
not runtime save-state JSON. Loading is only exposed through save-point menus.

Minimum saved fields:

- save point id and location id
- gold
- main quest state
- side quest state
- active party ids
- actor HP, MP, EXP, level, and equipment ids
- inventory counts for potion, antidote, moonleaf, and mine charm
- puzzle input, failure count, and solved state
- important flags such as gate opened and forest encounter cleared

The playthrough writes Slot 1 at the village shrine, deliberately mutates gold
and HP, then loads Slot 1 to prove restoration. It later writes Slot 2 at the
mine gate camp and Slot 3 at the mine lift camp. Loading a slot restores the
active system variables and then routes through a save-point id switch so the
story re-enters the location represented by that snapshot.

Represent each snapshot as a `SaveSlot` struct stored in
`Dict<int, SaveSlot>`. If that shape exposes an issue, stop and record it.

### Skills And Battle Effects

Use `SkillDef` static data and behavior dispatch.

Minimum skills:

- Strike: physical single-target damage.
- Guard: self buff or damage reduction.
- Spark: magic damage with MP cost.
- First Aid: heal an ally.
- Venom Slash or Smoke Bomb: applies poison or blind if status support is
  readable.

Preferred exploration path: `Dict<int, interface<ISkillEffect>>` maps skill ids
to modules that implement the effect. If this becomes noisy or exposes a real
bug, stop and record the issue rather than replacing it with a switch just to
pass.

### Battle System

One scripted battle is enough, but it should be a real turn loop.

Required battle behavior:

- Initialize enemies from static enemy defs and battle slot ids.
- Render party and enemy status.
- Player chooses an active ally action.
- Action choices include at least skills and items.
- Target choice depends on skill target kind.
- Enemy AI chooses an action from an AI behavior table or module.
- Status effects tick at turn start or turn end.
- Victory grants EXP, gold, and an item drop.
- Defeat offers restart.

Enemies:

- Cave Imp: low HP, attacks weakest ally.
- Mine Warden: higher HP, can poison or guard.

### Encounters And Loot

Use deterministic encounter tables rather than random generation. The goal is to
exercise encounter lookup and resolution, not RNG.

Encounter tables:

- Forest table: one encounter id for a Vine Sprite.
- Mine table: one encounter id for Cave Imp plus Mine Warden.

Encounter state should track whether a one-shot encounter has been cleared.
Starting an encounter should pass enemy ids into battle initialization, not make
the battle system know about locations directly.

Loot tables should be separate from battle rewards:

- Forest encounter drops one moonleaf through a loot table.
- Mine battle drops a Mine Charm and gold through a loot table.

Use `LootEntry` structs with item id, quantity, and deterministic threshold or
roll slot. The playthrough should show one resolved table so that table-driven
reward code is pinned.

### Dialogue And Banter

Dialogue should be short but stateful.

Required scenes:

- Elder quest start.
- Apothecary side quest.
- Ren recruitment.
- Ren temporary scouting leave and rejoin.
- Shop purchase feedback.
- Save shrine feedback.
- Forest encounter intro and loot result.
- Puzzle feedback.
- Battle victory banter.
- Caravan rescue.
- Return and quest completion.

Use party state and flags to gate lines. Keep exact prose concise so the
experiment remains about language mechanics rather than content volume.

### Puzzle Gate

Use a three-rune lock at the mine gate.

Puzzle state:

- current input sequence
- expected sequence
- failed attempt count
- solved flag

The playthrough should include one wrong input and then the correct sequence.
This tests array mutation, sequence comparison, reset behavior, and world gate
updates.

## Dynamic Choice Strategy

Use the existing dynamic choice thread pattern from `dynamic-battle-choice-list`
when menus are data-driven. Prefer creating a small reusable choice helper inside
this experiment rather than hand-writing every list, but do not hide awkwardness
by over-abstracting the experiment into unreadable code.

Menus that should be data-driven:

- travel destinations
- active party member action selection
- skill list
- target list
- item list
- shop stock list
- save point action list
- encounter continuation choices
- puzzle rune list

If current syntax cannot express one of these naturally, leave the direct source
shape that exposes the limitation and note it in the experiment recap.

## Implemented Follow-Up Improvements

The current implementation goes beyond the first vertical slice in a few places:

- `enemies.ink` defines static enemy data and an `IEnemyAi` interface. Enemy AI
  handlers are stored in a `Dict<int, interface<IEnemyAi>>`, so the JRPG slice
  now uses module values outside the skill system.
- `encounters.ink` exposes encounter name, enemy ids, and loot table helpers.
  `battle.ink` initializes enemy slots and HP from the mine encounter instead
  of hardcoding the battle roster.
- `shop.ink` now honors `ShopStock.unlock_flag`. The playthrough covers both an
  unaffordable purchase and a locked stock attempt.
- `save.ink` stores three raw snapshot slots for location, inventory counts,
  party stats, equipment ids, quest state, objective progress, puzzle state, and
  event flags, then restores Slot 1 during the playthrough.
- Save-point flow is pure ink: save menus are dynamic choices over slot ids,
  `save.ink` guards write/load behind an opened save point, and `story.ink`
  dispatches after load using the saved save-point id.
- Numeric ids are named at use sites with constants where current syntax allows
  it. Dict literal keys remain numeric because the current Dict literal grammar
  accepts literal string/int keys, while imported constants must be referenced
  through qualified names in value expressions.

## Validation Target

The first implementation should add a deterministic `story.ink.playthrough.json`
that verifies:

- quest acceptance and objective update text
- Ren joins the party
- Ren leaves for scouting and rejoins
- one village shop purchase and one failed purchase
- three save slot writes, one slot load, and final all-slot summary
- one deterministic encounter selected from a table
- one loot table resolution
- inventory gain and item use
- puzzle failure then success
- battle initialization, at least one player skill, one item or heal, one enemy
  turn, one status tick, and victory
- reward application and quest completion
- final story ending

Run:

```text
cargo test -p ink-experiments
git diff --check
```

Do not commit a failing experiment unless it is intentionally copied to a main
worktree with an issue file to reproduce a newly found bug.

## Known Risks To Expose, Not Hide

- Dynamic choices may remain too verbose for common JRPG menus.
- Cross-file imports may make type names or module-qualified references noisy.
- `Dict<int, Struct>` and `Dict<int, interface<T>>` may reveal lowering/runtime
  shape issues when used across modules.
- Updating nested state like `party_stats[actor_id].hp` may be awkward or
  unsupported depending on the chosen representation.
- Lack of string formatting helpers may make battle logs noisy.
- Lack of loop `break` may force full scans for target and winner checks.
