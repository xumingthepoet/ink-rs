=== module game ===
STRUCT Sheet {
hp: int
scores: Dict<string, int>
}

CONST base_scores: Dict<string, int> = {"ada": 10}
CONST by_id: Dict<int, string> = {1: "one"}

VAR default_scores: Dict<string, int>
VAR literal_scores: Dict<string, int> = {"ada": 10, "grace": 11}
VAR copied_scores: Dict<string, int> = base_scores
VAR literal_by_id: Dict<int, string> = {1: "one"}
VAR nested_scores: Dict<int, Dict<string, int>> = {1: {"ada": 10}}
VAR score_tables: Dict<string, int>[] = [{"ada": 10}, {}]
VAR sheet: Sheet = { hp: 3, scores: {"luck": 7} }
VAR dynamic_scores: Dict<string, int> = {"sum": 1 + 1}

== main ==
~ temp local_default: Dict<int, string>
~ temp local_literal: Dict<int, string> = {2: "two"}
-> DONE
