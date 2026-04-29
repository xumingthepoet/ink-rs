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
VAR party: Player[] = [
    {
        name: "Ada",
        stats: {
            hp: 10,
            ready: true
        },
        tags: ["scout"]
    },
    {
        name: "Bea",
        stats: {
            hp: 8
        },
        tags: []
    }
]
CONST fallback: Stats = {
    hp: 3,
    ready: true
}
CONST backups: Stats[] = [
    {
        hp: 1
    },
    {
        hp: 2,
        ready: true
    }
]

== main ==
{party[0].name}|{party[1].stats.hp}|{fallback.ready}|{backups[1].hp}|{LEN(party)}
-> DONE
