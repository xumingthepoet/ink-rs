=== module game ===
VAR x: int = 1
VAR y: int = 2
VAR z: int = 0

== main ==
How much do you give?
* $1
    -> give(x)
* $2
    -> give(y)
* Nothing
    -> give(z)

== give(amount) ==
You give {amount} dollars.
-> DONE
