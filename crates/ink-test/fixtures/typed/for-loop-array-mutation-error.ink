=== module game ===
VAR values: int[] = [1, 2, 3]

== main ==
{ for index, value in values:
{index}:{value}
~ ARRAY_REMOVE(values, 0)
}
-> END
