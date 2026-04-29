=== module game ===
VAR done: bool = true

== main ==
{ switch done:
- true:
    -> finish
- false:
    -> fail
}

== finish ==
Finished.
-> END

== fail ==
Failed.
-> END
