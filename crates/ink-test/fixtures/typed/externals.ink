=== module game ===
STRUCT Player {
    hp: int
}

EXTERNAL next_score(value: int) => int
EXTERNAL make_scores() => int[]
EXTERNAL make_player() => Player

== main ==
~ temp score: int = next_score(4)
~ temp scores: int[] = make_scores()
~ temp player: Player = make_player()
{score}|{scores[0]}|{scores[1]}|{player.hp}
