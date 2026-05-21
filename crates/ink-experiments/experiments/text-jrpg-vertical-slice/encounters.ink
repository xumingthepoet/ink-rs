=== module encounters ===
FROM events IMPORT set_flag, FLAG_FOREST_ENCOUNTER_CLEARED
FROM loot IMPORT resolve, LOOT_FOREST

STRUCT EncounterDef {
    name: string
    enemy_ids: int[]
    loot_table: int
}

CONST ENCOUNTER_FOREST: int = 1
CONST ENCOUNTER_MINE: int = 2
CONST encounters: Dict<int, EncounterDef> = %{
    1: %EncounterDef{ name: "Vine Sprite ambush", enemy_ids: [301], loot_table: 1 },
    2: %EncounterDef{ name: "Mine Warden patrol", enemy_ids: [401, 402], loot_table: 2 }
}

== forest_encounter ==
Encounter: {encounters[ENCOUNTER_FOREST].name}.
Vine Sprite lashes at the path; Ren cuts the roots loose.
~ events::set_flag(events::FLAG_FOREST_ENCOUNTER_CLEARED)
-> loot::resolve(loot::LOOT_FOREST) ->
->->

== function enemy_count(encounter_id: int) => int ==
~ return LEN(encounters[encounter_id].enemy_ids)
