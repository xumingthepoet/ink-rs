=== module game ===

STRUCT Fighter {
    name: string
    power: int
    speed: int
    discipline: int
    resilience: int
    injuries: int
    wins: int
}

VAR entrants: Fighter[] = [
    %Fighter{
        name: "Arius",
        power: 12,
        speed: 10,
        discipline: 7,
        resilience: 11,
        injuries: 0,
        wins: 0
    },
    %Fighter{
        name: "Bren",
        power: 11,
        speed: 9,
        discipline: 8,
        resilience: 10,
        injuries: 0,
        wins: 0
    },
    %Fighter{
        name: "Cora",
        power: 13,
        speed: 8,
        discipline: 9,
        resilience: 12,
        injuries: 0,
        wins: 0
    },
    %Fighter{
        name: "Doran",
        power: 10,
        speed: 7,
        discipline: 10,
        resilience: 13,
        injuries: 0,
        wins: 0
    },
    %Fighter{
        name: "Eira",
        power: 9,
        speed: 9,
        discipline: 12,
        resilience: 9,
        injuries: 0,
        wins: 0
    },
    %Fighter{
        name: "Fen",
        power: 8,
        speed: 8,
        discipline: 11,
        resilience: 12,
        injuries: 0,
        wins: 0
    },
    %Fighter{
        name: "Galen",
        power: 12,
        speed: 7,
        discipline: 9,
        resilience: 11,
        injuries: 0,
        wins: 0
    },
    %Fighter{
        name: "Hela",
        power: 7,
        speed: 10,
        discipline: 8,
        resilience: 10,
        injuries: 0,
        wins: 0
    }
]

== main ==
Arena bracket runs in fixed rounds.
~ print_roster(0)
~ run_bracket()
-> DONE

== function run_bracket() => void ==
Quarterfinals begin.
~ temp qf_a: int = fight(0, 1, "Quarterfinal 1")
~ temp qf_b: int = fight(2, 3, "Quarterfinal 2")
~ temp qf_c: int = fight(4, 5, "Quarterfinal 3")
~ temp qf_d: int = fight(6, 7, "Quarterfinal 4")
Quarterfinals done.
Semifinals begin.
~ temp sf_a: int = fight(qf_a, qf_b, "Semifinal 1")
~ temp sf_b: int = fight(qf_c, qf_d, "Semifinal 2")
Semifinals done.
Final round.
~ temp champion: int = fight(sf_a, sf_b, "Final")
~ print_champion(champion)
Final roster status.
~ print_roster(0)

== function fight(first_index: int, second_index: int, label: string) => int ==
-- {label} --
~ temp first: Fighter = entrants[first_index]
~ temp second: Fighter = entrants[second_index]
Matchup: {first.name} vs {second.name}
~ temp winner: int = resolve_fight(first_index, second_index)
~ temp loser: int = first_index
{ if winner == second_index:
    ~ loser = first_index
- else:
    ~ loser = second_index
}
{ if winner == first_index:
    {first.name} wins.
- else:
    {second.name} wins.
}

~ entrants[winner].wins = entrants[winner].wins + 1
~ entrants[winner].power = entrants[winner].power - 1
~ entrants[winner].discipline = entrants[winner].discipline + 1

~ entrants[loser].injuries = entrants[loser].injuries + 1
~ entrants[loser].resilience = entrants[loser].resilience - 1
~ temp loser_name: string = entrants[loser].name
{ if entrants[loser].injuries == 1:
    {loser_name} is lightly injured.
- else:
    { if entrants[loser].injuries == 2:
        {loser_name} is clearly hurt after repeated fights.
    - else:
        {loser_name} is heavily injured and unreliable.
    }
}

{ if entrants[loser].resilience <= 2:
    {loser_name} is at risk of withdrawal if the bracket continued.
}

~ return winner

== function resolve_fight(first_index: int, second_index: int) => int ==
~ temp first_score: int = entrants[first_index].power + entrants[first_index].speed + entrants[first_index].discipline + entrants[first_index].resilience - entrants[first_index].injuries * 2
~ temp second_score: int = entrants[second_index].power + entrants[second_index].speed + entrants[second_index].discipline + entrants[second_index].resilience - entrants[second_index].injuries * 2
~ temp first_name: string = entrants[first_index].name
~ temp second_name: string = entrants[second_index].name
-- Score compare: {first_name} {first_score} vs {second_name} {second_score}
{ if first_score > second_score:
    ~ return first_index
- else:
    { if first_score < second_score:
        ~ return second_index
    - else:
        { if entrants[first_index].speed >= entrants[second_index].speed:
            ~ return first_index
        - else:
            ~ return second_index
        }
    }
}

== function print_champion(index: int) => void ==
Champion:
{ entrants[index].name}
Wins: {entrants[index].wins}
Power remaining: {entrants[index].power}
Speed: {entrants[index].speed}
Discipline: {entrants[index].discipline}
Resilience: {entrants[index].resilience}
Injuries: {entrants[index].injuries}
Final champion output:
{ if entrants[index].injuries == 0:
    The champion stayed clear and uninjured.
- else:
    The champion prevailed but carries at least one injury.
}

== function print_roster(index: int) => void ==
{ if index < LEN(entrants):
    ~ temp fighter: Fighter = entrants[index]
    { index + 1}. {fighter.name} | score {fighter.power + fighter.speed + fighter.discipline + fighter.resilience} | wins {fighter.wins} | injuries {fighter.injuries}
    ~ print_roster(index + 1)
}
