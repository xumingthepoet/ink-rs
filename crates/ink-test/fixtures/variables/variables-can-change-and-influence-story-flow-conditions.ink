=== module game ===
VAR value: float = 3.6
VAR unit: string = "Röntgen"
VAR is_hazardous: bool = false

== main ==
-> root

== root ==
The latest measurement is {value} {unit}. {not is_hazardous: Not terrible, not great. | Oh no.}

*   Redo measurement -> root
