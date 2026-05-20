=== module game ===
STRUCT Hireling {
    name: string
    daily_wage: int
    morale: int
    contract_days_left: int
    employed: bool
}

VAR treasury: int = 22
VAR simulation_days: int = 4
VAR hirelings: Hireling[] = [
    %Hireling{
        name: "Ari",
        daily_wage: 5,
        morale: 8,
        contract_days_left: 2,
        employed: true
    },
    %Hireling{
        name: "Bora",
        daily_wage: 7,
        morale: 6,
        contract_days_left: 3,
        employed: true
    }
]

== main ==
Hireling contract and wage simulation.
Day 1 begins with {treasury} gold.
~ run_day(1)
Final tally.
Treasury: {treasury}
~ print_all_hirelings()
-> DONE

== function run_day(day: int) => void ==
{ if day > simulation_days:
    Day {simulation_days} cycle complete.
- else:
    Day {day}
    ~ process_hirelings(0)
    Treasury after payroll: {treasury}
    ~ print_all_hirelings()
    ~ run_day(day + 1)
}

== function process_hirelings(index: int) => void ==
{ if index < LEN(hirelings):
    ~ apply_daily_wage(index)
    ~ process_hirelings(index + 1)
}

== function apply_daily_wage(index: int) => void ==
{ if !hirelings[index].employed:
    {hirelings[index].name} is not employed.
- else:
    { if treasury >= hirelings[index].daily_wage:
        Treasury pays {hirelings[index].daily_wage} to {hirelings[index].name}.
        ~ treasury = treasury - hirelings[index].daily_wage
        ~ hirelings[index].morale = hirelings[index].morale + 1
        {hirelings[index].name} remains motivated at morale {hirelings[index].morale}.
    - else:
        {hirelings[index].name} waits unpaid today.
        ~ hirelings[index].morale = hirelings[index].morale - 2
        { if hirelings[index].morale <= 0:
            {hirelings[index].name} leaves immediately from exhaustion.
            ~ hirelings[index].employed = false
        - else:
            {hirelings[index].name} morale drops to {hirelings[index].morale}.
        }
    }
    { if hirelings[index].employed:
        ~ hirelings[index].contract_days_left = hirelings[index].contract_days_left - 1
        { if hirelings[index].contract_days_left <= 0:
            ~ evaluate_contract_renewal(index)
        }
    }
}

== function evaluate_contract_renewal(index: int) => void ==
{ if hirelings[index].morale >= 7:
    { if treasury >= hirelings[index].daily_wage:
        ~ hirelings[index].contract_days_left = 2
        ~ treasury = treasury - hirelings[index].daily_wage
        ~ hirelings[index].morale = hirelings[index].morale - 1
        {hirelings[index].name} accepts contract renewal and serves {hirelings[index].contract_days_left} more days.
    - else:
        ~ hirelings[index].employed = false
        {hirelings[index].name} wants renewal, but payroll cannot fund it.
    }
- else:
    ~ hirelings[index].employed = false
    {hirelings[index].name} declines renewal due low morale.
}

== function print_all_hirelings() => void ==
~ print_hireling(0)

== function print_hireling(index: int) => void ==
{ if index < LEN(hirelings):
    { if hirelings[index].employed:
        {hirelings[index].name}: employed, morale {hirelings[index].morale}, contract_days {hirelings[index].contract_days_left}.
    - else:
        {hirelings[index].name}: contract ended.
    }
    ~ print_hireling(index + 1)
}
