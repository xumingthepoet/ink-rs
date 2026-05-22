=== module game ===
VAR scores: Dict<string, int> = %{"ada": 10}
VAR names: Dict<int, string> = %{1: "one"}
VAR nested: Dict<int, Dict<string, int>> = %{1: %{"ada": 10}}

== main ==
{scores["ada"]}|{names[1]}|{nested[1]["ada"]}
