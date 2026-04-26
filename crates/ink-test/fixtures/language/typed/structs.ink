=== module game ===
STRUCT Stats {
    hp: int
    ready: bool
}

STRUCT Player {
    name: string
    stats: Stats
}

VAR original: Player = { name: "Ada", stats: { hp: 10, ready: true } }
VAR copy: Player = original

== main ==
~ copy.stats.hp = 3
{original.stats.hp}|{copy.stats.hp}|{original == copy}|{original.name}
-> DONE
