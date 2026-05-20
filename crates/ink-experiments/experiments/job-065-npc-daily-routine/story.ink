=== module game ===

STRUCT NPC {
    name: string
    role: string
}

STRUCT ScheduledDuty {
    npc: string
    location: string
    action: string
}

STRUCT Disruption {
    block: string
    npc: string
    reason: string
    reroute: string
    delay: int
}

VAR time_blocks: string[] = [
    "Dawn Watch",
    "Midday Patrol",
    "Dusk Sweep",
    "Night Rounds"
]

VAR roster: NPC[] = [
    %NPC{name: "Ari", role: "warden"},
    %NPC{name: "Brin", role: "messenger"},
    %NPC{name: "Lio", role: "forager"},
    %NPC{name: "Mara", role: "ranger"},
    %NPC{name: "Soren", role: "healer"}
]

VAR schedule: Dict<string, ScheduledDuty[]> = %{
    "Dawn Watch": [
        %ScheduledDuty{npc: "Ari", location: "North Gate", action: "log entry books"},
        %ScheduledDuty{npc: "Brin", location: "South Gate", action: "call shift bells"},
        %ScheduledDuty{npc: "Lio", location: "Outer Wall", action: "check wall torches"}
    ],
    "Midday Patrol": [
        %ScheduledDuty{npc: "Mara", location: "Market", action: "watch crowd paths"},
        %ScheduledDuty{npc: "Lio", location: "River Wharf", action: "escort suppliers"},
        %ScheduledDuty{npc: "Ari", location: "City Hall", action: "brief magistrate patrols"},
        %ScheduledDuty{npc: "Soren", location: "Temple", action: "check infirmary requests"}
    ],
    "Dusk Sweep": [
        %ScheduledDuty{npc: "Soren", location: "Residential Ring", action: "visit fire checks"},
        %ScheduledDuty{npc: "Brin", location: "Courier Dock", action: "route outgoing packets"},
        %ScheduledDuty{npc: "Mara", location: "Outer Docks", action: "inspect night stores"},
        %ScheduledDuty{npc: "Ari", location: "Watchtower", action: "prepare shift handoff"}
    ],
    "Night Rounds": [
        %ScheduledDuty{npc: "Lio", location: "North Ridge", action: "scan for intruders"},
        %ScheduledDuty{npc: "Brin", location: "North Ridge", action: "replace lantern oils"},
        %ScheduledDuty{npc: "Mara", location: "Old Mill", action: "guard gate lock"},
        %ScheduledDuty{npc: "Soren", location: "Monastery", action: "quiet patrol"}
    ]
}

VAR disruptions: Disruption[] = [
    %Disruption{
        block: "Midday Patrol",
        npc: "Mara",
        reason: "market stampede blocks alley lanes",
        reroute: "Watchtower",
        delay: 52
    },
    %Disruption{
        block: "Dusk Sweep",
        npc: "Brin",
        reason: "storm drains flood the cargo lane",
        reroute: "Temple Stair",
        delay: 34
    },
    %Disruption{
        block: "Night Rounds",
        npc: "Lio",
        reason: "wild fox triggers alarm bells",
        reroute: "Monastery Bell",
        delay: 48
    },
    %Disruption{
        block: "Dawn Watch",
        npc: "Ari",
        reason: "lantern rack burns too slow to relight",
        reroute: "North Gate Annex",
        delay: 18
    }
]

VAR disruption_by_npc: Dict<string, int> = %{
    "Ari": 0,
    "Brin": 0,
    "Lio": 0,
    "Mara": 0,
    "Soren": 0
}

VAR resolved_tasks: int = 0
VAR disrupted_tasks: int = 0
VAR delay_minutes: int = 0
VAR carried_over_minutes: int = 0
VAR backups_called: int = 0

== main ==
Municipal routine simulation starts.
~ run_schedule(0)
~ print_day_summary()
-> DONE

== function run_schedule(block_index: int) => void ==
{ if block_index >= LEN(time_blocks):
    Day schedule complete.
- else:
    ~ temp block: string = time_blocks[block_index]
    ~ print_block_open(block, carried_over_minutes)
    ~ carried_over_minutes = 0
    ~ process_block(block, 0)
    ~ run_schedule(block_index + 1)
}

== function print_block_open(block: string, carry: int) => void ==
-- {block} --
{ if carry > 0:
    Block is delayed by {carry} minutes. Backup readiness is increased.
- else:
    Block starts on time.
}

== function process_block(block: string, duty_index: int) => void ==
{ if duty_index >= LEN(schedule[block]):
    Block {block} check complete.
- else:
    ~ temp duty: ScheduledDuty = schedule[block][duty_index]
    ~ temp disruption_index: int = find_disruption(block, duty.npc, 0)
    { if disruption_index == -1:
        {duty.npc} reaches {duty.location} for {duty.action}.
        ~ resolved_tasks = resolved_tasks + 1
    - else:
        ~ temp issue: Disruption = disruptions[disruption_index]
        {duty.npc} cannot follow the plan.
        Disruption: {issue.reason}
        Response profile: {response_profile(issue.delay)}
        {duty.npc} shifts to {issue.reroute} and adjusts task.
        ~ disruption_by_npc[duty.npc] = disruption_by_npc[duty.npc] + 1
        ~ disrupted_tasks = disrupted_tasks + 1
        ~ delay_minutes = delay_minutes + issue.delay

        { if issue.delay >= 45:
            Response requires next-block borrow.
            ~ backups_called = backups_called + 1
            ~ carried_over_minutes = carried_over_minutes + issue.delay
        - else:
            Reroute still fits this block.
        }
    }

    ~ process_block(block, duty_index + 1)
}

== function find_disruption(block: string, npc: string, index: int) => int ==
{ if index >= LEN(disruptions):
    ~ return -1
- else:
    ~ temp candidate: Disruption = disruptions[index]
    { if candidate.block == block && candidate.npc == npc:
        ~ return index
    - else:
        ~ return find_disruption(block, npc, index + 1)
    }
}

== function response_profile(delay: int) => string ==
{ if delay >= 50:
    ~ return "containment + full reroute"
- else:
    { if delay >= 40:
        ~ return "reroute + backup signal"
    - else:
        { if delay >= 25:
            ~ return "reroute + compressed route"
        - else:
            ~ return "detour + pace hold"
        }
    }
}

== function print_day_summary() => void ==
Municipal day summary.
Duty outcomes:
Resolved: {resolved_tasks}
Disrupted: {disrupted_tasks}
Total delay minutes: {delay_minutes}
Backups called: {backups_called}
Carryover minutes: {carried_over_minutes}
~ print_disruption_counts(0)
~ print_roster_status()

== function print_disruption_counts(index: int) => void ==
{ if index >= LEN(roster):
    ~ return
- else:
    ~ temp person: NPC = roster[index]
    ~ temp hits: int = disruption_by_npc[person.name]
    {person.name} ({person.role}) disruption hits: {hits}
    ~ print_disruption_counts(index + 1)
}

== function print_roster_status() => void ==
Roster readiness check.
~ print_npc_status(0)

== function print_npc_status(index: int) => void ==
{ if index >= LEN(roster):
    ~ return
- else:
    ~ temp person: NPC = roster[index]
    { if disruption_by_npc[person.name] > 1:
        {person.name} remains alert for repeated blockages.
    - else:
        { if disruption_by_npc[person.name] == 1:
            {person.name} can hold position once disrupted.
        - else:
            {person.name} held steady with no disruptions.
        }
    }
    ~ print_npc_status(index + 1)
}
