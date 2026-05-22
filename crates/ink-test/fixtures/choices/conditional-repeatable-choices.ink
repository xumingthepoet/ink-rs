=== module game ===
VAR open: bool = false

== main ==
-> menu

== menu ==
* {open}: Open path
    Done.
* Toggle
    ~ open = true
    -> menu
