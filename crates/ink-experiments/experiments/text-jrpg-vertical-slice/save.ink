=== module save ===
FROM items IMPORT gold, item_count, count_label, set_gold, set_item_count, ITEM_POTION, ITEM_ANTIDOTE, ITEM_MOONLEAF, ITEM_MINE_CHARM
FROM world IMPORT current_location_id, current_location_name, location_name, travel_to
FROM party IMPORT party_summary, is_active, actor_hp, actor_mp, actor_exp, actor_level, restore_roster, set_actor_state, count_label, ACTOR_HERO, ACTOR_REN
FROM quests IMPORT QuestState, quest_state_value, objective_value, restore_quest, set_objective, state_label, QUEST_MAIN, QUEST_SIDE, OBJ_MAIN_REN, OBJ_MAIN_GATE, OBJ_MAIN_BATTLE, OBJ_SIDE_MOONLEAF
FROM events IMPORT has_flag, set_flag_to, flag_label, FLAG_MAIN_ACCEPTED, FLAG_SIDE_ACCEPTED, FLAG_REN_JOINED, FLAG_REN_SCOUTING, FLAG_REN_REJOINED, FLAG_MOONLEAF_GATHERED, FLAG_VILLAGE_SAVE_WRITTEN, FLAG_FOREST_ENCOUNTER_CLEARED, FLAG_GATE_OPENED, FLAG_MINE_BATTLE_WON, FLAG_MINE_LOOT_RESOLVED, FLAG_CARAVAN_RESCUED, FLAG_MAIN_COMPLETE, FLAG_SIDE_COMPLETE

STRUCT SaveSlot {
    written: bool
    location_id: int
    gold: int
    potion_count: int
    antidote_count: int
    moonleaf_count: int
    mine_charm_count: int
    hero_hp: int
    hero_mp: int
    hero_exp: int
    hero_level: int
    ren_active: bool
    ren_hp: int
    ren_mp: int
    ren_exp: int
    ren_level: int
    main_state: quests::QuestState
    side_state: quests::QuestState
    obj_main_ren: int
    obj_main_gate: int
    obj_main_battle: int
    obj_side_moonleaf: int
    flag_main_accepted: bool
    flag_side_accepted: bool
    flag_ren_joined: bool
    flag_ren_scouting: bool
    flag_ren_rejoined: bool
    flag_moonleaf_gathered: bool
    flag_village_save_written: bool
    flag_forest_encounter_cleared: bool
    flag_gate_opened: bool
    flag_mine_battle_won: bool
    flag_mine_loot_resolved: bool
    flag_caravan_rescued: bool
    flag_main_complete: bool
    flag_side_complete: bool
}

VAR slot: SaveSlot = %SaveSlot{
    written: false,
    location_id: 0,
    gold: 0,
    potion_count: 0,
    antidote_count: 0,
    moonleaf_count: 0,
    mine_charm_count: 0,
    hero_hp: 0,
    hero_mp: 0,
    hero_exp: 0,
    hero_level: 1,
    ren_active: false,
    ren_hp: 0,
    ren_mp: 0,
    ren_exp: 0,
    ren_level: 1,
    main_state: quests::QuestState.Hidden,
    side_state: quests::QuestState.Hidden,
    obj_main_ren: 0,
    obj_main_gate: 0,
    obj_main_battle: 0,
    obj_side_moonleaf: 0,
    flag_main_accepted: false,
    flag_side_accepted: false,
    flag_ren_joined: false,
    flag_ren_scouting: false,
    flag_ren_rejoined: false,
    flag_moonleaf_gathered: false,
    flag_village_save_written: false,
    flag_forest_encounter_cleared: false,
    flag_gate_opened: false,
    flag_mine_battle_won: false,
    flag_mine_loot_resolved: false,
    flag_caravan_rescued: false,
    flag_main_complete: false,
    flag_side_complete: false
}

== write_checkpoint ==
~ slot.written = true
~ slot.location_id = world::current_location_id()
~ slot.gold = items::gold
~ slot.potion_count = items::item_count(items::ITEM_POTION)
~ slot.antidote_count = items::item_count(items::ITEM_ANTIDOTE)
~ slot.moonleaf_count = items::item_count(items::ITEM_MOONLEAF)
~ slot.mine_charm_count = items::item_count(items::ITEM_MINE_CHARM)
~ slot.hero_hp = party::actor_hp(party::ACTOR_HERO)
~ slot.hero_mp = party::actor_mp(party::ACTOR_HERO)
~ slot.hero_exp = party::actor_exp(party::ACTOR_HERO)
~ slot.hero_level = party::actor_level(party::ACTOR_HERO)
~ slot.ren_active = party::is_active(party::ACTOR_REN)
~ slot.ren_hp = party::actor_hp(party::ACTOR_REN)
~ slot.ren_mp = party::actor_mp(party::ACTOR_REN)
~ slot.ren_exp = party::actor_exp(party::ACTOR_REN)
~ slot.ren_level = party::actor_level(party::ACTOR_REN)
~ slot.main_state = quests::quest_state_value(quests::QUEST_MAIN)
~ slot.side_state = quests::quest_state_value(quests::QUEST_SIDE)
~ slot.obj_main_ren = quests::objective_value(quests::OBJ_MAIN_REN)
~ slot.obj_main_gate = quests::objective_value(quests::OBJ_MAIN_GATE)
~ slot.obj_main_battle = quests::objective_value(quests::OBJ_MAIN_BATTLE)
~ slot.obj_side_moonleaf = quests::objective_value(quests::OBJ_SIDE_MOONLEAF)
~ slot.flag_main_accepted = events::has_flag(events::FLAG_MAIN_ACCEPTED)
~ slot.flag_side_accepted = events::has_flag(events::FLAG_SIDE_ACCEPTED)
~ slot.flag_ren_joined = events::has_flag(events::FLAG_REN_JOINED)
~ slot.flag_ren_scouting = events::has_flag(events::FLAG_REN_SCOUTING)
~ slot.flag_ren_rejoined = events::has_flag(events::FLAG_REN_REJOINED)
~ slot.flag_moonleaf_gathered = events::has_flag(events::FLAG_MOONLEAF_GATHERED)
~ slot.flag_village_save_written = events::has_flag(events::FLAG_VILLAGE_SAVE_WRITTEN)
~ slot.flag_forest_encounter_cleared = events::has_flag(events::FLAG_FOREST_ENCOUNTER_CLEARED)
~ slot.flag_gate_opened = events::has_flag(events::FLAG_GATE_OPENED)
~ slot.flag_mine_battle_won = events::has_flag(events::FLAG_MINE_BATTLE_WON)
~ slot.flag_mine_loot_resolved = events::has_flag(events::FLAG_MINE_LOOT_RESOLVED)
~ slot.flag_caravan_rescued = events::has_flag(events::FLAG_CARAVAN_RESCUED)
~ slot.flag_main_complete = events::has_flag(events::FLAG_MAIN_COMPLETE)
~ slot.flag_side_complete = events::has_flag(events::FLAG_SIDE_COMPLETE)
Save written at {world::location_name(slot.location_id)}.
Saved party: {saved_party_summary()}.
->->

== load_checkpoint ==
{ if slot.written:
    ~ world::travel_to(slot.location_id)
    ~ items::set_gold(slot.gold)
    ~ items::set_item_count(items::ITEM_POTION, slot.potion_count)
    ~ items::set_item_count(items::ITEM_ANTIDOTE, slot.antidote_count)
    ~ items::set_item_count(items::ITEM_MOONLEAF, slot.moonleaf_count)
    ~ items::set_item_count(items::ITEM_MINE_CHARM, slot.mine_charm_count)
    ~ party::restore_roster(slot.ren_active)
    ~ party::set_actor_state(party::ACTOR_HERO, slot.hero_hp, slot.hero_mp, slot.hero_exp, slot.hero_level)
    ~ party::set_actor_state(party::ACTOR_REN, slot.ren_hp, slot.ren_mp, slot.ren_exp, slot.ren_level)
    ~ quests::restore_quest(quests::QUEST_MAIN, slot.main_state)
    ~ quests::restore_quest(quests::QUEST_SIDE, slot.side_state)
    ~ quests::set_objective(quests::OBJ_MAIN_REN, slot.obj_main_ren)
    ~ quests::set_objective(quests::OBJ_MAIN_GATE, slot.obj_main_gate)
    ~ quests::set_objective(quests::OBJ_MAIN_BATTLE, slot.obj_main_battle)
    ~ quests::set_objective(quests::OBJ_SIDE_MOONLEAF, slot.obj_side_moonleaf)
    ~ restore_flags()
    Checkpoint loaded at {world::current_location_name()}.
    Loaded party: {party::party_summary()}.
- else:
    No checkpoint to load.
}
->->

== show_checkpoint ==
{ if slot.written:
    Checkpoint: {world::location_name(slot.location_id)}; gold {items::count_label(slot.gold)}; potions {items::count_label(slot.potion_count)}; moonleaf {items::count_label(slot.moonleaf_count)}.
    Saved party: {saved_party_summary()}.
    Saved quests: main {quests::state_label(slot.main_state)}, side {quests::state_label(slot.side_state)}.
    Saved flags: {saved_flag_summary()}.
- else:
    Checkpoint: empty.
}
->->

== function saved_party_summary() => string ==
{ if slot.ren_active:
    ~ return "Lio HP " + party::count_label(slot.hero_hp) + " MP " + party::count_label(slot.hero_mp) + ", Ren HP " + party::count_label(slot.ren_hp) + " MP " + party::count_label(slot.ren_mp)
- else:
    ~ return "Lio HP " + party::count_label(slot.hero_hp) + " MP " + party::count_label(slot.hero_mp)
}

== function saved_flag_summary() => string ==
~ temp text: string = ""
{ if slot.flag_main_accepted:
    ~ text = append_flag(text, events::FLAG_MAIN_ACCEPTED)
}
{ if slot.flag_side_accepted:
    ~ text = append_flag(text, events::FLAG_SIDE_ACCEPTED)
}
{ if slot.flag_village_save_written:
    ~ text = append_flag(text, events::FLAG_VILLAGE_SAVE_WRITTEN)
}
{ if text == "":
    ~ return "none"
- else:
    ~ return text
}

== function append_flag(text: string, flag_id: int) => string ==
{ if text == "":
    ~ return events::flag_label(flag_id)
- else:
    ~ return text + ", " + events::flag_label(flag_id)
}

== function restore_flags() => void ==
~ events::set_flag_to(events::FLAG_MAIN_ACCEPTED, slot.flag_main_accepted)
~ events::set_flag_to(events::FLAG_SIDE_ACCEPTED, slot.flag_side_accepted)
~ events::set_flag_to(events::FLAG_REN_JOINED, slot.flag_ren_joined)
~ events::set_flag_to(events::FLAG_REN_SCOUTING, slot.flag_ren_scouting)
~ events::set_flag_to(events::FLAG_REN_REJOINED, slot.flag_ren_rejoined)
~ events::set_flag_to(events::FLAG_MOONLEAF_GATHERED, slot.flag_moonleaf_gathered)
~ events::set_flag_to(events::FLAG_VILLAGE_SAVE_WRITTEN, slot.flag_village_save_written)
~ events::set_flag_to(events::FLAG_FOREST_ENCOUNTER_CLEARED, slot.flag_forest_encounter_cleared)
~ events::set_flag_to(events::FLAG_GATE_OPENED, slot.flag_gate_opened)
~ events::set_flag_to(events::FLAG_MINE_BATTLE_WON, slot.flag_mine_battle_won)
~ events::set_flag_to(events::FLAG_MINE_LOOT_RESOLVED, slot.flag_mine_loot_resolved)
~ events::set_flag_to(events::FLAG_CARAVAN_RESCUED, slot.flag_caravan_rescued)
~ events::set_flag_to(events::FLAG_MAIN_COMPLETE, slot.flag_main_complete)
~ events::set_flag_to(events::FLAG_SIDE_COMPLETE, slot.flag_side_complete)
