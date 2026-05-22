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
    %Player{
        name: "Ada",
        stats: %Stats{
            hp: 10,
            ready: true
        },
        tags: ["scout"]
    },
    %Player{
        name: "Bea",
        stats: %Stats{
            hp: 8
        },
        tags: []
    }
]
CONST fallback: Stats = %Stats{
    hp: 3,
    ready: true
}
CONST backups: Stats[] = [
    %Stats{
        hp: 1
    },
    %Stats{
        hp: 2,
        ready: true
    }
]

== main ==
{party[0].name}|{party[1].stats.hp}|{fallback.ready}|{backups[1].hp}|{LEN(party)}
