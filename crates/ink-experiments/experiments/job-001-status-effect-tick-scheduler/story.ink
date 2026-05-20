=== module game ===
STRUCT StatusEffect {
    id: int
    name: string
    remaining: int
    damage_per_tick: int
}

VAR status_slots: StatusEffect[] = [
    %StatusEffect{ id: 101, name: "Poison", remaining: 3, damage_per_tick: 2 },
    %StatusEffect{ id: 102, name: "Burn", remaining: 0, damage_per_tick: 4 },
    %StatusEffect{ id: 103, name: "Chill", remaining: 0, damage_per_tick: 1 }
]

== main ==
Status scheduler starts.
-> process_tick(1)
-> DONE

== process_tick(tick: int) ==
{ if tick > 6:
    Scheduler complete.
    -> DONE
- else:
    Tick {tick}.
    { if tick == 2:
        ~ status_slots[1].remaining = 2
        Burn enters on slot 2 for 2 ticks.
    - else:
        No new status entered this tick.
    }
    -> process_statuses(0)
    -> process_tick(tick + 1)
}

== process_statuses(index: int) ==
{ if index >= LEN(status_slots):
    ->->
- else:
    { if status_slots[index].remaining > 0:
        ~ temp next_remaining: int = status_slots[index].remaining - 1
        ~ status_slots[index].remaining = next_remaining
        Status slot {status_slots[index].id} ({status_slots[index].name}) deals {status_slots[index].damage_per_tick} and now has {next_remaining} ticks left.
        { if next_remaining == 0:
            Status slot {status_slots[index].id} ({status_slots[index].name}) expires.
        }
    - else:
        Status slot {status_slots[index].id} ({status_slots[index].name}) is idle.
    }
    -> process_statuses(index + 1)
}
