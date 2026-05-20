=== module game ===
STRUCT AffinityBanter {
    companion: string
    line: string
    min_affinity: int
    max_affinity: int
    delivered: bool
}

VAR companion_affinity: Dict<string, int> = %{
    "Mira": 4,
    "Tao": 2,
    "Nia": 4
}

VAR banter_queue: AffinityBanter[] = [
    %AffinityBanter{
        companion: "Mira",
        line: "Mira whispers, \"stay quiet on this line of walls.\"",
        min_affinity: 0,
        max_affinity: 8,
        delivered: false
    },
    %AffinityBanter{
        companion: "Tao",
        line: "Tao nods: \"Trust me and follow.\"",
        min_affinity: 5,
        max_affinity: 12,
        delivered: false
    },
    %AffinityBanter{
        companion: "Nia",
        line: "Nia says, \"I can cover you from the flank.\"",
        min_affinity: 9,
        max_affinity: 20,
        delivered: false
    },
    %AffinityBanter{
        companion: "Mira",
        line: "Mira grins, \"you've got me as an ally now.\"",
        min_affinity: 12,
        max_affinity: 20,
        delivered: false
    }
]

== main ==
Companion banter queue with affinity gating.
~ temp _round_1: bool = run_banter_round("1")
~ companion_affinity["Tao"] = companion_affinity["Tao"] + 5
~ companion_affinity["Nia"] = companion_affinity["Nia"] + 3
~ temp _round_2: bool = run_banter_round("2")
~ companion_affinity["Mira"] = companion_affinity["Mira"] + 8
~ companion_affinity["Nia"] = companion_affinity["Nia"] + 5
~ temp _round_3: bool = run_banter_round("3")
~ companion_affinity["Mira"] = companion_affinity["Mira"] + 1
~ temp _round_4: bool = run_banter_round("4")
~ temp _round_5: bool = run_banter_round("5")
-> DONE

== function run_banter_round(label: string) => bool ==
Round {label}:
Mira:{companion_affinity["Mira"]}, Tao:{companion_affinity["Tao"]}, Nia:{companion_affinity["Nia"]}
~ temp delivered: bool = choose_banter(0)
{ if !delivered:
    No queued banter qualifies right now.
}
~ return true

== function choose_banter(index: int) => bool ==
{ if index >= LEN(banter_queue):
    ~ return false
- else:
    { if banter_queue[index].delivered:
        ~ return choose_banter(index + 1)
    - else:
        { if queue_qualified(index):
            {banter_queue[index].line}
            ~ banter_queue[index].delivered = true
            ~ return true
        - else:
            ~ return choose_banter(index + 1)
        }
    }
}

== function queue_qualified(index: int) => bool ==
~ temp entry: AffinityBanter = banter_queue[index]
~ temp affinity: int = companion_affinity[entry.companion]
~ return affinity >= entry.min_affinity && affinity <= entry.max_affinity
