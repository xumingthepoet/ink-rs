=== module game ===
VAR scores: Dict<string, int> = %{"ada": 10}
VAR count: int = LEN(scores)
VAR wrong_has_key: bool = DICT_HAS(scores, 1)
VAR wrong_size_arg: int = DICT_SIZE(1)
VAR wrong_keys_arg: string[] = DICT_KEYS(1)

== main ==
~ ARRAY_REMOVE(scores, 0)
~ DICT_REMOVE(scores, 1)
~ DICT_REMOVE(copy_scores(), "ada")

== function copy_scores() => Dict<string, int> ==
~ return scores
