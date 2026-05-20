=== module game ===

STRUCT Injury {
    part: string
    severity: int
    treatment_cost: int
    treated: bool
    status: string
    recovery_days: int
}

VAR medical_supplies: int = 10

VAR injuries: Injury[] = [
    %Injury{
        part: "Left forearm",
        severity: 9,
        treatment_cost: 5,
        treated: false,
        status: "queued",
        recovery_days: 0
    },
    %Injury{
        part: "Rib fracture",
        severity: 7,
        treatment_cost: 4,
        treated: false,
        status: "queued",
        recovery_days: 0
    },
    %Injury{
        part: "Scalp cut",
        severity: 3,
        treatment_cost: 3,
        treated: false,
        status: "queued",
        recovery_days: 0
    },
    %Injury{
        part: "Bruised knee",
        severity: 4,
        treatment_cost: 2,
        treated: false,
        status: "queued",
        recovery_days: 0
    }
]

== main ==
Field triage simulation.
Medical supplies available: {medical_supplies}.
~ print_injuries("Initial list")
~ rank_injuries()
Triage order ready.
~ print_injuries("Ranked by severity")
~ treat_all(0)
Triage treatment complete.
~ print_injuries("After treatment")
~ simulate_recovery_days(0)
Triage complete.
~ print_injuries("Final recovery status")
-> DONE

== function rank_injuries() => void ==
~ rank_injuries_step(1)

== function rank_injuries_step(index: int) => void ==
{ if index < LEN(injuries):
    ~ place_injury(index)
    ~ rank_injuries_step(index + 1)
}

== function place_injury(index: int) => void ==
{ if index > 0:
    { if injuries[index - 1].severity < injuries[index].severity:
        ~ temp swapper: Injury = injuries[index - 1]
        ~ temp current: Injury = injuries[index]
        ~ injuries[index - 1] = current
        ~ injuries[index] = swapper
        ~ place_injury(index - 1)
    }
}

== function print_injuries(label: string) => void ==
-- {label} --
~ print_injury_entry(0)

== function print_injury_entry(index: int) => void ==
{ if index < LEN(injuries):
    ~ temp injury: Injury = injuries[index]
    { if injury.treated:
        {index + 1}. {injury.part} | severity {injury.severity} | treated | status {injury.status} | recovery {injury.recovery_days}
    - else:
        {index + 1}. {injury.part} | severity {injury.severity} | untreated | status {injury.status} | recovery {injury.recovery_days}
    }
    ~ print_injury_entry(index + 1)
}

== function treat_all(index: int) => void ==
{ if index < LEN(injuries):
    ~ temp injury: Injury = injuries[index]
    { if injury.treated:
        {injury.part} already stabilized.
    - else:
        ~ temp cost: int = injury.treatment_cost
        { if medical_supplies >= cost:
            ~ injuries[index].treated = true
            ~ medical_supplies = medical_supplies - cost
            ~ injuries[index].recovery_days = recovery_estimate(injury.severity)
            ~ injuries[index].status = "stabilized"
            Triage: treated {injury.part} for {cost} supplies.
            Supplies left: {medical_supplies}.
        - else:
            ~ injuries[index].status = "deferred"
            Warning: shortage while treating {injury.part}.
            Needed {cost} supplies, but only {medical_supplies} remain.
        }
    }
    ~ treat_all(index + 1)
}

== function recovery_estimate(severity: int) => int ==
{ if severity >= 8:
    ~ return 5
- else:
    { if severity >= 5:
        ~ return 3
    - else:
        { if severity >= 3:
            ~ return 2
        - else:
            ~ return 1
        }
    }
}

== function simulate_recovery_days(day: int) => void ==
{ if day < 3:
    ~ temp day_label: int = day + 1
    Day {day_label} recovery check.
    ~ advance_recovery(0)
    ~ simulate_recovery_days(day + 1)
}

== function advance_recovery(index: int) => void ==
{ if index < LEN(injuries):
    { if injuries[index].treated:
        { if injuries[index].recovery_days > 0:
            ~ temp remaining: int = injuries[index].recovery_days - 1
            ~ injuries[index].recovery_days = remaining
            { if remaining == 0:
                ~ injuries[index].status = "healed"
                {injuries[index].part}: recovery complete.
            - else:
                ~ injuries[index].status = "recovering"
                {injuries[index].part}: {remaining} days remaining.
            }
        - else:
            {injuries[index].part}: stable.
            ~ injuries[index].status = "stable"
        }
    - else:
        {injuries[index].part}: deferred and waiting for supplies.
    }
    ~ advance_recovery(index + 1)
}
