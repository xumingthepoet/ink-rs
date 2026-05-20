=== module game ===

STRUCT Mercenary {
    name: string
    role: string
    rank_tier: int
    wage: int
    morale: int
    wounds: int
    contract_days_left: int
    alive: bool
}

STRUCT CasualtyEvent {
    day: int
    name: string
    severity: int
    note: string
}

VAR mercenaries: Mercenary[] = [
    %Mercenary{
        name: "Ari",
        role: "Frontline",
        rank_tier: 0,
        wage: 9,
        morale: 8,
        wounds: 0,
        contract_days_left: 2,
        alive: true
    },
    %Mercenary{
        name: "Brin",
        role: "Engineer",
        rank_tier: 0,
        wage: 6,
        morale: 7,
        wounds: 1,
        contract_days_left: 1,
        alive: true
    },
    %Mercenary{
        name: "Cora",
        role: "Scout",
        rank_tier: 0,
        wage: 7,
        morale: 9,
        wounds: 0,
        contract_days_left: 3,
        alive: true
    },
    %Mercenary{
        name: "Dax",
        role: "Medic",
        rank_tier: 0,
        wage: 5,
        morale: 6,
        wounds: 1,
        contract_days_left: 3,
        alive: true
    },
    %Mercenary{
        name: "Ely",
        role: "Archer",
        rank_tier: 0,
        wage: 4,
        morale: 5,
        wounds: 2,
        contract_days_left: 2,
        alive: true
    }
]

VAR casualty_events: CasualtyEvent[] = [
    %CasualtyEvent{
        day: 2,
        name: "Brin",
        severity: 4,
        note: "Shrapnel from a collapsed support strut."
    },
    %CasualtyEvent{
        day: 2,
        name: "Dax",
        severity: 9,
        note: "Blast wound through shoulder rig."
    },
    %CasualtyEvent{
        day: 3,
        name: "Cora",
        severity: 3,
        note: "Trip through unstable rubble."
    }
]

VAR merc_index: Dict<string, int> = %{
    "Ari": 0,
    "Brin": 1,
    "Cora": 2,
    "Dax": 3,
    "Ely": 4
}

VAR treasury: int = 78
VAR simulation_days: int = 3
VAR promotions: int = 0
VAR payroll_shortfalls: int = 0
VAR casualties: int = 0
VAR terminated_contracts: int = 0
VAR readiness_score: int = 0

== main ==
Mercenary roster review begins.
Treasury starts at {treasury}.
~ print_roster("Starting roster")
~ run_day(1)
~ final_state_report()
-> DONE

== function run_day(day: int) => void ==
{ if day > simulation_days:
    Day progression complete.
- else:
    Day {day} operations begin.
    ~ process_payroll(day)
    ~ apply_casualties(day, 0)
    ~ evaluate_promotions()
    ~ calculate_readiness()
    ~ print_readiness_status(day)
    ~ print_roster("After day {day}")
    ~ run_day(day + 1)
}

== function process_payroll(day: int) => void ==
-- Payroll and contract processing --
~ process_payroll_worker(0, day)

== function process_payroll_worker(index: int, day: int) => void ==
{ if index < LEN(mercenaries):
    ~ temp merc: Mercenary = mercenaries[index]
    { if !merc.alive:
        {merc.name} is unavailable (inactive).
    - else:
        ~ mercenaries[index].contract_days_left = merc.contract_days_left - 1
        { if treasury >= merc.wage:
            ~ treasury = treasury - merc.wage
            ~ mercenaries[index].morale = merc.morale + 1
            {merc.name} receives wage {merc.wage} and morale rises to {mercenaries[index].morale}.
        - else:
            ~ payroll_shortfalls = payroll_shortfalls + 1
            ~ mercenaries[index].morale = merc.morale - 2
            {merc.name} cannot be paid today.
            ~ mercenaries[index].wounds = merc.wounds + 1
            {merc.name} morale drops to {mercenaries[index].morale}; morale stress wound added.
        }
        { if mercenaries[index].contract_days_left <= 0:
            ~ review_contract(index, day)
        }
        { if mercenaries[index].morale <= 0:
            ~ mercenaries[index].alive = false
            ~ terminated_contracts = terminated_contracts + 1
            {merc.name} is dismissed for exhaustion.
        }
    }
    ~ process_payroll_worker(index + 1, day)
- else:
    Payroll cycle complete.
}

== function review_contract(index: int, day: int) => void ==
~ temp merc: Mercenary = mercenaries[index]
{ if merc.alive:
    { if merc.morale >= 6 && treasury >= merc.wage * 2:
        ~ treasury = treasury - merc.wage
        ~ mercenaries[index].contract_days_left = 2
        ~ mercenaries[index].morale = merc.morale + 1
        {merc.name} renews contract on day {day}. New term: 2 days.
    - else:
        ~ mercenaries[index].alive = false
        ~ terminated_contracts = terminated_contracts + 1
        {merc.name}'s contract ends without renewal.
    }
}

== function apply_casualties(day: int, index: int) => void ==
{ if index < LEN(casualty_events):
    ~ temp event: CasualtyEvent = casualty_events[index]
    ~ temp index_in_roster: int = merc_index[event.name]
    ~ temp merc: Mercenary = mercenaries[index_in_roster]
    ~ temp casualty_active: bool = event.day == day && merc.alive
    { if casualty_active:
        ~ process_casualty_event(index_in_roster, event, merc)
    - else:
        ~ casualties = casualties
    }
    ~ apply_casualties(day, index + 1)
- else:
    Casualty event pass complete.
}

== function process_casualty_event(index_in_roster: int, event: CasualtyEvent, merc: Mercenary) => void ==
-- Casualty event for {event.name}: {event.note}
~ mercenaries[index_in_roster].wounds = merc.wounds + event.severity
~ mercenaries[index_in_roster].morale = merc.morale - 2
{ if mercenaries[index_in_roster].wounds >= 10:
    ~ mercenaries[index_in_roster].alive = false
    ~ casualties = casualties + 1
    {event.name} is removed from duty (wounds {mercenaries[index_in_roster].wounds}).
- else:
    {event.name} wounded, morale now {mercenaries[index_in_roster].morale}.
}
{ if mercenaries[index_in_roster].morale <= 0:
    ~ mercenaries[index_in_roster].alive = false
    ~ casualties = casualties + 1
    {event.name} collapses from sustained stress and leaves.
- else:
    ~ casualties = casualties
}

== function evaluate_promotions() => void ==
-- Promotion review --
~ evaluate_promotion_worker(0)

== function evaluate_promotion_worker(index: int) => void ==
{ if index < LEN(mercenaries):
    ~ temp merc: Mercenary = mercenaries[index]
    { if merc.alive:
        { if merc.rank_tier == 0 && merc.morale >= 8 && merc.wounds <= 4:
            ~ mercenaries[index].rank_tier = 1
            ~ mercenaries[index].wage = merc.wage + 2
            ~ mercenaries[index].morale = merc.morale + 1
            ~ promotions = promotions + 1
            {merc.name} promoted to tier 1.
        - else:
            { if merc.rank_tier == 1 && merc.morale >= 11 && merc.wounds == 0:
                ~ mercenaries[index].rank_tier = 2
                ~ mercenaries[index].wage = merc.wage + 3
                ~ mercenaries[index].morale = merc.morale + 1
                ~ promotions = promotions + 1
                {merc.name} promoted to tier 2.
            }
        }
    }
    ~ evaluate_promotion_worker(index + 1)
- else:
    Promotion pass complete.
}

== function calculate_readiness() => void ==
~ readiness_score = 0
~ calculate_readiness_worker(0)

== function calculate_readiness_worker(index: int) => void ==
{ if index < LEN(mercenaries):
    ~ temp merc: Mercenary = mercenaries[index]
    { if merc.alive:
        ~ readiness_score = readiness_score + 10
        ~ readiness_score = readiness_score + merc.morale
        ~ readiness_score = readiness_score - merc.wounds
        { if merc.rank_tier == 1:
            ~ readiness_score = readiness_score + 2
        - else:
            { if merc.rank_tier == 2:
                ~ readiness_score = readiness_score + 5
            }
        }
    }
    ~ calculate_readiness_worker(index + 1)
- else:
    Readiness calculation complete.
}

== function print_readiness_status(day: int) => void ==
-- Readiness check for day {day} --
Company readiness: {readiness_score}
{ if readiness_score >= 45:
    Readiness status: operational.
- else:
    Readiness status: under readiness watch.
}

== function print_roster(label: string) => void ==
{label}
~ print_roster_worker(0)

== function print_roster_worker(index: int) => void ==
{ if index < LEN(mercenaries):
    ~ temp merc: Mercenary = mercenaries[index]
    { if merc.alive:
        {merc.name} | role {merc.role} | tier {merc.rank_tier} | morale {merc.morale} | wounds {merc.wounds} | contract {merc.contract_days_left} | wage {merc.wage}
    - else:
        {merc.name} | inactive.
    }
    ~ print_roster_worker(index + 1)
- else:
    Roster line complete.
}

== function final_state_report() => void ==
Campaign summary:
Treasury remaining: {treasury}
Casualties recorded: {casualties}
Payroll shortfalls: {payroll_shortfalls}
Contracts ended without renewal: {terminated_contracts}
Promotions awarded: {promotions}
~ calculate_readiness()
Final readiness score: {readiness_score}
~ print_roster_worker(0)
