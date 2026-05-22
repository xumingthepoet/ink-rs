=== module encounters ===
FROM enemies IMPORT ENEMY_VINE_SPRITE, ENEMY_CAVE_IMP, ENEMY_MINE_WARDEN
FROM events IMPORT set_flag, FLAG_FOREST_ENCOUNTER_CLEARED
FROM loot IMPORT resolve, LOOT_FOREST, LOOT_MINE

STRUCT EncounterDef {
    name: string
    enemy_ids: int[]
    loot_table: int
}

CONST ENCOUNTER_FOREST: int = 1
CONST ENCOUNTER_MINE: int = 2
CONST encounters: Dict<int, EncounterDef> = %{
    1: %EncounterDef{ name: "Vine Sprite ambush", enemy_ids: [enemies::ENEMY_VINE_SPRITE], loot_table: loot::LOOT_FOREST },
    2: %EncounterDef{ name: "Mine Warden patrol", enemy_ids: [enemies::ENEMY_CAVE_IMP, enemies::ENEMY_MINE_WARDEN], loot_table: loot::LOOT_MINE }
}

== forest_encounter ==
Encounter: {encounters[ENCOUNTER_FOREST].name}.
Vine Sprite lashes at the path; Ren cuts the roots loose.
~ events::set_flag(events::FLAG_FOREST_ENCOUNTER_CLEARED)
-> loot::resolve(loot::LOOT_FOREST) ->
->->

== function enemy_count(encounter_id: int) => int ==
~ return LEN(encounters[encounter_id].enemy_ids)

== function encounter_name(encounter_id: int) => string ==
~ return encounters[encounter_id].name

== function encounter_enemy_ids(encounter_id: int) => int[] ==
~ return encounters[encounter_id].enemy_ids

== function encounter_loot_table(encounter_id: int) => int ==
~ return encounters[encounter_id].loot_table
