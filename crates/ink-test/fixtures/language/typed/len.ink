=== module game ===
STRUCT Player {
hp: int
}
VAR empty: int[] = []
VAR items: int[] = [1, 2, 3]
VAR players: Player[] = [{ hp: 1 }, { hp: 2 }]

== main ==
{LEN(empty)}|{LEN(items)}|{LEN(players)}
-> DONE
