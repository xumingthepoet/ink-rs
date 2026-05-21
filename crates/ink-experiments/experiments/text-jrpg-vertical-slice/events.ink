=== module events ===

CONST FLAG_MAIN_ACCEPTED: int = 1
CONST FLAG_SIDE_ACCEPTED: int = 2
CONST FLAG_REN_JOINED: int = 3
CONST FLAG_REN_SCOUTING: int = 4
CONST FLAG_REN_REJOINED: int = 5
CONST FLAG_MOONLEAF_GATHERED: int = 6
CONST FLAG_VILLAGE_SAVE_WRITTEN: int = 7
CONST FLAG_FOREST_ENCOUNTER_CLEARED: int = 8
CONST FLAG_GATE_OPENED: int = 9
CONST FLAG_MINE_BATTLE_WON: int = 10
CONST FLAG_MINE_LOOT_RESOLVED: int = 11
CONST FLAG_CARAVAN_RESCUED: int = 12
CONST FLAG_MAIN_COMPLETE: int = 13
CONST FLAG_SIDE_COMPLETE: int = 14

VAR flags: Dict<int, bool> = %{}

== function set_flag(flag_id: int) => void ==
~ flags[flag_id] = true

== function set_flag_to(flag_id: int, enabled: bool) => void ==
~ flags[flag_id] = enabled

== function clear_flag(flag_id: int) => void ==
~ flags[flag_id] = false

== function has_flag(flag_id: int) => bool ==
{ if DICT_HAS(flags, flag_id):
    ~ return flags[flag_id]
- else:
    ~ return false
}

== function flag_summary() => string ==
~ temp text: string = ""
{ for flag_id, enabled in flags:
    { if enabled:
        { if text == "":
            ~ text = flag_label(flag_id)
        - else:
            ~ text = text + ", " + flag_label(flag_id)
        }
    }
}
{ if text == "":
    ~ return "none"
- else:
    ~ return text
}

== function flag_label(flag_id: int) => string ==
{ switch flag_id:
- 1:
    ~ return "main accepted"
- 2:
    ~ return "side accepted"
- 3:
    ~ return "Ren joined"
- 4:
    ~ return "Ren scouting"
- 5:
    ~ return "Ren rejoined"
- 6:
    ~ return "moonleaf"
- 7:
    ~ return "saved"
- 8:
    ~ return "forest clear"
- 9:
    ~ return "gate open"
- 10:
    ~ return "mine won"
- 11:
    ~ return "mine loot"
- 12:
    ~ return "caravan"
- 13:
    ~ return "main complete"
- else:
    ~ return "side complete"
}
