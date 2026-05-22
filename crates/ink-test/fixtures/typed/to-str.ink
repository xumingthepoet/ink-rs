=== module game ===

ENUM State { Idle Busy }

STRUCT Player {
    hp: int
    name: string
}

VAR score: int = 7
VAR ratio: float = 1.5
VAR ready: bool = true
VAR player: Player = %Player{ hp: 10, name: "Ada" }
VAR values: int[] = [1, 2]
VAR scores: Dict<string, int> = %{"ada": 10}

== function hp_label(value: int) => string ==
~ return "hp " + to_str(value)

== main ==
{to_str(score)}|{to_str(ratio)}|{to_str(ready)}|{to_str("ok")}|{to_str(State.Busy)}|{hp_label(player.hp)}|{to_str(values)}|{to_str(player)}|{to_str(scores)}
