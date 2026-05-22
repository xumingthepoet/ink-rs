=== module game ===
STRUCT Player {
hp: int
}

VAR unsupportedStruct: Player = { hp: 10 }
VAR unsupportedStringDict: Dict<string, int> = {"ada": 10}
VAR unsupportedIntDict: Dict<int, string> = {1: "one"}
VAR unsupportedEmptyDict: Dict<string, int> = {}

== main ==
