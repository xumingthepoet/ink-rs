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
        ~ text = objective_label(objective_id) + " " + count_label(progress)
    - else:
        ~ text = text + ", " + objective_label(objective_id) + " " + count_label(progress)
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
- 101:
    ~ return "recruit Ren"
- 102:
    ~ return "open gate"
- 103:
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

== function count_label(value: int) => string ==
{ switch value:
- 0:
    ~ return "0"
- 1:
    ~ return "1"
- else:
    ~ return "2"
}
