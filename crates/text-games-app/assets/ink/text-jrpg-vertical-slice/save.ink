=== module save ===
FROM equipment IMPORT equipped_id, set_equipped
FROM events IMPORT has_flag, set_flag_to, flag_label, FLAG_MAIN_ACCEPTED, FLAG_SIDE_ACCEPTED, FLAG_REN_JOINED, FLAG_REN_SCOUTING, FLAG_REN_REJOINED, FLAG_MOONLEAF_GATHERED, FLAG_VILLAGE_SAVE_WRITTEN, FLAG_FOREST_ENCOUNTER_CLEARED, FLAG_GATE_OPENED, FLAG_MINE_BATTLE_WON, FLAG_MINE_LOOT_RESOLVED, FLAG_CARAVAN_RESCUED, FLAG_MAIN_COMPLETE, FLAG_SIDE_COMPLETE
FROM items IMPORT gold, item_count, set_gold, set_item_count, ITEM_POTION, ITEM_ANTIDOTE, ITEM_MOONLEAF, ITEM_MINE_CHARM
FROM party IMPORT party_summary, is_active, actor_hp, actor_mp, actor_exp, actor_level, restore_roster, set_actor_state, ACTOR_HERO, ACTOR_REN
FROM puzzle IMPORT input_snapshot, failed_attempt_count, is_solved, restore_state
FROM quests IMPORT QuestState, quest_state_value, objective_value, restore_quest, set_objective, QUEST_MAIN, QUEST_SIDE, OBJ_MAIN_REN, OBJ_MAIN_GATE, OBJ_MAIN_BATTLE, OBJ_SIDE_MOONLEAF
FROM world IMPORT current_location_id, current_location_name, location_name, travel_to

CONST SAVE_POINT_NONE: int = 0
CONST SAVE_POINT_VILLAGE: int = 1
CONST SAVE_POINT_GATE: int = 2
CONST SAVE_POINT_MINE: int = 3
CONST SLOT_ONE: int = 1
CONST SLOT_TWO: int = 2
CONST SLOT_THREE: int = 3
CONST slot_ids: int[] = [SLOT_ONE, SLOT_TWO, SLOT_THREE]

STRUCT SaveSlot {
    written: bool
    save_point_id: int
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
    hero_equipment: int
    ren_active: bool
    ren_hp: int
    ren_mp: int
    ren_exp: int
    ren_level: int
    ren_equipment: int
    main_state: quests::QuestState
    side_state: quests::QuestState
    obj_main_ren: int
    obj_main_gate: int
    obj_main_battle: int
    obj_side_moonleaf: int
    puzzle_input: int[]
    puzzle_failed_attempts: int
    puzzle_solved: bool
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

VAR current_slot_id: int = SLOT_ONE
VAR current_save_point: int = SAVE_POINT_NONE
VAR save_point_open: bool = false
VAR loaded_save_point: int = SAVE_POINT_NONE
VAR slots: Dict<int, SaveSlot> = %{
    1: %SaveSlot{
        written: false,
        save_point_id: SAVE_POINT_NONE,
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
        hero_equipment: 0,
        ren_active: false,
        ren_hp: 0,
        ren_mp: 0,
        ren_exp: 0,
        ren_level: 1,
        ren_equipment: 0,
        main_state: quests::QuestState.Hidden,
        side_state: quests::QuestState.Hidden,
        obj_main_ren: 0,
        obj_main_gate: 0,
        obj_main_battle: 0,
        obj_side_moonleaf: 0,
        puzzle_input: [],
        puzzle_failed_attempts: 0,
        puzzle_solved: false,
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
    },
    2: %SaveSlot{
        written: false,
        save_point_id: SAVE_POINT_NONE,
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
        hero_equipment: 0,
        ren_active: false,
        ren_hp: 0,
        ren_mp: 0,
        ren_exp: 0,
        ren_level: 1,
        ren_equipment: 0,
        main_state: quests::QuestState.Hidden,
        side_state: quests::QuestState.Hidden,
        obj_main_ren: 0,
        obj_main_gate: 0,
        obj_main_battle: 0,
        obj_side_moonleaf: 0,
        puzzle_input: [],
        puzzle_failed_attempts: 0,
        puzzle_solved: false,
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
    },
    3: %SaveSlot{
        written: false,
        save_point_id: SAVE_POINT_NONE,
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
        hero_equipment: 0,
        ren_active: false,
        ren_hp: 0,
        ren_mp: 0,
        ren_exp: 0,
        ren_level: 1,
        ren_equipment: 0,
        main_state: quests::QuestState.Hidden,
        side_state: quests::QuestState.Hidden,
        obj_main_ren: 0,
        obj_main_gate: 0,
        obj_main_battle: 0,
        obj_side_moonleaf: 0,
        puzzle_input: [],
        puzzle_failed_attempts: 0,
        puzzle_solved: false,
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
}

== function enter_save_point(save_point_id: int) => void ==
~ current_save_point = save_point_id
~ save_point_open = true

== function leave_save_point() => void ==
~ save_point_open = false
~ current_save_point = SAVE_POINT_NONE

== function current_save_point_id() => int ==
~ return current_save_point

== function loaded_save_point_id() => int ==
~ return loaded_save_point

== function current_save_point_label() => string ==
~ return save_point_label(current_save_point)

== write_slot(slot_id: int) ==
{ if !save_point_open:
    Cannot save outside a save point.
- else:
    ~ current_slot_id = slot_id
    ~ temp snapshot: SaveSlot = slots[slot_id]
    ~ snapshot.written = true
    ~ snapshot.save_point_id = current_save_point
    ~ snapshot.location_id = world::current_location_id()
    ~ snapshot.gold = items::gold
    ~ snapshot.potion_count = items::item_count(items::ITEM_POTION)
    ~ snapshot.antidote_count = items::item_count(items::ITEM_ANTIDOTE)
    ~ snapshot.moonleaf_count = items::item_count(items::ITEM_MOONLEAF)
    ~ snapshot.mine_charm_count = items::item_count(items::ITEM_MINE_CHARM)
    ~ snapshot.hero_hp = party::actor_hp(party::ACTOR_HERO)
    ~ snapshot.hero_mp = party::actor_mp(party::ACTOR_HERO)
    ~ snapshot.hero_exp = party::actor_exp(party::ACTOR_HERO)
    ~ snapshot.hero_level = party::actor_level(party::ACTOR_HERO)
    ~ snapshot.hero_equipment = equipment::equipped_id(party::ACTOR_HERO)
    ~ snapshot.ren_active = party::is_active(party::ACTOR_REN)
    ~ snapshot.ren_hp = party::actor_hp(party::ACTOR_REN)
    ~ snapshot.ren_mp = party::actor_mp(party::ACTOR_REN)
    ~ snapshot.ren_exp = party::actor_exp(party::ACTOR_REN)
    ~ snapshot.ren_level = party::actor_level(party::ACTOR_REN)
    ~ snapshot.ren_equipment = equipment::equipped_id(party::ACTOR_REN)
    ~ snapshot.main_state = quests::quest_state_value(quests::QUEST_MAIN)
    ~ snapshot.side_state = quests::quest_state_value(quests::QUEST_SIDE)
    ~ snapshot.obj_main_ren = quests::objective_value(quests::OBJ_MAIN_REN)
    ~ snapshot.obj_main_gate = quests::objective_value(quests::OBJ_MAIN_GATE)
    ~ snapshot.obj_main_battle = quests::objective_value(quests::OBJ_MAIN_BATTLE)
    ~ snapshot.obj_side_moonleaf = quests::objective_value(quests::OBJ_SIDE_MOONLEAF)
    ~ snapshot.puzzle_input = puzzle::input_snapshot()
    ~ snapshot.puzzle_failed_attempts = puzzle::failed_attempt_count()
    ~ snapshot.puzzle_solved = puzzle::is_solved()
    ~ snapshot.flag_main_accepted = events::has_flag(events::FLAG_MAIN_ACCEPTED)
    ~ snapshot.flag_side_accepted = events::has_flag(events::FLAG_SIDE_ACCEPTED)
    ~ snapshot.flag_ren_joined = events::has_flag(events::FLAG_REN_JOINED)
    ~ snapshot.flag_ren_scouting = events::has_flag(events::FLAG_REN_SCOUTING)
    ~ snapshot.flag_ren_rejoined = events::has_flag(events::FLAG_REN_REJOINED)
    ~ snapshot.flag_moonleaf_gathered = events::has_flag(events::FLAG_MOONLEAF_GATHERED)
    ~ snapshot.flag_village_save_written = events::has_flag(events::FLAG_VILLAGE_SAVE_WRITTEN)
    ~ snapshot.flag_forest_encounter_cleared = events::has_flag(events::FLAG_FOREST_ENCOUNTER_CLEARED)
    ~ snapshot.flag_gate_opened = events::has_flag(events::FLAG_GATE_OPENED)
    ~ snapshot.flag_mine_battle_won = events::has_flag(events::FLAG_MINE_BATTLE_WON)
    ~ snapshot.flag_mine_loot_resolved = events::has_flag(events::FLAG_MINE_LOOT_RESOLVED)
    ~ snapshot.flag_caravan_rescued = events::has_flag(events::FLAG_CARAVAN_RESCUED)
    ~ snapshot.flag_main_complete = events::has_flag(events::FLAG_MAIN_COMPLETE)
    ~ snapshot.flag_side_complete = events::has_flag(events::FLAG_SIDE_COMPLETE)
    ~ slots[slot_id] = snapshot
    {slot_label(slot_id)} saved at {world::current_location_name()}.
    Saved party: {saved_party_summary(snapshot)}.
    Saved flags: {saved_flag_summary(snapshot)}.
}
->->

== load_slot(slot_id: int) ==
{ if !save_point_open:
    Cannot load outside a save point.
- else:
    { if slot_written(slot_id):
        -> restore_slot(slot_id) ->
        {slot_label(slot_id)} loaded from {world::current_location_name()}.
        Loaded party: {party::party_summary()}.
        Resume point: {save_point_label(loaded_save_point)}.
    - else:
        {slot_label(slot_id)} is empty.
    }
}
->->

== load_current_slot_after_death ==
{ if slot_written(current_slot_id):
    Death rollback: loading {slot_label(current_slot_id)}.
    -> restore_slot(current_slot_id) ->
    {slot_label(current_slot_id)} loaded from {world::current_location_name()}.
    Loaded party: {party::party_summary()}.
    Resume point: {save_point_label(loaded_save_point)}.
- else:
    ~ loaded_save_point = SAVE_POINT_NONE
    Death rollback failed: {slot_label(current_slot_id)} is empty.
}
->->

== restore_slot(slot_id: int) ==
~ current_slot_id = slot_id
~ save_point_open = false
~ current_save_point = SAVE_POINT_NONE
~ temp snapshot: SaveSlot = slots[slot_id]
~ loaded_save_point = snapshot.save_point_id
~ world::travel_to(snapshot.location_id)
~ items::set_gold(snapshot.gold)
~ items::set_item_count(items::ITEM_POTION, snapshot.potion_count)
~ items::set_item_count(items::ITEM_ANTIDOTE, snapshot.antidote_count)
~ items::set_item_count(items::ITEM_MOONLEAF, snapshot.moonleaf_count)
~ items::set_item_count(items::ITEM_MINE_CHARM, snapshot.mine_charm_count)
~ party::restore_roster(snapshot.ren_active)
~ party::set_actor_state(party::ACTOR_HERO, snapshot.hero_hp, snapshot.hero_mp, snapshot.hero_exp, snapshot.hero_level)
~ party::set_actor_state(party::ACTOR_REN, snapshot.ren_hp, snapshot.ren_mp, snapshot.ren_exp, snapshot.ren_level)
~ equipment::set_equipped(party::ACTOR_HERO, snapshot.hero_equipment)
~ equipment::set_equipped(party::ACTOR_REN, snapshot.ren_equipment)
~ quests::restore_quest(quests::QUEST_MAIN, snapshot.main_state)
~ quests::restore_quest(quests::QUEST_SIDE, snapshot.side_state)
~ quests::set_objective(quests::OBJ_MAIN_REN, snapshot.obj_main_ren)
~ quests::set_objective(quests::OBJ_MAIN_GATE, snapshot.obj_main_gate)
~ quests::set_objective(quests::OBJ_MAIN_BATTLE, snapshot.obj_main_battle)
~ quests::set_objective(quests::OBJ_SIDE_MOONLEAF, snapshot.obj_side_moonleaf)
~ puzzle::restore_state(snapshot.puzzle_input, snapshot.puzzle_failed_attempts, snapshot.puzzle_solved)
~ restore_flags(snapshot)
->->

== show_slots ==
Save point: {save_point_label(current_save_point)}
Current slot: {slot_label(current_slot_id)}
Slots:
{ for slot_id in slot_ids:
{slot_label(slot_id)}: {slot_summary(slot_id)}
}
->->

== function any_slot_written() => bool ==
~ temp found: bool = false
{ for slot_id in slot_ids:
    { if slot_written(slot_id):
        ~ found = true
    }
}
~ return found

== function slot_written(slot_id: int) => bool ==
{ if DICT_HAS(slots, slot_id):
    ~ return slots[slot_id].written
- else:
    ~ return false
}

== function slot_label(slot_id: int) => string ==
~ return "Slot " + to_str(slot_id)

== function slot_choice_text(slot_id: int) => string ==
~ return slot_label(slot_id) + " - " + slot_summary(slot_id)

== function slot_summary(slot_id: int) => string ==
{ if !slot_written(slot_id):
    ~ return "empty"
- else:
    ~ temp snapshot: SaveSlot = slots[slot_id]
    ~ return world::location_name(snapshot.location_id) + "; " + save_point_label(snapshot.save_point_id) + "; gold " + to_str(snapshot.gold) + "; " + saved_party_summary(snapshot)
}

== function save_point_label(save_point_id: int) => string ==
{ switch save_point_id:
- SAVE_POINT_VILLAGE:
    ~ return "Village shrine"
- SAVE_POINT_GATE:
    ~ return "Mine gate camp"
- SAVE_POINT_MINE:
    ~ return "Mine lift camp"
- else:
    ~ return "not at save point"
}

== function saved_party_summary(snapshot: SaveSlot) => string ==
{ if snapshot.ren_active:
    ~ return "Lio HP " + to_str(snapshot.hero_hp) + " MP " + to_str(snapshot.hero_mp) + ", Ren HP " + to_str(snapshot.ren_hp) + " MP " + to_str(snapshot.ren_mp)
- else:
    ~ return "Lio HP " + to_str(snapshot.hero_hp) + " MP " + to_str(snapshot.hero_mp)
}

== function saved_flag_summary(snapshot: SaveSlot) => string ==
~ temp text: string = ""
{ if snapshot.flag_main_accepted:
    ~ text = append_flag(text, events::FLAG_MAIN_ACCEPTED)
}
{ if snapshot.flag_side_accepted:
    ~ text = append_flag(text, events::FLAG_SIDE_ACCEPTED)
}
{ if snapshot.flag_ren_joined:
    ~ text = append_flag(text, events::FLAG_REN_JOINED)
}
{ if snapshot.flag_ren_scouting:
    ~ text = append_flag(text, events::FLAG_REN_SCOUTING)
}
{ if snapshot.flag_ren_rejoined:
    ~ text = append_flag(text, events::FLAG_REN_REJOINED)
}
{ if snapshot.flag_moonleaf_gathered:
    ~ text = append_flag(text, events::FLAG_MOONLEAF_GATHERED)
}
{ if snapshot.flag_village_save_written:
    ~ text = append_flag(text, events::FLAG_VILLAGE_SAVE_WRITTEN)
}
{ if snapshot.flag_forest_encounter_cleared:
    ~ text = append_flag(text, events::FLAG_FOREST_ENCOUNTER_CLEARED)
}
{ if snapshot.flag_gate_opened:
    ~ text = append_flag(text, events::FLAG_GATE_OPENED)
}
{ if snapshot.flag_mine_battle_won:
    ~ text = append_flag(text, events::FLAG_MINE_BATTLE_WON)
}
{ if snapshot.flag_mine_loot_resolved:
    ~ text = append_flag(text, events::FLAG_MINE_LOOT_RESOLVED)
}
{ if snapshot.flag_caravan_rescued:
    ~ text = append_flag(text, events::FLAG_CARAVAN_RESCUED)
}
{ if snapshot.flag_main_complete:
    ~ text = append_flag(text, events::FLAG_MAIN_COMPLETE)
}
{ if snapshot.flag_side_complete:
    ~ text = append_flag(text, events::FLAG_SIDE_COMPLETE)
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

== function restore_flags(snapshot: SaveSlot) => void ==
~ events::set_flag_to(events::FLAG_MAIN_ACCEPTED, snapshot.flag_main_accepted)
~ events::set_flag_to(events::FLAG_SIDE_ACCEPTED, snapshot.flag_side_accepted)
~ events::set_flag_to(events::FLAG_REN_JOINED, snapshot.flag_ren_joined)
~ events::set_flag_to(events::FLAG_REN_SCOUTING, snapshot.flag_ren_scouting)
~ events::set_flag_to(events::FLAG_REN_REJOINED, snapshot.flag_ren_rejoined)
~ events::set_flag_to(events::FLAG_MOONLEAF_GATHERED, snapshot.flag_moonleaf_gathered)
~ events::set_flag_to(events::FLAG_VILLAGE_SAVE_WRITTEN, snapshot.flag_village_save_written)
~ events::set_flag_to(events::FLAG_FOREST_ENCOUNTER_CLEARED, snapshot.flag_forest_encounter_cleared)
~ events::set_flag_to(events::FLAG_GATE_OPENED, snapshot.flag_gate_opened)
~ events::set_flag_to(events::FLAG_MINE_BATTLE_WON, snapshot.flag_mine_battle_won)
~ events::set_flag_to(events::FLAG_MINE_LOOT_RESOLVED, snapshot.flag_mine_loot_resolved)
~ events::set_flag_to(events::FLAG_CARAVAN_RESCUED, snapshot.flag_caravan_rescued)
~ events::set_flag_to(events::FLAG_MAIN_COMPLETE, snapshot.flag_main_complete)
~ events::set_flag_to(events::FLAG_SIDE_COMPLETE, snapshot.flag_side_complete)
