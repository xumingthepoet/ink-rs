=== module game ===
STRUCT EnemyStatic {
    name: string
    max_hp: int
    threat: int
    intro: string
}

VAR enemy_defs: Dict<int, EnemyStatic> = %{
    101: %EnemyStatic{ name: "Slime", max_hp: 12, threat: 1, intro: "splits out of the grass" },
    204: %EnemyStatic{ name: "Ash Bat", max_hp: 9, threat: 2, intro: "drops from the rafters" },
    305: %EnemyStatic{ name: "Iron Warden", max_hp: 30, threat: 5, intro: "locks the gate" }
}

VAR current_hp: Dict<int, int> = %{}

== main ==
-> start_battle([101, 204, 305])

== start_battle(enemy_ids: int[]) ==
Battle begins with {LEN(enemy_ids)} enemies.
-> initialize_enemies(enemy_ids, 0)

== initialize_enemies(enemy_ids: int[], index: int) ==
{ if index >= LEN(enemy_ids):
    -> enemy_summary(enemy_ids, 0)
- else:
    ~ temp enemy_id: int = enemy_ids[index]
    ~ temp def: EnemyStatic = enemy_defs[enemy_id]
    ~ current_hp[enemy_id] = def.max_hp
    Spawn id {enemy_id} {def.name}: {def.intro}.
    -> initialize_enemies(enemy_ids, index + 1)
}

== enemy_summary(enemy_ids: int[], index: int) ==
{ if index >= LEN(enemy_ids):
    -> END
- else:
    ~ temp enemy_id: int = enemy_ids[index]
    ~ temp def: EnemyStatic = enemy_defs[enemy_id]
    Ready id {enemy_id}: {def.name} hp={current_hp[enemy_id]} threat={def.threat}
    -> enemy_summary(enemy_ids, index + 1)
}
