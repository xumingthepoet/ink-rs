=== module game ===

STRUCT BorderOfficer {
    name: string
    pass_threshold: int
    hold_threshold: int
    scrutiny: int
    bribe_cost: int
    corruption: int
}

STRUCT Document {
    title: string
    present: bool
    authenticity: int
}

VAR border_officers: BorderOfficer[] = [
    %BorderOfficer{
        name: "North Lantern Post",
        pass_threshold: 36,
        hold_threshold: 56,
        scrutiny: 8,
        bribe_cost: 12,
        corruption: 0
    },
    %BorderOfficer{
        name: "Ridge Ledger Gate",
        pass_threshold: 38,
        hold_threshold: 49,
        scrutiny: 18,
        bribe_cost: 14,
        corruption: 6
    },
    %BorderOfficer{
        name: "Frontier Annex",
        pass_threshold: 52,
        hold_threshold: 74,
        scrutiny: 22,
        bribe_cost: 9,
        corruption: 1
    }
]

VAR travel_documents: Document[] = [
    %Document{title: "Passport", present: true, authenticity: 95},
    %Document{title: "Transit Permit", present: false, authenticity: 0},
    %Document{title: "Cargo Ledger", present: true, authenticity: 78},
    %Document{title: "Health Exemption", present: true, authenticity: 83},
    %Document{title: "City Tax Stamp", present: true, authenticity: 92}
]

VAR traveler_reputation: int = 68
VAR nervousness: int = 6
VAR wallet_coins: int = 120
VAR suspicion_total: int = 0
VAR cleared_checks: int = 0
VAR holding_checks: int = 0
VAR denial_checks: int = 0

== main ==
A merchant caravan reaches the border in heavy rain.
~ suspicion_total = 0
~ cleared_checks = 0
~ holding_checks = 0
~ denial_checks = 0
~ wallet_coins = 120
~ inspect_documents(0, 0)
~ run_border_checks(0)
~ border_outcome()
-> DONE

== function inspect_documents(index: int, running: int) => void ==
{ if index >= LEN(travel_documents):
    ~ suspicion_total = running
    Documents ready for crossing.
- else:
    ~ temp doc: Document = travel_documents[index]
    { if doc.present == false:
        ~ inspect_documents(index + 1, running + 24)
    - else:
        { if doc.authenticity < 70:
            ~ inspect_documents(index + 1, running + 20)
        - else:
            { if doc.authenticity < 85:
                ~ inspect_documents(index + 1, running + 6)
            - else:
                ~ inspect_documents(index + 1, running)
            }
        }
    }
}

== function run_border_checks(index: int) => void ==
{ if index >= LEN(border_officers):
    ~ return
- else:
    ~ temp officer: BorderOfficer = border_officers[index]
    ~ temp base_risk: int = suspicion_total + trust_penalty() - officer.scrutiny
    ~ temp adjusted_risk: int = base_risk + nervousness
    { if adjusted_risk < 0:
        ~ adjusted_risk = 0
    }
    ~ checkpoint_report(index, adjusted_risk)
    ~ temp final_risk: int = process_bribe(index, adjusted_risk)
    ~ finalize_officer(index, final_risk)
    ~ run_border_checks(index + 1)
}

== function trust_penalty() => int ==
{ if traveler_reputation < 60:
    ~ return 10
- else:
    { if traveler_reputation < 70:
        ~ return 6
    - else:
        ~ return 2
    }
}

== function process_bribe(index: int, risk: int) => int ==
~ temp officer: BorderOfficer = border_officers[index]
{ if risk <= officer.pass_threshold:
    ~ return risk
- else:
    { if wallet_coins < officer.bribe_cost or officer.corruption <= 0:
        The official refuses extra incentives.
        ~ return risk + 4
    - else:
        ~ wallet_coins = wallet_coins - officer.bribe_cost
        A discreet envelope changes hands.
        ~ temp discounted: int = risk - (officer.corruption * 2)
        { if discounted < 0:
            ~ discounted = 0
        }
        ~ return discounted
    }
}

== function finalize_officer(index: int, risk: int) => void ==
~ temp officer: BorderOfficer = border_officers[index]
~ suspicion_total = suspicion_total + risk
{ if risk > officer.hold_threshold:
    ~ denial_checks = denial_checks + 1
    {officer.name} issues a formal denial.
- else:
    { if risk > officer.pass_threshold:
        ~ holding_checks = holding_checks + 1
        {officer.name} requests a review hold.
    - else:
        ~ cleared_checks = cleared_checks + 1
        Passage from {officer.name} is approved.
    }
}

== function checkpoint_report(index: int, risk: int) => void ==
The checkpoint {index + 1} measured risk is {risk}.

== function border_outcome() => void ==
Border review complete.
Passes granted: {cleared_checks}
Holds: {holding_checks}
Denials: {denial_checks}
Total suspicion score: {suspicion_total}
Wallet coins: {wallet_coins}
{ if denial_checks > 0:
    Full crossing is denied.
- else:
    { if holding_checks > 0:
        Crossing succeeds after review holds.
    - else:
        Crossing succeeds without delay.
    }
}
{ if holding_checks > 0:
    Review pressure at the border remains high.
- else:
    The crossing line settles into a calm flow.
}
