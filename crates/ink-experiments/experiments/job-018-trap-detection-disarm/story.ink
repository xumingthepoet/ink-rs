=== module game ===
STRUCT Trap {
    name: string
    required_perception: int
    required_tool: int
    detected: bool
    disarmed: bool
}

VAR party_stats: Dict<string, int> = %{
    "perception": 2,
    "tool": 1
}

VAR traps: Trap[] = [
    %Trap{
        name: "Hallway spike floor",
        required_perception: 2,
        required_tool: 4,
        detected: false,
        disarmed: false
    },
    %Trap{
        name: "Ceiling dart rail",
        required_perception: 4,
        required_tool: 3,
        detected: false,
        disarmed: false
    },
    %Trap{
        name: "Vault door snare",
        required_perception: 5,
        required_tool: 6,
        detected: false,
        disarmed: false
    }
]

== main ==
Trap detection and disarm checks.
Perception {party_stats["perception"]}, tool {party_stats["tool"]}.
~ evaluate_traps(0)
~ print_traps("first scan")
The party improves awareness.
~ party_stats["perception"] = party_stats["perception"] + 2
~ evaluate_traps(0)
~ print_traps("focused scan")
~ party_stats["tool"] = party_stats["tool"] + 3
~ attempt_disarm_all(0)
~ print_traps("final state")
-> DONE

== function evaluate_traps(index: int) => void ==
{ if index < LEN(traps):
    ~ temp trap: Trap = traps[index]
    { if !trap.detected && party_stats["perception"] >= trap.required_perception:
        {trap.name} detected.
        ~ traps[index].detected = true
    - else:
        { if trap.detected:
            {trap.name} already mapped in memory.
        - else:
            {trap.name} remains hidden.
        }
    }
    ~ evaluate_traps(index + 1)
}

== function attempt_disarm_all(index: int) => void ==
{ if index < LEN(traps):
    ~ temp trap: Trap = traps[index]
    { if !trap.detected:
        {trap.name} is not detected and cannot be disarmed.
    - else:
        { if trap.disarmed:
            {trap.name} stays disarmed.
        - else:
            { if party_stats["tool"] >= trap.required_tool:
                {trap.name} disarmed successfully.
                ~ traps[index].disarmed = true
            - else:
                {trap.name} resists disarm attempts.
            }
        }
    }
    ~ attempt_disarm_all(index + 1)
}

== function print_traps(label: string) => void ==
-- {label} --
~ print_trap_status(0)

== function print_trap_status(index: int) => void ==
{ if index < LEN(traps):
    ~ temp trap: Trap = traps[index]
    { if trap.detected:
        { if trap.disarmed:
            {trap.name} | detected, disarmed
        - else:
            {trap.name} | detected, armed
        }
    - else:
        {trap.name} | unknown
    }
    ~ print_trap_status(index + 1)
}
