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
VAR full: Player = { name: "Ada", stats: { hp: 10, ready: true }, tags: ["scout"] }
VAR partial: Player = { name: "Bea" }
VAR nested: Stats = { hp: 3 }

== main ==
{full}|{partial}|{nested}
-> DONE