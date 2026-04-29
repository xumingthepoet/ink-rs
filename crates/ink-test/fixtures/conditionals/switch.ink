=== module game ===
VAR quest_stage: int = 1

== main ==
{ quest_stage:
- 0:
    stage zero
- 1:
    stage one
- else:
    stage many
}
-> END
