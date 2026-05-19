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

== INTERNAL build_scores(value: int) => int[] ==
~ temp scores: int[] = [value, value + 1]
~ return scores

== INTERNAL build_player(hp: int) => Player ==
~ temp player: Player = { hp: hp }
~ return player

== INTERNAL noisy() => string ==
This should not be output.
~ return "noisy"

== function private_value() => string ==
~ return "private"

=== module host_config ===
FROM config_data IMPORT default_value

VAR reads: int = 0

== INTERNAL read_unimported_config() => string ==
~ reads = reads + 1
~ return config_data::default_value()

=== module config_data ===
== function default_value() => string ==
~ return "remote"
