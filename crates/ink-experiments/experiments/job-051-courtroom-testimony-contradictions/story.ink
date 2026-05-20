=== module game ===

STRUCT Testimony {
    witness: string
    claim_id: string
    claimed: bool
    statement: string
    status: string
}

VAR testimonies: Testimony[] = [
    %Testimony{
        witness: "Lark",
        claim_id: "entry_broken",
        claimed: true,
        statement: "The east entry hatch showed forced seam marks.",
        status: "pending"
    },
    %Testimony{
        witness: "Mara",
        claim_id: "alibi_cleared",
        claimed: false,
        statement: "Captain Sern was never seen near the vault at night.",
        status: "pending"
    },
    %Testimony{
        witness: "Dain",
        claim_id: "ledger_tamper",
        claimed: true,
        statement: "The ledger was altered before the hearing.",
        status: "pending"
    },
    %Testimony{
        witness: "Orrin",
        claim_id: "security_grid_live",
        claimed: true,
        statement: "The alarm grid was fully live during the intrusion.",
        status: "pending"
    },
    %Testimony{
        witness: "Nessa",
        claim_id: "guard_swap",
        claimed: false,
        statement: "No guard swap occurred on the west stairwell.",
        status: "pending"
    },
    %Testimony{
        witness: "Voss",
        claim_id: "camera_outage",
        claimed: true,
        statement: "A camera outage covered a full fifteen minutes.",
        status: "pending"
    }
]

VAR evidence_flags: Dict<string, bool> = %{
    "entry_broken": false,
    "alibi_cleared": true,
    "ledger_tamper": true,
    "security_grid_live": true,
    "guard_swap": false,
    "camera_outage": false
}

VAR evidence_confidence: Dict<string, int> = %{
    "entry_broken": 9,
    "alibi_cleared": 8,
    "ledger_tamper": 7,
    "security_grid_live": 4,
    "guard_swap": 6,
    "camera_outage": 2
}

VAR evidence_sources: Dict<string, string> = %{
    "entry_broken": "forensic inspection",
    "alibi_cleared": "shift and badge audit",
    "ledger_tamper": "ledger audit trail",
    "security_grid_live": "floor alert summary",
    "guard_swap": "security sign-off sheets",
    "camera_outage": "single anonymous tip"
}

VAR witness_credibility: Dict<string, int> = %{
    "Lark": 9,
    "Mara": 7,
    "Dain": 8,
    "Orrin": 4,
    "Nessa": 9,
    "Voss": 3
}

VAR total_testimonies: int = 0
VAR contradictions: int = 0
VAR high_impact_contradictions: int = 0
VAR supported: int = 0
VAR unresolved: int = 0

== main ==
Court hearing records are opened.
~ list_evidence_flags(0)
~ record_all_testimonies(0)
~ print_transcript_status(0)
~ expose_results()
-> DONE

== function list_evidence_flags(index: int) => void ==
Evidence reference set:
~ print_evidence_row(0)

== function print_evidence_row(index: int) => void ==
{ if index < LEN(testimonies):
    ~ temp claim_id: string = testimonies[index].claim_id
    ~ temp flag_state: bool = evidence_flags[claim_id]
    ~ temp source: string = evidence_sources[claim_id]
    { claim_id }: {flag_state} ({source})
    ~ print_evidence_row(index + 1)
- else:
    End evidence flags.
}

== function record_all_testimonies(index: int) => void ==
{ if index < LEN(testimonies):
    ~ total_testimonies = total_testimonies + 1
    ~ process_testimony(index)
    ~ record_all_testimonies(index + 1)
- else:
    All testimonies recorded.
}

== function process_testimony(index: int) => void ==
~ temp witness: string = testimonies[index].witness
~ temp claim_id: string = testimonies[index].claim_id
~ temp claimed: bool = testimonies[index].claimed
~ temp statement: string = testimonies[index].statement
~ temp recorded_state: bool = evidence_flags[claim_id]
~ temp confidence: int = evidence_confidence[claim_id]
~ temp source: string = evidence_sources[claim_id]
~ temp witness_score: int = witness_credibility[witness]

-- {witness}: "{statement}" --
Evidence source "{source}" confidence {confidence}.
{ if confidence >= 5:
    { if claimed == recorded_state:
        ~ supported = supported + 1
        ~ testimonies[index].status = "supported"
        Claim aligns with evidence.
    - else:
        ~ contradictions = contradictions + 1
        ~ testimonies[index].status = "contradiction"
        Contradiction found.
        { if witness_score >= 8:
            ~ high_impact_contradictions = high_impact_contradictions + 1
            High-credibility contradiction.
        }
    }
- else:
    ~ unresolved = unresolved + 1
    ~ testimonies[index].status = "uncertain"
    Evidence confidence too weak for a firm ruling.
}

== function print_transcript_status(index: int) => void ==
Transcript status:
~ print_transcript_line(0)

== function print_transcript_line(index: int) => void ==
{ if index < LEN(testimonies):
    ~ temp t: Testimony = testimonies[index]
    { if t.status == "supported":
        {index + 1}) {t.witness}: {t.statement}: Supported.
    - else:
        { if t.status == "contradiction":
            {index + 1}) {t.witness}: {t.statement}: Contradicted.
        - else:
            {index + 1}) {t.witness}: {t.statement}: Uncertain.
        }
    }
    ~ print_transcript_line(index + 1)
- else:
    End transcript summary.
}

== function expose_results() => void ==
-- Courtroom analysis --
Recorded testimonies: {total_testimonies}
Supported claims: {supported}
Uncertain claims: {unresolved}
Contradictions: {contradictions}
High-impact contradictions: {high_impact_contradictions}

{ if contradictions == 0:
    No direct contradictions; prosecution case is internally consistent.
- else:
    Contradictions block a clean verdict.
    { if high_impact_contradictions >= 1:
        Rework the case: trusted witnesses differ from evidence on core facts.
    - else:
        Inconsistencies are present but mostly from lower-credibility statements.
    }
}
