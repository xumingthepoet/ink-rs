=== module game ===
VAR scores: Dict<string, int> = %{"ada": 10}
VAR count: int = LEN(scores)

== main ==
~ ARRAY_REMOVE(scores, 0)
-> DONE
