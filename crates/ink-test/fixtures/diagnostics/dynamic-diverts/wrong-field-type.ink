=== module game ===
STRUCT Player {
hp: int
}
VAR player: Player = %Player{ hp: 10 }

== main ==
-> {player.hp}

== player ==
= hp
