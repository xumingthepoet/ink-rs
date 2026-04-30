=== module game ===
VAR value: float = 3.6
VAR threshold: float = 10.0
VAR unit: string = "Röntgen"

== main ==
-> root

== root ==
The latest measurement is {value} {unit}. {value < threshold: Not terrible, not great. | Oh no.}

*   Redo measurement -> root
-> END
