=== module game ===

STRUCT EvidenceCard {
    id: string
    description: string
    significance: string
    collected: bool
}

VAR evidence_cards: EvidenceCard[] = [
    %EvidenceCard{
        id: "E-01",
        description: "Mud-caked boot print at the vault entrance.",
        significance: "Physical trail",
        collected: false
    },
    %EvidenceCard{
        id: "E-02",
        description: "Second set of identical prints beside the records room.",
        significance: "Print comparison",
        collected: false
    },
    %EvidenceCard{
        id: "E-03",
        description: "Ledger page showing a guard shift mismatch.",
        significance: "Operational access",
        collected: false
    },
    %EvidenceCard{
        id: "E-04",
        description: "Unsigned override token request from the south door.",
        significance: "Door control log",
        collected: false
    },
    %EvidenceCard{
        id: "E-05",
        description: "Dock manifest stamped with an internal clerk signature.",
        significance: "Route access",
        collected: false
    },
    %EvidenceCard{
        id: "E-06",
        description: "Intercepted note naming Maren as the gate opener.",
        significance: "Motive link",
        collected: false
    }
]

VAR deductions_boot_chain: bool = false
VAR deductions_access_window: bool = false
VAR deductions_route_leak: bool = false
VAR deductions_inside_help: bool = false
VAR deductions_case_ready: bool = false
VAR unlocked_deductions: int = 0

== main ==
Investigation begins at midnight.
~ collect_evidence(0)
~ collect_evidence(1)
~ collect_evidence(2)
~ collect_evidence(3)
~ collect_evidence(4)
~ collect_evidence(5)
~ summarize_deductions()
~ conclude_case()
-> DONE

== function collect_evidence(index: int) => void ==
~ temp item: EvidenceCard = evidence_cards[index]
-- Gathered {item.id} --
{item.description}
~ evidence_cards[index].collected = true
~ unlock_new_deductions()

== function unlock_new_deductions() => void ==
{ if !deductions_boot_chain && evidence_cards[0].collected && evidence_cards[1].collected:
    ~ deductions_boot_chain = true
    ~ unlocked_deductions = unlocked_deductions + 1
    Deduction unlocked: a single intruder used the same boots through the front and records routes.
}

{ if !deductions_access_window && evidence_cards[2].collected && evidence_cards[3].collected:
    ~ deductions_access_window = true
    ~ unlocked_deductions = unlocked_deductions + 1
    Deduction unlocked: guard-shift paperwork was altered during the intrusion window.
}

{ if !deductions_route_leak && evidence_cards[1].collected && evidence_cards[4].collected:
    ~ deductions_route_leak = true
    ~ unlocked_deductions = unlocked_deductions + 1
    Deduction unlocked: route manifest leaked from internal logistics.
}

{ if !deductions_inside_help && deductions_boot_chain && deductions_route_leak && evidence_cards[5].collected:
    ~ deductions_inside_help = true
    ~ unlocked_deductions = unlocked_deductions + 1
    Deduction unlocked: internal helper opened the gate with duplicated authorization.
}

{ if !deductions_case_ready && deductions_boot_chain && deductions_access_window && deductions_inside_help:
    ~ deductions_case_ready = true
    ~ unlocked_deductions = unlocked_deductions + 1
    Deduction unlocked: complete evidence chain forms a single responsible actor.
}

== function summarize_deductions() => void ==
Unlocked deductions: {unlocked_deductions}
~ print_chain_status()

== function print_chain_status() => void ==
-- Chain status --
{ if deductions_boot_chain:
    Boot-chain match established.
- else:
    Boot-chain link missing.
}
{ if deductions_access_window:
    Guard-window issue identified.
- else:
    Guard-window issue not established.
}
{ if deductions_route_leak:
    Route leak identified.
- else:
    Route leak not yet identified.
}
{ if deductions_inside_help:
    Inside support identified.
- else:
    Inside support unresolved.
}
{ if deductions_case_ready:
    Complete chain established.
- else:
    Case chain still incomplete.
}

== function conclude_case() => void ==
-- Final conclusion --
{ if deductions_case_ready:
    Deduction outcome: Maren the gate supervisor orchestrated the heist in concert with dock logistics.
- else:
    Deduction outcome: Evidence is still incomplete for a courtroom-ready conclusion.
}
