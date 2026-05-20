=== module game ===
EXTERNAL bump_score(scores: Dict<string, int>, key: string) => int
EXTERNAL make_scores(seed: int) => Dict<string, int>

VAR host_scores: Dict<string, Dict<int, string>> = %{}

== main ==
~ temp source_scores: Dict<string, int> = %{"ada": 10}
~ temp bumped: int = bump_score(source_scores, "ada")
~ temp made: Dict<string, int> = make_scores(7)
{bumped}|{made["seed"]}|{made["next"]}
* Read host variable
    {host_scores["ada"][1]}|{host_scores["ada"][2]}
    -> DONE

== INTERNAL identity_scores(input_scores: Dict<string, int>) => Dict<string, int> ==
~ return input_scores

== INTERNAL build_table() => Dict<string, Dict<int, string>> ==
~ temp table: Dict<string, Dict<int, string>> = %{"ada": %{1: "one", 2: "two"}}
~ return table
