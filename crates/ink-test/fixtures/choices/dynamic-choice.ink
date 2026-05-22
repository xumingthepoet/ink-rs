=== module game ===
VAR labels: string[] = ["Alpha", "Hidden", "Beta"]
VAR enabled: bool[] = [true, false, true]

== main ==
* Fixed first
    fixed first.
* [i, label in labels] {enabled[i]}: {i}:{label}
    picked {i}:{label}.
* Fixed last
    fixed last.
