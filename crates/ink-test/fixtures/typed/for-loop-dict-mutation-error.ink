=== module game ===
VAR scores: Dict<string, int> = %{"ada": 1, "grace": 2}

== main ==
{ for key, value in scores:
{key}:{value}
~ DICT_REMOVE(scores, "grace")
}
