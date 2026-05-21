=== module battle ===
FROM encounters IMPORT encounter_name, encounter_enemy_ids, encounter_loot_table, ENCOUNTER_MINE
FROM enemies IMPORT enemy_name, enemy_max_hp, enemy_action_text, enemy_damage
FROM party IMPORT party_summary, damage_actor, heal_actor, spend_mp, add_exp, ACTOR_HERO, ACTOR_REN
FROM items IMPORT remove_item, ITEM_POTION
FROM skills IMPORT use_skill, SKILL_SPARK, SKILL_GUARD, SKILL_STRIKE
FROM loot IMPORT resolve
FROM events IMPORT set_flag, FLAG_MINE_BATTLE_WON, FLAG_MINE_LOOT_RESOLVED

CONST ENEMY_IMP_SLOT: int = 1
CONST ENEMY_WARDEN_SLOT: int = 2

VAR active_enemy_slots: int[] = []
VAR enemy_ids_by_slot: Dict<int, int> = %{}
VAR enemy_hp: Dict<int, int> = %{}
VAR poison_turns: int = 0
VAR guarded: bool = false
VAR turn: int = 1

== mine_battle ==
-> start_encounter(encounters::ENCOUNTER_MINE) ->
~ poison_turns = 0
~ guarded = false
~ turn = 1
-> battle_prompt

== start_encounter(encounter_id: int) ==
~ active_enemy_slots = []
~ enemy_ids_by_slot = %{}
~ enemy_hp = %{}
~ temp enemy_ids: int[] = encounters::encounter_enemy_ids(encounter_id)
{ for index, enemy_id in enemy_ids:
    ~ temp slot: int = index + 1
    ~ ARRAY_PUSH(active_enemy_slots, slot)
    ~ enemy_ids_by_slot[slot] = enemy_id
    ~ enemy_hp[slot] = enemies::enemy_max_hp(enemy_id)
}
Mine battle begins: {encounters::encounter_name(encounter_id)}.
->->

== battle_prompt ==
Turn {to_str(turn)}
Party: {party::party_summary()}
Enemies: {enemy_summary()}
Poison turns: {to_str(poison_turns)}
* Hero Spark
    -> hero_spark
* Use Potion
    -> use_potion
* Ren Guard
    -> ren_guard
* Hero Strike
    -> hero_strike

== hero_spark ==
-> skills::use_skill(skills::SKILL_SPARK, party::ACTOR_HERO, ENEMY_WARDEN_SLOT) ->
{ if party::spend_mp(party::ACTOR_HERO, 2):
    ~ enemy_hp[ENEMY_WARDEN_SLOT] = enemy_hp[ENEMY_WARDEN_SLOT] - 4
    ~ poison_turns = 2
    Lio spends 2 MP; the warden is poisoned.
- else:
    Lio lacks MP.
}
-> enemy_turn ->
-> status_tick ->
~ turn += 1
-> battle_prompt

== use_potion ==
{ if items::remove_item(items::ITEM_POTION, 1):
    ~ party::heal_actor(party::ACTOR_HERO, 5)
    Lio drinks a potion.
- else:
    No potion remains.
}
-> enemy_turn ->
-> status_tick ->
~ turn += 1
-> battle_prompt

== ren_guard ==
-> skills::use_skill(skills::SKILL_GUARD, party::ACTOR_REN, party::ACTOR_REN) ->
~ guarded = true
Ren guards the line.
-> enemy_turn ->
-> status_tick ->
~ turn += 1
-> battle_prompt

== hero_strike ==
-> skills::use_skill(skills::SKILL_STRIKE, party::ACTOR_HERO, ENEMY_IMP_SLOT) ->
{ for slot in active_enemy_slots:
    ~ enemy_hp[slot] = 0
}
Lio and Ren finish the patrol.
~ events::set_flag(events::FLAG_MINE_BATTLE_WON)
Victory: mine patrol defeated.
~ temp level_text: string = party::add_exp(5)
EXP result: {level_text}.
-> loot::resolve(encounters::encounter_loot_table(encounters::ENCOUNTER_MINE)) ->
~ events::set_flag(events::FLAG_MINE_LOOT_RESOLVED)
->->

== enemy_turn ==
~ temp attacker_id: int = enemy_ids_by_slot[ENEMY_WARDEN_SLOT]
{enemies::enemy_action_text(attacker_id, guarded)} for {to_str(enemies::enemy_damage(attacker_id, guarded))} damage.
~ party::damage_actor(party::ACTOR_HERO, enemies::enemy_damage(attacker_id, guarded))
{ if guarded:
    ~ guarded = false
}
->->

== status_tick ==
{ if poison_turns > 0:
    ~ enemy_hp[ENEMY_WARDEN_SLOT] = enemy_hp[ENEMY_WARDEN_SLOT] - 2
    ~ poison_turns -= 1
    Poison ticks on the warden for 2.
- else:
    No status tick.
}
->->

== function enemy_summary() => string ==
~ temp text: string = ""
{ for slot in active_enemy_slots:
    ~ temp enemy_id: int = enemy_ids_by_slot[slot]
    ~ temp slot_text: string = enemies::enemy_name(enemy_id) + " " + to_str(enemy_hp[slot])
    { if text == "":
        ~ text = slot_text
    - else:
        ~ text = text + ", " + slot_text
    }
}
~ return text
