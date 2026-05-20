=== module game ===
STRUCT Player {
hp: int
}

== main ==
~ temp player: Player = %Player{ hp: 7 }
{player.hp}
-> DONE

== player ==
= hp
-> DONE
