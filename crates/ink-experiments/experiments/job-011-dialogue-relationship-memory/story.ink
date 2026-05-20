=== module game ===
STRUCT EncounterEvent {
    npc: string
    tone: string
    relationship_delta: int
}

VAR relationship_memory: Dict<string, int> = %{
    "Mara": 0,
    "Borin": -2
}

VAR encounter_log: EncounterEvent[] = [
    %EncounterEvent{ npc: "Mara", tone: "You share tea during a storm.", relationship_delta: 3 },
    %EncounterEvent{ npc: "Borin", tone: "You interrupt a deal and accuse him of overcharge.", relationship_delta: -3 },
    %EncounterEvent{ npc: "Mara", tone: "You repair her wagon rim.", relationship_delta: 2 },
    %EncounterEvent{ npc: "Borin", tone: "You help carry crates to market.", relationship_delta: 4 },
    %EncounterEvent{ npc: "Mara", tone: "You praise her plan in front of everyone.", relationship_delta: 1 }
]

VAR tracked_npcs: string[] = ["Mara", "Borin"]

== main ==
Dialogue relationship memory:
-> run_encounters(0)
-> DONE

== run_encounters(index: int) ==
{ if index >= LEN(encounter_log):
    -> show_relationships(0)
- else:
    ~ temp encounter: EncounterEvent = encounter_log[index]
    ~ temp previous: int = relationship_memory[encounter.npc]
    ~ relationship_memory[encounter.npc] = previous + encounter.relationship_delta
    { line_for(encounter.npc, previous) }
    {encounter.npc}: {encounter.tone}
    Memory now {relationship_memory[encounter.npc]}.
    -> run_encounters(index + 1)
}

== function line_for(npc: string, relation: int) => string ==
{ if relation >= 5:
    ~ return npc + " greets you as a trusted ally."
- else:
    { if relation >= 2:
        ~ return npc + " seems openly friendly."
    - else:
        { if relation >= 0:
            ~ return npc + " stays politely distant."
        - else:
            { if relation >= -4:
                ~ return npc + " keeps some distance."
            - else:
                ~ return npc + " avoids you entirely."
            }
        }
    }
}

== show_relationships(index: int) ==
{ if index >= LEN(tracked_npcs):
    -> DONE
- else:
    ~ temp npc: string = tracked_npcs[index]
    ~ temp value: int = relationship_memory[npc]
    {npc} final status: {relationship_label(value)} ({value}).
    -> show_relationships(index + 1)
}

== function relationship_label(value: int) => string ==
{ if value >= 5:
    ~ return "Trusted"
- else:
    { if value >= 2:
        ~ return "Friendly"
    - else:
        { if value >= 0:
            ~ return "Neutral"
        - else:
            { if value >= -4:
                ~ return "Cold"
            - else:
                ~ return "Hostile"
            }
        }
    }
}
