=== module game ===

STRUCT ContactEvent {
    day: int
    source: string
    target: string
    exposure: int
}

VAR npc_ids: string[] = [
    "Ari",
    "Bram",
    "Cora",
    "Dax",
    "Elin"
]

VAR infection_level: Dict<string, int> = %{
    "Ari": 0,
    "Bram": 11,
    "Cora": 0,
    "Dax": 2,
    "Elin": 0
}

VAR quarantine_status: Dict<string, bool> = %{
    "Ari": false,
    "Bram": false,
    "Cora": false,
    "Dax": false,
    "Elin": true
}

VAR contact_events: ContactEvent[] = [
    %ContactEvent{ day: 1, source: "Bram", target: "Ari", exposure: 4 },
    %ContactEvent{ day: 1, source: "Ari", target: "Dax", exposure: 3 },
    %ContactEvent{ day: 1, source: "Bram", target: "Cora", exposure: 5 },
    %ContactEvent{ day: 1, source: "Elin", target: "Ari", exposure: 2 },
    %ContactEvent{ day: 1, source: "Cora", target: "Dax", exposure: 4 },
    %ContactEvent{ day: 1, source: "Dax", target: "Bram", exposure: 2 },
    %ContactEvent{ day: 1, source: "Bram", target: "Elin", exposure: 3 },
    %ContactEvent{ day: 1, source: "Ari", target: "Cora", exposure: 4 },
    %ContactEvent{ day: 2, source: "Cora", target: "Bram", exposure: 2 },
    %ContactEvent{ day: 2, source: "Elin", target: "Dax", exposure: 3 },
    %ContactEvent{ day: 2, source: "Dax", target: "Cora", exposure: 1 },
    %ContactEvent{ day: 2, source: "Ari", target: "Dax", exposure: 2 }
]

VAR current_day: int = 1
VAR new_case_count: int = 0
VAR infected_count: int = 0
VAR quarantined_count: int = 0
VAR total_exposure: int = 0
VAR highest_infected_id: string = "none"
VAR highest_infection_level: int = 0
VAR outbreak_threshold: int = 10

== main ==
Outbreak simulation by contact and quarantine.
~ print_infection_snapshot("Initial state")
~ simulate_contacts(0)
~ print_infection_snapshot("Final state")
~ summarize_outbreak()
-> DONE

== function simulate_contacts(index: int) => void ==
{ if index < LEN(contact_events):
    ~ temp event: ContactEvent = contact_events[index]
    { if current_day != event.day:
        ~ print_day_end(current_day)
        ~ current_day = event.day
        ~ print_day_start(current_day)
    }
    ~ process_event(event)
    ~ simulate_contacts(index + 1)
}

== function print_day_end(day: int) => void ==
Day {day} phase done.
-- End of day {day} --

== function print_day_start(day: int) => void ==
Day {day} starts.

== function process_event(event: ContactEvent) => void ==
~ temp source_quarantined: bool = quarantine_status[event.source]
{ if source_quarantined:
    Event blocked: {event.source} is quarantined and cannot spread to {event.target}.
- else:
    ~ temp target_quarantined: bool = quarantine_status[event.target]
    { if target_quarantined:
        Event blocked: {event.source} reaches quarantined target {event.target}.
    - else:
        ~ temp source_level: int = infection_level[event.source]
        { if source_level == 0:
            {event.source} is not currently contagious.
        - else:
            ~ temp transfer: int = event.exposure
            { if source_level >= 12:
                ~ transfer = transfer + 2
            - else:
                { if source_level >= 8:
                    ~ transfer = transfer + 1
                }
            }
            ~ temp before_level: int = infection_level[event.target]
            ~ temp after_level: int = before_level + transfer
            ~ infection_level[event.target] = after_level
            { if before_level == 0:
                {event.target} receives first exposure of {transfer} from {event.source}.
                ~ new_case_count = new_case_count + 1
            - else:
                {event.target} increases by {transfer}.
            }
            ~ evaluate_quarantine(event.target, after_level)
        }
    }
}

== function evaluate_quarantine(npc_id: string, level: int) => void ==
{ if level >= outbreak_threshold:
    { if !quarantine_status[npc_id]:
        ~ quarantine_status[npc_id] = true
        ~ quarantined_count = quarantined_count + 1
        {npc_id} reaches outbreak threshold ({level}) and is quarantined.
    - else:
        {npc_id} remains under quarantine at level {level}.
    }
}

== function print_infection_snapshot(label: string) => void ==
-- {label} --
~ print_npc_row(0)

== function print_npc_row(index: int) => void ==
{ if index < LEN(npc_ids):
    ~ temp npc: string = npc_ids[index]
    ~ temp level: int = infection_level[npc]
    ~ temp quarantined: bool = quarantine_status[npc]
    { if quarantined:
        {npc} | exposure {level} | quarantined
    - else:
        { if level == 0:
            {npc} | exposure {level} | clear
        - else:
            {npc} | exposure {level} | tracked
        }
    }
    ~ print_npc_row(index + 1)
}

== function summarize_outbreak() => void ==
~ infected_count = 0
~ quarantined_count = 0
~ total_exposure = 0
~ highest_infection_level = 0
~ highest_infected_id = "none"
~ summarize_npc(0)
Outbreak summary:
New infections this run: {new_case_count}
Tracked infected NPCs: {infected_count}
NPCs under quarantine: {quarantined_count}
Total exposure pool: {total_exposure}
Highest burden: {highest_infected_id} ({highest_infection_level})

== function summarize_npc(index: int) => void ==
{ if index < LEN(npc_ids):
    ~ temp npc: string = npc_ids[index]
    ~ temp level: int = infection_level[npc]
    ~ total_exposure = total_exposure + level
    { if level > 0:
        ~ infected_count = infected_count + 1
    }
    { if quarantine_status[npc]:
        ~ quarantined_count = quarantined_count + 1
    }
    { if level > highest_infection_level:
        ~ highest_infection_level = level
        ~ highest_infected_id = npc
    }
    ~ summarize_npc(index + 1)
}
