=== module game ===
STRUCT Sheet {
scores: Dict<string, int>
}

VAR scores: Dict<string, int> = %{"ada": 10}
VAR nested: Dict<int, Dict<string, int>> = %{1: %{"ada": 10}}
VAR score_arrays: Dict<string, int>[] = [%{"ada": 10}]
VAR sheet: Sheet = %Sheet{ scores: %{"ada": 10} }

== main ==
~ scores["bea"] = 11
~ scores["ada"] = 12
~ nested[1]["ada"] = 13
~ score_arrays[0]["ada"] = 14
~ sheet.scores["ada"] = 15
