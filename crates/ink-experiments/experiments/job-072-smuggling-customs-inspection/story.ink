=== module game ===

STRUCT Cargo {
    name: string
    concealment: int
    declared_value: int
    contraband: bool
    bribe_offer: int
}

VAR cargo_manifest: Cargo[] = [
    %Cargo{name: "Salted grain", concealment: 1, declared_value: 24, contraband: false, bribe_offer: 2},
    %Cargo{name: "Copper knives", concealment: 4, declared_value: 46, contraband: true, bribe_offer: 14},
    %Cargo{name: "Moonglass idol", concealment: 5, declared_value: 74, contraband: true, bribe_offer: 18},
    %Cargo{name: "Herbal tincture", concealment: 1, declared_value: 30, contraband: false, bribe_offer: 4},
    %Cargo{name: "Signal powder", concealment: 3, declared_value: 58, contraband: true, bribe_offer: 10}
]

VAR inspection_intensity: int = 3
VAR officer_integrity: int = 3
VAR smuggler_cash: int = 96
VAR inspection_checks: int = 0
VAR bribes_offered: int = 0
VAR bribes_accepted: int = 0
VAR seizures: int = 0
VAR seized_value: int = 0

== main ==
The customs lane narrows at dawn.
~ print_customs_roster()
~ inspect_manifest(0)
~ print_customs_report()
-> DONE

== function inspect_manifest(index: int) => void ==
{ if index >= LEN(cargo_manifest):
    ~ return
}

~ temp item: Cargo = cargo_manifest[index]
~ inspection_checks = inspection_checks + 1

Inspection pass for {item.name}.
~ temp pressure: int = suspicion_pressure(item)
~ temp limit: int = inspection_intensity * 4
Risk score {pressure}, inspection limit {limit}.
~ temp allowed_threshold: int = limit

{ if pressure <= allowed_threshold:
    ~ item_passes(item)
    ~ inspect_manifest(index + 1)
- else:
    { if attempt_bribe(item, pressure):
        ~ bribes_offered = bribes_offered + 1
        ~ bribe_accepted(item)
        ~ inspect_manifest(index + 1)
    - else:
        ~ bribes_offered = bribes_offered + 1
        ~ seize_cargo(item, pressure, limit)
        ~ inspect_manifest(index + 1)
    }
}

== function suspicion_pressure(item: Cargo) => int ==
{ if item.contraband:
    ~ return item.concealment * 2 + item.declared_value / 10 + 1
- else:
    ~ return item.concealment + item.declared_value / 12
}

== function attempt_bribe(item: Cargo, pressure: int) => bool ==
~ temp required_offer: int = pressure + inspection_intensity + officer_integrity + 1
{ if smuggler_cash < item.bribe_offer:
    ~ return false
- else:
    { if item.bribe_offer + 2 >= required_offer:
        ~ return true
    - else:
        ~ return false
    }
}

== function bribe_accepted(item: Cargo) => void ==
~ bribes_accepted = bribes_accepted + 1
~ smuggler_cash = smuggler_cash - item.bribe_offer
Bribe paid: {item.name} for {item.bribe_offer}.
Outcome: inspection diverted.

== function seize_cargo(item: Cargo, pressure: int, limit: int) => void ==
~ seizures = seizures + 1
~ temp full_seizure: bool = pressure >= limit + 4
{ if full_seizure:
    ~ seize_value(item.declared_value)
    Full seizure of {item.name}.
    {if item.contraband:
        Contraband removed.
    - else:
        Contraband charge failed; all declared property still held.
    }
- else:
    ~ seize_value(item.declared_value / 2)
    Partial seizure of {item.name}.
    {if item.contraband:
        Contraband warning.
    - else:
        Document inconsistency found.
    }
}

== function seize_value(value: int) => void ==
~ seized_value = seized_value + value
~ smuggler_cash = smuggler_cash - value
{ if smuggler_cash < 0:
    ~ smuggler_cash = 0
}
Seized value credited: {value}.

== function item_passes(item: Cargo) => void ==
Clearance granted for {item.name}.

== function print_customs_roster() => void ==
Inspection intensity: {intensity_name(inspection_intensity)}.
Officer integrity: {officer_integrity}
Current cash available: {smuggler_cash}

== function intensity_name(level: int) => string ==
{ if level >= 4:
    ~ return "high"
- else:
    { if level >= 2:
        ~ return "moderate"
    - else:
        ~ return "low"
    }
}

== function print_customs_report() => void ==
Inspection complete.
Total checks: {inspection_checks}
Bribe attempts: {bribes_offered}
Accepted bribes: {bribes_accepted}
Seizures: {seizures}
Total value confiscated: {seized_value}
Cash remaining for travel: {smuggler_cash}

{ if bribes_accepted == 0:
    No bribes passed this run.
- else:
    { if bribes_accepted == bribes_offered:
        Every suspicious check was resolved with a bribe.
    - else:
        Some checks were sealed by seizure and some by bribe.
    }
}
