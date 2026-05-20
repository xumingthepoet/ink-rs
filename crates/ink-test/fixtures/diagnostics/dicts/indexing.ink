=== module game ===
VAR scores: Dict<string, int> = %{"ada": 10}
VAR value: int = 1

== main ==
{scores[1]}
{value[0]}
~ scores[1] = 5
~ value[0] = 2
-> DONE
