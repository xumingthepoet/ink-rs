=== module game ===
STRUCT Stats {
hp: int
ready: bool
}
STRUCT Player {
name: string
stats: Stats
tags: string[]
}
VAR full: Player = %Player{ name: "Ada", stats: %Stats{ hp: 10, ready: true }, tags: ["scout"] }
VAR partial: Player = %Player{ name: "Bea" }
VAR nested: Stats = %Stats{ hp: 3 }

== main ==
{full}|{partial}|{nested}
