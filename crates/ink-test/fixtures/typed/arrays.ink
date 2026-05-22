=== module game ===
VAR items: int[] = [1, 2, 3]

== main ==
~ items[1] += 8
{LEN(items)}|{items[0]}|{items[1]}|{items[2]}
~ ARRAY_REMOVE(items, 0)
{LEN(items)}|{items[0]}|{items[1]}
