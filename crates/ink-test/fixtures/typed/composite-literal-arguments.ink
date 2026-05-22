=== interface IComposite ===
== target(scores: Dict<string, int>, player: types::Player, empty_scores: Dict<string, int>) ==
== function score(scores: Dict<string, int>, player: types::Player, empty_scores: Dict<string, int>) => int ==

=== module types ===
STRUCT Player {
hp: int
}

=== module game ===
FROM impl
FROM types IMPORT Player

VAR route: interface<IComposite> = impl

== main ==
~ temp from_function: int = identity(%{"ada": 10}, %types::Player{ hp: 2 }, %{})
~ temp from_dynamic: int = {route}::score(%{"ada": 20}, %types::Player{ hp: 3 }, %{})
Function {from_function}. Dynamic {from_dynamic}.
-> static(%{"ada": 30}, %types::Player{ hp: 4 }, %{})

== function identity(scores: Dict<string, int>, player: types::Player, empty_scores: Dict<string, int>) => int ==
~ return scores["ada"] + player.hp

== static(scores: Dict<string, int>, player: types::Player, empty_scores: Dict<string, int>) ==
Static {scores["ada"] + player.hp}.
-> tunnel(%{"ada": 50}, %types::Player{ hp: 6 }, %{}) -> after_tunnel

== tunnel(scores: Dict<string, int>, player: types::Player, empty_scores: Dict<string, int>) ==
Tunnel {scores["ada"] + player.hp}.
->->

== after_tunnel ==
-> override_tunnel(%{"ada": 60}, %types::Player{ hp: 7 }, %{}) -> after_override

== override_tunnel(scores: Dict<string, int>, player: types::Player, empty_scores: Dict<string, int>) ==
Override {scores["ada"] + player.hp}.
->-> onward(%{"ada": 70}, %types::Player{ hp: 8 }, %{})

== onward(scores: Dict<string, int>, player: types::Player, empty_scores: Dict<string, int>) ==
Onward {scores["ada"] + player.hp}.
-> {{route}::target}(%{"ada": 40}, %types::Player{ hp: 5 }, %{})

== after_override ==

=== module impl implements IComposite ===
FROM types IMPORT Player

== target(scores: Dict<string, int>, player: types::Player, empty_scores: Dict<string, int>) ==
Target {scores["ada"] + player.hp}.

== function score(scores: Dict<string, int>, player: types::Player, empty_scores: Dict<string, int>) => int ==
~ return scores["ada"] + player.hp
