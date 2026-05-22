=== module game ===
STRUCT Stats {
hp: int
ready: bool
}
VAR state: Stats = %Stats{ hp: 2, ready: false }

== main ==
~ state.hp = 5
~ state.ready = true
{state.hp}|{state.ready}
