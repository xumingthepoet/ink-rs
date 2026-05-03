=== module game ===
STRUCT Player {
    hp: int
}

VAR counter: int = 0

== main ==
Ready.
-> DONE

== INTERNAL read_config() => string ==
~ counter = counter + 1
~ return "enabled"

== INTERNAL double(value: int) => int ==
~ return value * 2

== INTERNAL identity_scores(values: int[]) => int[] ==
~ return values

== INTERNAL identity_player(player: Player) => Player ==
~ return player

== INTERNAL noisy() => string ==
This should not be output.
~ return "noisy"

== function private_value() => string ==
~ return "private"
