=== module game ===
STRUCT Player {
hp: int
name: string
ready: bool
inventory: int[]
}
VAR global_score: int
VAR global_ready: bool
VAR global_label: string
VAR global_ratio: float
VAR global_values: int[]
VAR global_player: Player

== main ==
~ temp temp_score: int
~ temp temp_values: int[]
~ temp temp_player: Player
{global_score}|{global_ready}|{global_label}|{global_ratio}|{global_values}|{global_player}|{temp_score}|{temp_values}|{temp_player}
