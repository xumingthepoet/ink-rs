=== module quests ===

ENUM QuestState { Hidden Active Complete }

CONST QUEST_MAIN: int = 1
CONST QUEST_SIDE: int = 2
CONST OBJ_MAIN_REN: int = 101
CONST OBJ_MAIN_GATE: int = 102
CONST OBJ_MAIN_BATTLE: int = 103
CONST OBJ_SIDE_MOONLEAF: int = 201

VAR quest_state: Dict<int, QuestState> = %{1: QuestState.Hidden, 2: QuestState.Hidden}
VAR objective_progress: Dict<int, int> = %{}

== function accept_main() => void ==
~ quest_state[QUEST_MAIN] = QuestState.Active
~ objective_progress[OBJ_MAIN_REN] = 0
~ objective_progress[OBJ_MAIN_GATE] = 0
~ objective_progress[OBJ_MAIN_BATTLE] = 0

== function accept_side() => void ==
~ quest_state[QUEST_SIDE] = QuestState.Active
~ objective_progress[OBJ_SIDE_MOONLEAF] = 0

== function set_objective(objective_id: int, progress: int) => void ==
~ objective_progress[objective_id] = progress

== function quest_state_value(quest_id: int) => QuestState ==
~ return quest_state[quest_id]

== function objective_value(objective_id: int) => int ==
{ if DICT_HAS(objective_progress, objective_id):
    ~ return objective_progress[objective_id]
- else:
    ~ return 0
}

== function restore_quest(quest_id: int, state: QuestState) => void ==
~ quest_state[quest_id] = state

== function complete_main() => void ==
~ quest_state[QUEST_MAIN] = QuestState.Complete

== function complete_side() => void ==
~ quest_state[QUEST_SIDE] = QuestState.Complete

== function quest_log() => string ==
~ temp text: string = ""
{ for quest_id, state in quest_state:
    { if state != QuestState.Hidden:
        { if text == "":
            ~ text = quest_label(quest_id) + "=" + state_label(state)
        - else:
            ~ text = text + ", " + quest_label(quest_id) + "=" + state_label(state)
        }
    }
}
{ if text == "":
    ~ return "none"
- else:
    ~ return text
}

== function objective_summary() => string ==
~ temp text: string = ""
{ for objective_id, progress in objective_progress:
    { if text == "":
        ~ text = objective_label(objective_id) + " " + to_str(progress)
    - else:
        ~ text = text + ", " + objective_label(objective_id) + " " + to_str(progress)
    }
}
{ if text == "":
    ~ return "none"
- else:
    ~ return text
}

== function quest_label(quest_id: int) => string ==
{ if quest_id == QUEST_MAIN:
    ~ return "Missing Caravan"
- else:
    ~ return "Moonleaf Remedy"
}

== function objective_label(objective_id: int) => string ==
{ switch objective_id:
- OBJ_MAIN_REN:
    ~ return "recruit Ren"
- OBJ_MAIN_GATE:
    ~ return "open gate"
- OBJ_MAIN_BATTLE:
    ~ return "win mine battle"
- else:
    ~ return "gather moonleaf"
}

== function state_label(state: QuestState) => string ==
{ switch state:
- QuestState.Active:
    ~ return "active"
- QuestState.Complete:
    ~ return "complete"
- else:
    ~ return "hidden"
}
