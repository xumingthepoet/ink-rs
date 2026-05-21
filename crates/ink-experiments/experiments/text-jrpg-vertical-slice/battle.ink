=== module battle ===
FROM party IMPORT party_summary, damage_actor, heal_actor, spend_mp, add_exp, ACTOR_HERO, ACTOR_REN
FROM items IMPORT remove_item
FROM skills IMPORT use_skill, SKILL_SPARK, SKILL_GUARD, SKILL_STRIKE
FROM loot IMPORT resolve, LOOT_MINE
FROM events IMPORT set_flag, FLAG_MINE_BATTLE_WON, FLAG_MINE_LOOT_RESOLVED

VAR enemy_hp: Dict<int, int> = %{}
VAR poison_turns: int = 0
VAR guarded: bool = false
VAR turn: int = 1

== mine_battle ==
~ enemy_hp = %{1: 6, 2: 10}
~ poison_turns = 0
~ guarded = false
~ turn = 1
Mine battle begins: Cave Imp and Mine Warden.
-> battle_prompt

== battle_prompt ==
Turn {count_label(turn)}
Party: {party::party_summary()}
Enemies: imp {count_label(enemy_hp[1])}, warden {count_label(enemy_hp[2])}
Poison turns: {count_label(poison_turns)}
* Hero Spark
    -> hero_spark
* Use Potion
    -> use_potion
* Ren Guard
    -> ren_guard
* Hero Strike
    -> hero_strike

== hero_spark ==
-> skills::use_skill(skills::SKILL_SPARK, party::ACTOR_HERO, 2) ->
{ if party::spend_mp(party::ACTOR_HERO, 2):
    ~ enemy_hp[2] = enemy_hp[2] - 4
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
{ if items::remove_item(1, 1):
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
-> skills::use_skill(skills::SKILL_STRIKE, party::ACTOR_HERO, 1) ->
~ enemy_hp[1] = 0
~ enemy_hp[2] = 0
Lio and Ren finish the patrol.
~ events::set_flag(events::FLAG_MINE_BATTLE_WON)
Victory: mine patrol defeated.
~ temp level_text: string = party::add_exp(5)
EXP result: {level_text}.
-> loot::resolve(loot::LOOT_MINE) ->
~ events::set_flag(events::FLAG_MINE_LOOT_RESOLVED)
->->

== enemy_turn ==
{ if guarded:
    ~ party::damage_actor(party::ACTOR_HERO, 1)
    Mine Warden attacks into Ren's guard for 1 damage.
    ~ guarded = false
- else:
    ~ party::damage_actor(party::ACTOR_HERO, 3)
    Mine Warden strikes Lio for 3 damage.
}
->->

== status_tick ==
{ if poison_turns > 0:
    ~ enemy_hp[2] = enemy_hp[2] - 2
    ~ poison_turns -= 1
    Poison ticks on the warden for 2.
- else:
    No status tick.
}
->->

== function count_label(value: int) => string ==
{ switch value:
- 0:
    ~ return "0"
- 1:
    ~ return "1"
- 2:
    ~ return "2"
- 3:
    ~ return "3"
- 4:
    ~ return "4"
- 5:
    ~ return "5"
- 6:
    ~ return "6"
- 7:
    ~ return "7"
- 8:
    ~ return "8"
- 9:
    ~ return "9"
- else:
    ~ return "10"
}
