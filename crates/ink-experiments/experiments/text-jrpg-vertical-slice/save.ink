=== module save ===
FROM items IMPORT gold, item_count, count_label
FROM world IMPORT current_location_name
FROM party IMPORT party_summary
FROM quests IMPORT quest_log
FROM events IMPORT flag_summary

STRUCT SaveSlot {
    written: bool
    location: string
    party: string
    quest: string
    flags: string
    gold: int
    potion_count: int
    moonleaf_count: int
}

VAR slot: SaveSlot = %SaveSlot{}

== write_checkpoint ==
~ slot.written = true
~ slot.location = world::current_location_name()
~ slot.party = party::party_summary()
~ slot.quest = quests::quest_log()
~ slot.flags = events::flag_summary()
~ slot.gold = items::gold
~ slot.potion_count = items::item_count(1)
~ slot.moonleaf_count = items::item_count(3)
Save written at {slot.location}.
Saved party: {slot.party}.
->->

== show_checkpoint ==
{ if slot.written:
    Checkpoint: {slot.location}; gold {items::count_label(slot.gold)}; potions {items::count_label(slot.potion_count)}; moonleaf {items::count_label(slot.moonleaf_count)}.
    Saved quests: {slot.quest}.
    Saved flags: {slot.flags}.
- else:
    Checkpoint: empty.
}
->->
