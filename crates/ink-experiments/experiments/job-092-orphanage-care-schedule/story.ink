=== module game ===

STRUCT Caregiver {
    name: string
    skill: int
    stamina: int
    morale: int
    assigned: int
}

STRUCT Child {
    name: string
    need: int
    health: int
    morale: int
    missed_care: int
}

STRUCT CareAssignment {
    round: int
    caregiver: int
    child: int
    effort: int
    warmth: int
}

VAR round_labels: string[] = [
    "Morning",
    "Midday",
    "Night"
]

VAR caregivers: Caregiver[] = [
    %Caregiver{name: "Elara", skill: 6, stamina: 9, morale: 8, assigned: 0},
    %Caregiver{name: "Tomas", skill: 5, stamina: 8, morale: 7, assigned: 0},
    %Caregiver{name: "Nia", skill: 4, stamina: 7, morale: 9, assigned: 0}
]

VAR children: Child[] = [
    %Child{name: "Ivo", need: 6, health: 9, morale: 7, missed_care: 0},
    %Child{name: "Lark", need: 5, health: 8, morale: 8, missed_care: 0},
    %Child{name: "Mina", need: 7, health: 9, morale: 6, missed_care: 0},
    %Child{name: "Puck", need: 4, health: 7, morale: 7, missed_care: 0}
]

VAR assignments: CareAssignment[] = [
    %CareAssignment{round: 0, caregiver: 0, child: 0, effort: 2, warmth: 2},
    %CareAssignment{round: 0, caregiver: 1, child: 2, effort: 1, warmth: 1},
    %CareAssignment{round: 0, caregiver: 2, child: 1, effort: 1, warmth: 2},
    %CareAssignment{round: 1, caregiver: 0, child: 2, effort: 2, warmth: 1},
    %CareAssignment{round: 1, caregiver: 1, child: 3, effort: 1, warmth: 1},
    %CareAssignment{round: 1, caregiver: 2, child: 1, effort: 0, warmth: 2},
    %CareAssignment{round: 2, caregiver: 0, child: 0, effort: 1, warmth: 1},
    %CareAssignment{round: 2, caregiver: 1, child: 2, effort: 0, warmth: 1},
    %CareAssignment{round: 2, caregiver: 2, child: 3, effort: 2, warmth: 2}
]

VAR cared_this_round: Dict<string, bool> = %{
    "Ivo": false,
    "Lark": false,
    "Mina": false,
    "Puck": false
}

VAR total_missed_rounds: int = 0
VAR total_under_care: int = 0
VAR escalation_events: int = 0
VAR critical_children: int = 0
VAR total_child_health: int = 0
VAR total_child_morale: int = 0

== main ==
Orphanage care schedule begins.
~ run_rounds(0)
~ print_final_summary()
-> DONE

== function run_rounds(round_index: int) => void ==
{ if round_index >= LEN(round_labels):
    All care rounds completed.
- else:
    ~ temp round_name: string = round_labels[round_index]
    Round: {round_name}
    ~ reset_round_flags(0)
    ~ run_round_assignments(round_index, 0)
    ~ apply_missed_care(0)
    ~ print_round_children(0)
    ~ run_rounds(round_index + 1)
}

== function reset_round_flags(index: int) => void ==
{ if index >= LEN(children):
    ~ return
- else:
    ~ temp child: Child = children[index]
    ~ cared_this_round[child.name] = false
    ~ reset_round_flags(index + 1)
}

== function run_round_assignments(round_index: int, assignment_index: int) => void ==
{ if assignment_index >= LEN(assignments):
    ~ return
- else:
    ~ temp assignment: CareAssignment = assignments[assignment_index]
    { if assignment.round == round_index:
        ~ apply_assignment(assignment_index)
    }
    ~ run_round_assignments(round_index, assignment_index + 1)
}

== function apply_assignment(assignment_index: int) => void ==
~ temp assignment: CareAssignment = assignments[assignment_index]
~ temp caregiver: Caregiver = caregivers[assignment.caregiver]
~ temp child: Child = children[assignment.child]
~ temp drag: int = fatigue_drag(caregiver.stamina)
~ temp budget: int = caregiver.skill + assignment.effort - drag
~ temp required: int = child.need

Caregiver {caregiver.name} tends {child.name}. Need {required}, budget {budget}.
{ if budget >= required:
    Full care delivered.
    ~ children[assignment.child].health = clamp_child_health(child.health + 2)
    ~ children[assignment.child].morale = clamp_child_morale(child.morale + assignment.warmth)
    ~ caregivers[assignment.caregiver].morale = clamp_caregiver_morale(caregiver.morale + 1)
- else:
    { if budget >= required - 1:
        Partial care delivered.
        ~ children[assignment.child].health = clamp_child_health(child.health + 1)
        ~ children[assignment.child].morale = clamp_child_morale(child.morale + (assignment.warmth - 1))
    - else:
        Care demand exceeds current capacity.
        ~ children[assignment.child].health = clamp_child_health(child.health - 1)
        ~ children[assignment.child].morale = clamp_child_morale(child.morale - 2)
        ~ children[assignment.child].missed_care = child.missed_care + 1
        ~ caregivers[assignment.caregiver].morale = clamp_caregiver_morale(caregiver.morale - 1)
        ~ total_under_care = total_under_care + 1
    }
}

~ caregivers[assignment.caregiver].stamina = clamp_stamina(caregiver.stamina - 2)
~ caregivers[assignment.caregiver].assigned = caregiver.assigned + 1
~ cared_this_round[child.name] = true

== function apply_missed_care(child_index: int) => void ==
{ if child_index >= LEN(children):
    ~ return
- else:
    ~ temp child: Child = children[child_index]
    ~ temp cared: bool = cared_this_round[child.name]
    { if !cared:
        Missed care: {child.name} had no caregiver this round.
        ~ children[child_index].health = clamp_child_health(child.health - 2)
        ~ children[child_index].morale = clamp_child_morale(child.morale - 3)
        ~ children[child_index].missed_care = child.missed_care + 1
        ~ total_missed_rounds = total_missed_rounds + 1
        { if child.missed_care + 1 >= 2:
            Repeated miss causes extra health strain for {child.name}.
            ~ children[child_index].health = clamp_child_health(children[child_index].health - 1)
            ~ escalation_events = escalation_events + 1
        }
    }
    ~ apply_missed_care(child_index + 1)
}

== function print_round_children(index: int) => void ==
{ if index >= LEN(children):
    ~ return
- else:
    ~ temp child: Child = children[index]
    {child.name} | health {child.health} | morale {child.morale} | missed {child.missed_care}
    ~ print_round_children(index + 1)
}

== function print_final_summary() => void ==
Orphanage schedule summary.
Missed rounds: {total_missed_rounds}
Under-care events: {total_under_care}
Escalation events: {escalation_events}
~ print_caregiver_summary(0)
~ critical_children = 0
~ total_child_health = 0
~ total_child_morale = 0
~ summarize_children(0)
Average child health: {total_child_health / LEN(children)}
Average child morale: {total_child_morale / LEN(children)}
Critical children: {critical_children}
~ print_outcome()

== function print_caregiver_summary(index: int) => void ==
{ if index >= LEN(caregivers):
    ~ return
- else:
    ~ temp caregiver: Caregiver = caregivers[index]
    {caregiver.name} | assignments {caregiver.assigned} | stamina {caregiver.stamina} | morale {caregiver.morale}
    ~ print_caregiver_summary(index + 1)
}

== function summarize_children(index: int) => void ==
{ if index >= LEN(children):
    ~ return
- else:
    ~ temp child: Child = children[index]
    ~ total_child_health = total_child_health + child.health
    ~ total_child_morale = total_child_morale + child.morale
    { if child.health <= 4 || child.morale <= 3:
        ~ critical_children = critical_children + 1
    }
    ~ summarize_children(index + 1)
}

== function print_outcome() => void ==
{ if critical_children > 0 || total_missed_rounds >= 4:
    Outcome: care schedule fails and emergency staffing is required.
- else:
    { if total_under_care > 1 || escalation_events > 0:
        Outcome: care schedule holds but child needs outpace staffing.
    - else:
        Outcome: care schedule remains stable for this day.
    }
}

== function fatigue_drag(stamina: int) => int ==
{ if stamina >= 7:
    ~ return 0
- else:
    { if stamina >= 4:
        ~ return 1
    - else:
        ~ return 2
    }
}

== function clamp_child_health(value: int) => int ==
{ if value < 0:
    ~ return 0
- else:
    { if value > 12:
        ~ return 12
    - else:
        ~ return value
    }
}

== function clamp_child_morale(value: int) => int ==
{ if value < 0:
    ~ return 0
- else:
    { if value > 12:
        ~ return 12
    - else:
        ~ return value
    }
}

== function clamp_caregiver_morale(value: int) => int ==
{ if value < 0:
    ~ return 0
- else:
    { if value > 12:
        ~ return 12
    - else:
        ~ return value
    }
}

== function clamp_stamina(value: int) => int ==
{ if value < 0:
    ~ return 0
- else:
    ~ return value
}
