=== module game ===
STRUCT Sheet {
scores: Dict<string, int>
}

CONST base_scores: Dict<string, int> = %{"ada": 10}
CONST by_id: Dict<int, string> = %{1: "one"}

EXTERNAL make_dict_scores(seed: int) => Dict<string, int>
EXTERNAL pick_dict_score(scores: Dict<string, int>, key: string) => int

VAR default_scores: Dict<string, int>
VAR scores: Dict<string, int> = %{"ada": 10}
VAR names: Dict<int, string> = %{1: "one"}
VAR nested: Dict<string, Dict<int, string>> = %{"ada": %{1: "one"}}
VAR score_tables: Dict<string, int>[] = [%{"ada": 10}]
VAR sheet: Sheet = %Sheet{ scores: %{"ada": 10} }

== main ==
{default_scores}|{scores["ada"]}|{names[1]}|{nested["ada"][1]}|{base_scores["ada"]}|{by_id[1]}
~ scores["bea"] = 11
~ scores["ada"] = 12
~ nested["ada"][2] = "two"
~ score_tables[0]["ada"] = 13
~ sheet.scores["ada"] = 14
{scores["bea"]}|{scores["ada"]}|{nested["ada"][2]}|{score_tables[0]["ada"]}|{sheet.scores["ada"]}
~ temp copy: Dict<string, int> = scores
{copy == scores}|{copy != base_scores}
{sum_scores(scores)}|{lookup(base_scores, "ada")}
~ temp made: Dict<string, int> = make_dict_scores(5)
{made["seed"]}|{pick_dict_score(made, "next")}
{DICT_HAS(scores, "ada")}|{DICT_HAS(scores, "missing")}|{DICT_SIZE(scores)}
~ temp score_keys: string[] = DICT_KEYS(scores)
{score_keys[0]}|{score_keys[1]}|{LEN(score_keys)}
~ DICT_REMOVE(scores, "bea")
{DICT_HAS(scores, "bea")}|{DICT_SIZE(scores)}
~ DICT_REMOVE(scores, "missing")
{DICT_SIZE(scores)}
~ temp name_keys: int[] = DICT_KEYS(names)
{name_keys[0]}|{LEN(name_keys)}
~ DICT_REMOVE(nested["ada"], 1)
{DICT_HAS(nested["ada"], 1)}|{DICT_SIZE(nested["ada"])}

== function sum_scores(values: Dict<string, int>) => int ==
~ return values["ada"] + values["bea"]

== function lookup(values: Dict<string, int>, key: string) => int ==
~ return values[key]
