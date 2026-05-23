=== module game ===
VAR labels: string[] = ["East", "West"]

== main ==
Start.
* Visit hub
    -> hub.entry
* [i, label in labels] Dynamic {label}
    Dynamic branch {i}:{label}.
- regroup
Regrouped.

== hub ==
= entry
Hub entry.
* Return to regroup
    Hub branch.
* Natural stitch end
    Hub natural end.
