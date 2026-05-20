=== module game ===
STRUCT Player {
hp: int
}

VAR oldStruct: Player = { hp: 10 }
VAR oldStringDict: Dict<string, int> = {"ada": 10}
VAR oldIntDict: Dict<int, string> = {1: "one"}
VAR oldEmptyDict: Dict<string, int> = {}

== main ==
-> DONE
