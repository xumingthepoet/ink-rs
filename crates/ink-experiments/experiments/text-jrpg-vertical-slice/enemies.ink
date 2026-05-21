=== interface IEnemyAi ===
== function label() => string ==
== function action_text(guarded: bool) => string ==
== function damage(guarded: bool) => int ==

=== module enemies ===
FROM vine_ai
FROM imp_ai
FROM warden_ai

STRUCT EnemyDef {
    name: string
    max_hp: int
    ai_id: int
}

CONST ENEMY_VINE_SPRITE: int = 301
CONST ENEMY_CAVE_IMP: int = 401
CONST ENEMY_MINE_WARDEN: int = 402

CONST enemy_defs: Dict<int, EnemyDef> = %{
    301: %EnemyDef{ name: "Vine Sprite", max_hp: 4, ai_id: ENEMY_VINE_SPRITE },
    401: %EnemyDef{ name: "Cave Imp", max_hp: 6, ai_id: ENEMY_CAVE_IMP },
    402: %EnemyDef{ name: "Mine Warden", max_hp: 10, ai_id: ENEMY_MINE_WARDEN }
}

VAR ai_handlers: Dict<int, interface<IEnemyAi>> = %{
    301: vine_ai,
    401: imp_ai,
    402: warden_ai
}

== function enemy_name(enemy_id: int) => string ==
~ return enemy_defs[enemy_id].name

== function enemy_max_hp(enemy_id: int) => int ==
~ return enemy_defs[enemy_id].max_hp

== function enemy_ai_label(enemy_id: int) => string ==
~ temp handler: interface<IEnemyAi> = ai_handlers[enemy_defs[enemy_id].ai_id]
~ return {handler}::label()

== function enemy_action_text(enemy_id: int, guarded: bool) => string ==
~ temp handler: interface<IEnemyAi> = ai_handlers[enemy_defs[enemy_id].ai_id]
~ return {handler}::action_text(guarded)

== function enemy_damage(enemy_id: int, guarded: bool) => int ==
~ temp handler: interface<IEnemyAi> = ai_handlers[enemy_defs[enemy_id].ai_id]
~ return {handler}::damage(guarded)

=== module vine_ai implements IEnemyAi ===
== function label() => string ==
~ return "snare"

== function action_text(guarded: bool) => string ==
{ if guarded:
    ~ return "Vine Sprite lashes at Ren's guard"
- else:
    ~ return "Vine Sprite lashes at the path"
}

== function damage(guarded: bool) => int ==
{ if guarded:
    ~ return 1
- else:
    ~ return 2
}

=== module imp_ai implements IEnemyAi ===
== function label() => string ==
~ return "skirmish"

== function action_text(guarded: bool) => string ==
{ if guarded:
    ~ return "Cave Imp claws at Ren's shield"
- else:
    ~ return "Cave Imp claws at Lio"
}

== function damage(guarded: bool) => int ==
{ if guarded:
    ~ return 1
- else:
    ~ return 2
}

=== module warden_ai implements IEnemyAi ===
== function label() => string ==
~ return "guard break"

== function action_text(guarded: bool) => string ==
{ if guarded:
    ~ return "Mine Warden attacks into Ren's guard"
- else:
    ~ return "Mine Warden strikes Lio"
}

== function damage(guarded: bool) => int ==
{ if guarded:
    ~ return 1
- else:
    ~ return 3
}
