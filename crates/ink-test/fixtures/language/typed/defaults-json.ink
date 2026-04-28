=== module game ===
STRUCT Player {
hp: int
name: string
ready: bool
inventory: int[]
}
VAR global_score: int
VAR global_values: int[]
VAR global_player: Player

== main ==
~ temp temp_score: int
~ temp temp_values: int[]
~ temp temp_player: Player
-> DONE
