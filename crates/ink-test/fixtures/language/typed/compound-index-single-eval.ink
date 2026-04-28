=== module game ===
VAR calls: int = 0
VAR items: int[] = [1, 2]

== main ==
~ items[idx()] += 1
{items[0]}|{calls}
-> DONE

== function idx() => int ==
~ calls += 1
~ return 0
