=== module game ===

== main ==
-> start([101, 204, 305])

== start(enemy_ids: int[]) ==
First {enemy_ids[0]}.
Count {LEN(enemy_ids)}.
~ temp copied: int[] = identity([7, 8])
Copied {copied[1]}.
-> END

== function identity(values: int[]) => int[] ==
~ return values
