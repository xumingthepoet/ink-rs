=== module game ===
FROM dialogue IMPORT elder_start, apothecary_start, ren_join, ren_scouts, ren_returns, caravan_rescue, ending
FROM events IMPORT set_flag, flag_summary, FLAG_MAIN_ACCEPTED, FLAG_SIDE_ACCEPTED, FLAG_REN_JOINED, FLAG_REN_SCOUTING, FLAG_REN_REJOINED, FLAG_MOONLEAF_GATHERED, FLAG_VILLAGE_SAVE_WRITTEN, FLAG_GATE_OPENED, FLAG_CARAVAN_RESCUED, FLAG_MAIN_COMPLETE, FLAG_SIDE_COMPLETE
FROM quests IMPORT accept_main, accept_side, set_objective, complete_main, complete_side, quest_log, objective_summary, OBJ_MAIN_REN, OBJ_MAIN_GATE, OBJ_MAIN_BATTLE, OBJ_SIDE_MOONLEAF
FROM party IMPORT join_ren, leave_ren, rejoin_ren, party_summary, damage_actor, ACTOR_HERO, ACTOR_REN
FROM items IMPORT add_item, add_gold, inventory_summary, gold, ITEM_POTION, ITEM_ANTIDOTE, ITEM_MINE_CHARM
FROM equipment IMPORT equip, equipment_summary, EQUIP_GUARD_BADGE
FROM shop IMPORT show_village_shop, buy
FROM save IMPORT enter_save_point, leave_save_point, write_slot, load_slot, show_slots, current_save_point_id, current_save_point_label, loaded_save_point_id, any_slot_written, slot_written, slot_label, slot_choice_text, SAVE_POINT_VILLAGE, SAVE_POINT_GATE, SAVE_POINT_MINE
FROM world IMPORT travel_to, current_location_name, travel_summary, LOC_FOREST, LOC_GATE, LOC_MINE, LOC_VILLAGE
FROM encounters IMPORT forest_encounter
FROM puzzle IMPORT gate
FROM battle IMPORT mine_battle

== main ==
Text JRPG vertical slice.
Location: {world::current_location_name()}
Routes: {world::travel_summary()}
Party: {party::party_summary()}
Inventory: {items::inventory_summary()}
* Talk to Elder Mara
    -> elder_scene

== elder_scene ==
-> dialogue::elder_start ->
~ quests::accept_main()
~ events::set_flag(events::FLAG_MAIN_ACCEPTED)
Quest log: {quests::quest_log()}.
* Talk to Apothecary Senn
    -> apothecary_scene

== apothecary_scene ==
-> dialogue::apothecary_start ->
~ quests::accept_side()
~ events::set_flag(events::FLAG_SIDE_ACCEPTED)
Objectives: {quests::objective_summary()}.
* Visit village shop
    -> shop_scene

== shop_scene ==
-> shop::show_village_shop ->
* Buy Potion
    -> buy_potion

== buy_potion ==
-> shop::buy(items::ITEM_POTION) ->
Inventory: {items::inventory_summary()}.
* Try Antidote
    -> fail_antidote

== fail_antidote ==
-> shop::buy(items::ITEM_ANTIDOTE) ->
* Try Mine Charm
    -> fail_charm

== fail_charm ==
-> shop::buy(items::ITEM_MINE_CHARM) ->
* Use save shrine
    -> save_scene

== save_scene ==
~ events::set_flag(events::FLAG_VILLAGE_SAVE_WRITTEN)
~ save::enter_save_point(save::SAVE_POINT_VILLAGE)
-> save_point_menu

== save_point_menu ==
Save point menu: {save::current_save_point_label()}.
-> save::show_slots ->
* Save current state
    -> save_choose_slot
* {save::any_slot_written()}: Load a slot
    -> save_load_slot
* {save::any_slot_written() && save::current_save_point_id() == save::SAVE_POINT_VILLAGE}: Stress mutate state before loading
    -> village_save_stress
* Continue
    -> save_continue

== save_choose_slot ==
Choose a slot to write.
* [slot_id in save::slot_ids] {save::slot_label(slot_id)}
    -> save_write_slot(slot_id)

== save_write_slot(slot_id: int) ==
-> save::write_slot(slot_id) ->
-> save_point_menu

== save_load_slot ==
Choose a slot to load.
* [slot_id in save::slot_ids] {save::slot_written(slot_id)}: {save::slot_choice_text(slot_id)}
    -> save_load_selected_slot(slot_id)
* Back
    -> save_point_menu

== save_load_selected_slot(slot_id: int) ==
-> save::load_slot(slot_id) ->
~ save::leave_save_point()
-> resume_loaded_save

== village_save_stress ==
~ items::add_gold(3)
~ party::damage_actor(party::ACTOR_HERO, 4)
Temporary state before slot load: gold {to_str(items::gold)}, party {party::party_summary()}.
-> save_point_menu

== save_continue ==
~ temp save_point_id: int = save::current_save_point_id()
~ save::leave_save_point()
{ switch save_point_id:
- save::SAVE_POINT_VILLAGE:
    -> village_after_save
- save::SAVE_POINT_GATE:
    -> gate_after_save
- else:
    -> mine_after_save
}

== resume_loaded_save ==
{ switch save::loaded_save_point_id():
- save::SAVE_POINT_VILLAGE:
    -> village_after_save
- save::SAVE_POINT_GATE:
    -> gate_after_save
- else:
    -> mine_after_save
}

== village_after_save ==
Flags: {events::flag_summary()}.
* Recruit Ren
    -> recruit_scene

== recruit_scene ==
-> dialogue::ren_join ->
~ party::join_ren()
~ equipment::equip(party::ACTOR_REN, equipment::EQUIP_GUARD_BADGE)
~ quests::set_objective(quests::OBJ_MAIN_REN, 1)
~ events::set_flag(events::FLAG_REN_JOINED)
Party: {party::party_summary()}
Equipment: {equipment::equipment_summary()}
* Travel to forest
    -> forest_scene

== forest_scene ==
~ world::travel_to(world::LOC_FOREST)
Location: {world::current_location_name()}.
-> encounters::forest_encounter ->
Inventory: {items::inventory_summary()}.
* Gather moonleaf
    -> gather_scene

== gather_scene ==
~ quests::set_objective(quests::OBJ_SIDE_MOONLEAF, 1)
~ events::set_flag(events::FLAG_MOONLEAF_GATHERED)
~ items::add_item(items::ITEM_ANTIDOTE, 1)
Senn's marker glows; moonleaf secured and an antidote is prepared.
Objectives: {quests::objective_summary()}.
* Let Ren scout ahead
    -> ren_scout_scene

== ren_scout_scene ==
-> dialogue::ren_scouts ->
~ party::leave_ren()
~ events::set_flag(events::FLAG_REN_SCOUTING)
Party: {party::party_summary()}.
* Travel to mine gate
    -> gate_scene

== gate_scene ==
~ world::travel_to(world::LOC_GATE)
-> dialogue::ren_returns ->
~ party::rejoin_ren()
~ events::set_flag(events::FLAG_REN_REJOINED)
Party: {party::party_summary()}.
* Use gate camp save point
    -> gate_save_scene

== gate_save_scene ==
~ save::enter_save_point(save::SAVE_POINT_GATE)
-> save_point_menu

== gate_after_save ==
-> puzzle::gate ->
~ quests::set_objective(quests::OBJ_MAIN_GATE, 1)
~ events::set_flag(events::FLAG_GATE_OPENED)
* Enter the mine
    -> mine_scene

== mine_scene ==
~ world::travel_to(world::LOC_MINE)
Location: {world::current_location_name()}.
* Use mine lift save point
    -> mine_save_scene

== mine_save_scene ==
~ save::enter_save_point(save::SAVE_POINT_MINE)
-> save_point_menu

== mine_after_save ==
-> battle::mine_battle ->
~ quests::set_objective(quests::OBJ_MAIN_BATTLE, 1)
Inventory: {items::inventory_summary()}.
* Rescue the caravan
    -> rescue_scene

== rescue_scene ==
-> dialogue::caravan_rescue ->
~ events::set_flag(events::FLAG_CARAVAN_RESCUED)
* Return to village
    -> return_scene

== return_scene ==
~ world::travel_to(world::LOC_VILLAGE)
-> dialogue::ending ->
~ quests::complete_main()
~ quests::complete_side()
~ events::set_flag(events::FLAG_MAIN_COMPLETE)
~ events::set_flag(events::FLAG_SIDE_COMPLETE)
Quest log: {quests::quest_log()}.
Flags: {events::flag_summary()}.
-> save::show_slots ->
JRPG slice complete.
-> END
