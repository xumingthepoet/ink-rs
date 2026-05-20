=== module game ===
STRUCT Sheet {
scores: Dict<string, int>
}

VAR scores: Dict<string, int> = %{"ada": 10}
VAR by_id: Dict<int, string> = %{1: "one"}
VAR nested: Dict<string, Dict<int, string>> = %{"row": %{1: "one"}}
VAR score_tables: Dict<string, int>[] = [%{"ada": 10}]
VAR empty_scores: Dict<string, int> = %{}
VAR sheet: Sheet = %Sheet{ scores: %{"ada": 10} }

== main ==
{scores["ada"]}|{by_id[1]}|{nested["row"][1]}|{score_tables[0]["ada"]}|{sheet.scores["ada"]}
~ scores["bea"] = 11
~ nested["row"][2] = "two"
~ score_tables[0]["ada"] = 12
~ sheet.scores["ada"] = 13
~ temp copy: Dict<string, int> = scores
{lookup(copy, "ada")}
-> DONE

== function lookup(values: Dict<string, int>, key: string) => int ==
~ return values[key]
