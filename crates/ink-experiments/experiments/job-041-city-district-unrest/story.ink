=== module game ===

STRUCT District {
    name: string
    unrest: int
    security: int
    supply: int
}

STRUCT PolicyEvent {
    title: string
    district_index: int
    unrest_delta: int
    security_delta: int
    supply_delta: int
}

VAR districts: District[] = [
    %District{
        name: "Canalside Quarter",
        unrest: 3,
        security: 5,
        supply: 7
    },
    %District{
        name: "Iron Market",
        unrest: 6,
        security: 4,
        supply: 5
    },
    %District{
        name: "North Bastion",
        unrest: 2,
        security: 7,
        supply: 4
    }
]

VAR policies: PolicyEvent[] = [
    %PolicyEvent{
        title: "Patrol Sweep",
        district_index: 0,
        unrest_delta: -1,
        security_delta: 2,
        supply_delta: 0
    },
    %PolicyEvent{
        title: "Food Subsidy",
        district_index: 1,
        unrest_delta: -3,
        security_delta: 0,
        supply_delta: 2
    },
    %PolicyEvent{
        title: "Search Quotas",
        district_index: 2,
        unrest_delta: 2,
        security_delta: 1,
        supply_delta: -1
    },
    %PolicyEvent{
        title: "Border Tax Reform",
        district_index: 1,
        unrest_delta: 1,
        security_delta: -1,
        supply_delta: -2
    },
    %PolicyEvent{
        title: "Harbor Quiet Hours",
        district_index: 0,
        unrest_delta: 1,
        security_delta: 1,
        supply_delta: -1
    },
    %PolicyEvent{
        title: "Garrison Rotation",
        district_index: 2,
        unrest_delta: -2,
        security_delta: 2,
        supply_delta: 1
    }
]

VAR unrest_crises: int = 0

== main ==
City governance turn begins.
-- Initial district status --
~ print_all_districts()
~ process_policy_events(0)
-- Final district status --
~ print_all_districts()
{ if unrest_crises == 0:
    Unrest is stable across all districts this cycle.
    - else:
    { unrest_crises } districts crossed the unrest alarm line.
}
End of turn.
-> DONE

== function process_policy_events(index: int) => void ==
{ if index >= LEN(policies):
    ~ return
}
~ temp policy: PolicyEvent = policies[index]
Policy event: {policy.title}
~ apply_policy(index)
-- Post-event status --
~ print_all_districts()
~ process_policy_events(index + 1)

== function apply_policy(index: int) => void ==
~ temp policy: PolicyEvent = policies[index]
~ temp district_index: int = policy.district_index
~ temp district: District = districts[district_index]
~ temp old_unrest: int = district.unrest
~ temp old_security: int = district.security
~ temp old_supply: int = district.supply

~ temp next_unrest: int = old_unrest + policy.unrest_delta
{ if next_unrest < 0:
    ~ next_unrest = 0
    - else:
    { if next_unrest > 10:
        ~ next_unrest = 10
    }
}

~ temp next_security: int = old_security + policy.security_delta
{ if next_security < 0:
    ~ next_security = 0
    - else:
    { if next_security > 10:
        ~ next_security = 10
    }
}

~ temp next_supply: int = old_supply + policy.supply_delta
{ if next_supply < 0:
    ~ next_supply = 0
    - else:
    { if next_supply > 10:
        ~ next_supply = 10
    }
}

~ districts[district_index].unrest = next_unrest
~ districts[district_index].security = next_security
~ districts[district_index].supply = next_supply

{ if old_unrest < 8 and next_unrest >= 8:
    District alarm at {district.name}: unrest reached critical.
    ~ unrest_crises = unrest_crises + 1
    - else:
    { if next_unrest < 4:
        {district.name} remains calm.
    - else:
        {district.name} unrest shifted to {next_unrest}.
    }
}

{ if next_security < 3:
    Guard pressure low in {district.name}.
    - else:
    Guard pressure holds in {district.name}.
}

{ if next_supply <= 2:
    Supply strain in {district.name}, ration watch needed.
    - else:
    Supply flow is sufficient in {district.name}.
}

== function print_all_districts() => void ==
~ print_district(0)
~ print_district(1)
~ print_district(2)

== function print_district(index: int) => void ==
~ temp district: District = districts[index]
{ district.name }: Unrest {district.unrest}, Security {district.security}, Supply {district.supply}
