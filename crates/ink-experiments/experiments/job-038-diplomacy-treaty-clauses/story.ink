=== module game ===

STRUCT TreatyClause {
    title: string
    proposer: string
    requested_concession: int
    base_accept_threshold: int
    concession_threshold: int
    sign_bonus: int
    rejection_penalty: int
    status: string
    signed: bool
    rejected: bool
    conceded: bool
}

VAR clauses: TreatyClause[] = [
    %TreatyClause{
        title: "Demilitarized Corridor",
        proposer: "North Federation",
        requested_concession: 1,
        base_accept_threshold: 35,
        concession_threshold: 30,
        sign_bonus: 4,
        rejection_penalty: 6,
        status: "pending",
        signed: false,
        rejected: false,
        conceded: false
    },
    %TreatyClause{
        title: "Joint Harbor Customs",
        proposer: "Coastal Republic",
        requested_concession: 1,
        base_accept_threshold: 52,
        concession_threshold: 44,
        sign_bonus: 7,
        rejection_penalty: 9,
        status: "pending",
        signed: false,
        rejected: false,
        conceded: false
    },
    %TreatyClause{
        title: "Mutual Rail Passage",
        proposer: "Mountain League",
        requested_concession: 2,
        base_accept_threshold: 44,
        concession_threshold: 36,
        sign_bonus: 5,
        rejection_penalty: 10,
        status: "pending",
        signed: false,
        rejected: false,
        conceded: false
    },
    %TreatyClause{
        title: "Emergency Aid Fund",
        proposer: "Coastal Republic",
        requested_concession: 2,
        base_accept_threshold: 60,
        concession_threshold: 48,
        sign_bonus: 6,
        rejection_penalty: 12,
        status: "pending",
        signed: false,
        rejected: false,
        conceded: false
    },
    %TreatyClause{
        title: "Armed Patrol Rotation",
        proposer: "North Federation",
        requested_concession: 1,
        base_accept_threshold: 48,
        concession_threshold: 40,
        sign_bonus: 3,
        rejection_penalty: 7,
        status: "pending",
        signed: false,
        rejected: false,
        conceded: false
    }
]

VAR relations_with_partners: int = 40
VAR concession_tokens: int = 2
VAR signed_count: int = 0
VAR rejected_count: int = 0
VAR tension_index: int = 0
VAR final_signed: bool = false

== main ==
A treaty table opens between three neighboring states.
~ print_clause_briefing()
~ negotiate_all_clauses(0)
~ print_clause_record()
~ finalize_treaty()
-> DONE

== function print_clause_briefing() => void ==
Proposed clauses:
~ print_clause_header(0)

== function print_clause_header(index: int) => void ==
{ if index < LEN(clauses):
    ~ temp clause: TreatyClause = clauses[index]
    {index + 1}. {clause.title} from {clause.proposer}
    Requested concession: {clause.requested_concession}, base acceptance score {clause.base_accept_threshold}, concession cutoff {clause.concession_threshold}.
    ~ print_clause_header(index + 1)
- else:
    End of briefing.
}

== function negotiate_all_clauses(index: int) => void ==
{ if index >= LEN(clauses):
    Negotiation phase complete.
- else:
    ~ evaluate_clause(index)
    ~ negotiate_all_clauses(index + 1)
}

== function evaluate_clause(index: int) => void ==
~ temp clause: TreatyClause = clauses[index]

-- Clause {index + 1}: {clause.title} --
{clause.proposer} requests concession level {clause.requested_concession}.

{ if relations_with_partners >= clause.base_accept_threshold:
    ~ clauses[index].signed = true
    ~ clauses[index].status = "accepted"
    ~ signed_count = signed_count + 1
    ~ relations_with_partners = relations_with_partners + clause.sign_bonus
    ~ tension_index = tension_index - 1
    Signed as proposed.
- else:
    { if concession_tokens >= clause.requested_concession && relations_with_partners >= clause.concession_threshold:
        ~ clauses[index].signed = true
        ~ clauses[index].conceded = true
        ~ clauses[index].status = "accepted after concession"
        ~ temp concession_cost: int = clause.requested_concession
        ~ concession_tokens = concession_tokens - concession_cost
        ~ signed_count = signed_count + 1
        ~ relations_with_partners = relations_with_partners + clause.sign_bonus - concession_cost
        ~ tension_index = tension_index + 0
        Concession used for this clause.
    - else:
        ~ clauses[index].rejected = true
        ~ clauses[index].status = "rejected"
        ~ rejected_count = rejected_count + 1
        ~ relations_with_partners = relations_with_partners - clause.rejection_penalty
        ~ tension_index = tension_index + 2
        Clause rejected under current relations.
    }
}

Current relations: {relations_with_partners}. Concession tokens left: {concession_tokens}. Tension index: {tension_index}.

== function print_clause_record() => void ==
Negotiation ledger:
~ print_clause_status(0)
Relations final before ruling: {relations_with_partners}
Concessions remaining: {concession_tokens}
Signed clauses: {signed_count}
Rejected clauses: {rejected_count}

== function print_clause_status(index: int) => void ==
{ if index < LEN(clauses):
    ~ temp clause: TreatyClause = clauses[index]
    {index + 1}. {clause.title}: {clause.status}
    ~ print_clause_status(index + 1)
- else:
    End ledger.
}

== function finalize_treaty() => void ==
-- Treaty ruling --
{ if signed_count >= 4 && relations_with_partners >= 45 && tension_index <= 2:
    ~ final_signed = true
    Treaty accepted. All major clauses pass.
- else:
    { if signed_count >= 3 && relations_with_partners >= 40 && tension_index <= 3:
        ~ final_signed = true
        Treaty accepted with reservations and side letter.
    - else:
        Treaty collapsed.
        Final_signed remains false.
    }
}

{ if final_signed:
    The conference hall records a successful accord.
- else:
    Diplomatic ties cool as talks end without a full signature.
}
