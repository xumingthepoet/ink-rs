=== module game ===

STRUCT CaseFile {
    title: string
    priority: int
    delay_days: int
    evidence_strength: int
    harm_index: int
}

VAR case_backlog: CaseFile[] = [
    %CaseFile{ title: "Canal Toll Embezzlement", priority: 4, delay_days: 7, evidence_strength: 8, harm_index: 6 },
    %CaseFile{ title: "Mill District Arson", priority: 5, delay_days: 10, evidence_strength: 9, harm_index: 9 },
    %CaseFile{ title: "Harvest Contract Fraud", priority: 3, delay_days: 5, evidence_strength: 5, harm_index: 4 },
    %CaseFile{ title: "Dockside Assault", priority: 4, delay_days: 11, evidence_strength: 4, harm_index: 8 },
    %CaseFile{ title: "Archive Seal Tampering", priority: 2, delay_days: 4, evidence_strength: 3, harm_index: 3 }
]

VAR urgencies: int[] = [0, 0, 0, 0, 0]
VAR delay_penalties: int[] = [0, 0, 0, 0, 0]
VAR verdicts: string[] = ["", "", "", "", ""]
VAR trust_deltas: int[] = [0, 0, 0, 0, 0]

VAR public_trust: int = 63
VAR backlog_pressure: int = 0
VAR resolved_cases: int = 0
VAR convictions: int = 0
VAR probations: int = 0
VAR dismissals: int = 0
VAR total_delay_penalty: int = 0
VAR restitution_points: int = 0

== main ==
Magistrate backlog review begins.
Opening public trust: {public_trust}
~ backlog_pressure = backlog_score(0)
Opening backlog pressure: {backlog_pressure}
~ process_cases(0)
~ print_case_ledger(0)
~ print_summary()
-> DONE

== function backlog_score(index: int) => int ==
{ if index >= LEN(case_backlog):
    ~ return 0
- else:
    ~ temp case_file: CaseFile = case_backlog[index]
    ~ temp local_pressure: int = case_file.priority * 2 + case_file.delay_days
    ~ return local_pressure + backlog_score(index + 1)
}

== function process_cases(index: int) => void ==
{ if index >= LEN(case_backlog):
    Cases processed for the current docket.
- else:
    ~ temp case_file: CaseFile = case_backlog[index]
    ~ temp urgency: int = case_file.priority * 3 + case_file.delay_days
    ~ temp delay_penalty: int = delay_penalty_for(case_file.delay_days, case_file.priority)
    ~ temp score: int = case_file.evidence_strength + case_file.priority - case_file.harm_index
    ~ temp verdict: string = verdict_for(score, delay_penalty)
    ~ temp trust_delta: int = trust_shift(score, delay_penalty, case_file.harm_index)

    ~ urgencies[index] = urgency
    ~ delay_penalties[index] = delay_penalty
    ~ verdicts[index] = verdict
    ~ trust_deltas[index] = trust_delta

    ~ total_delay_penalty = total_delay_penalty + delay_penalty
    ~ public_trust = clamp_trust(public_trust + trust_delta)
    ~ resolved_cases = resolved_cases + 1
    ~ backlog_pressure = clamp_floor(backlog_pressure - urgency)

    { if verdict == "Conviction":
        ~ convictions = convictions + 1
        ~ restitution_points = restitution_points + case_file.harm_index * 2
    - else:
        { if verdict == "Probation":
            ~ probations = probations + 1
            ~ restitution_points = restitution_points + case_file.harm_index
        - else:
            ~ dismissals = dismissals + 1
        }
    }

    Case {index + 1}: {case_file.title}
    Priority {case_file.priority}, delay {case_file.delay_days} days.
    Urgency score: {urgency}
    Delay penalty: {delay_penalty}
    Verdict: {verdict}
    Trust shift: {trust_delta}
    Public trust now: {public_trust}
    Remaining backlog pressure: {backlog_pressure}

    ~ process_cases(index + 1)
}

== function delay_penalty_for(delay_days: int, priority: int) => int ==
{ if delay_days <= 3:
    ~ return 0
- else:
    ~ temp late_days: int = delay_days - 3
    ~ return late_days * (priority + 1)
}

== function verdict_for(score: int, delay_penalty: int) => string ==
{ if score >= 6:
    ~ return "Conviction"
- else:
    { if score >= 2 and delay_penalty <= 12:
        ~ return "Probation"
    - else:
        ~ return "Dismissal"
    }
}

== function trust_shift(score: int, delay_penalty: int, harm_index: int) => int ==
~ temp shift: int = 0
{ if score >= 6:
    ~ shift = 4
- else:
    { if score >= 2 and delay_penalty <= 12:
        ~ shift = 1
    - else:
        ~ shift = -3
    }
}
~ shift = shift - delay_penalty / 4
{ if harm_index >= 8 and score < 2:
    ~ shift = shift - 2
}
~ return shift

== function clamp_trust(value: int) => int ==
{ if value < 0:
    ~ return 0
- else:
    { if value > 100:
        ~ return 100
    - else:
        ~ return value
    }
}

== function clamp_floor(value: int) => int ==
{ if value < 0:
    ~ return 0
- else:
    ~ return value
}

== function print_case_ledger(index: int) => void ==
{ if index == 0:
    Case ledger:
}
{ if index >= LEN(case_backlog):
    ~ return
- else:
    ~ temp case_file: CaseFile = case_backlog[index]
    {case_file.title}: urgency {urgencies[index]}, penalty {delay_penalties[index]}, verdict {verdicts[index]}, trust {trust_deltas[index]}
    ~ print_case_ledger(index + 1)
}

== function print_summary() => void ==
Final magistrate summary:
Cases resolved: {resolved_cases}
Convictions: {convictions}
Probations: {probations}
Dismissals: {dismissals}
Total delay penalty: {total_delay_penalty}
Restitution points: {restitution_points}
Public trust closing value: {public_trust}
{ if public_trust >= 70 and total_delay_penalty <= 25:
    Court confidence rises across the district.
- else:
    { if public_trust >= 55:
        Court remains credible, though backlog reform is required.
    - else:
        Court legitimacy slips; emergency reforms are demanded.
    }
}
