=== module game ===
STRUCT Stats {
hp: int
ready: bool
}
VAR state: Stats = { hp: 9, ready: true }

== main ==
{state.hp}|{state.ready}
-> DONE
