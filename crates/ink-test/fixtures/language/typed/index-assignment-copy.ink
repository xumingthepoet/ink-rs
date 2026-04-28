=== module game ===
VAR items1: int[] = [4, 5]
VAR items2: int[] = items1

== main ==
~ items2[0] = 9
{items1[0]}|{items2[0]}
-> DONE
