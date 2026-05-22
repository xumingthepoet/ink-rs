=== module game ===
STRUCT Player {
hp: int
}
VAR p1: Player = %Player{ hp: 3 }
VAR p2: Player = p1

== main ==
~ p2.hp = 1
{p1.hp}|{p2.hp}
