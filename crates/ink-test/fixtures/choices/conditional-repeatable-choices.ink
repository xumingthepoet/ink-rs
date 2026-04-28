=== module game ===
VAR open: bool = false

== main ==
-> menu

== menu ==
* {open} Open path
    Done.
    -> DONE
* Toggle
    ~ open = true
    -> menu
