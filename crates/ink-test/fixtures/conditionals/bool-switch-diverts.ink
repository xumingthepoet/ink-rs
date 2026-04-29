=== module game ===
VAR done: bool = true

== main ==
{ done:
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
