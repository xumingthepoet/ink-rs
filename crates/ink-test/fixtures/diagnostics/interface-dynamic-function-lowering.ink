=== interface IItem ===
== function score(amount: int) => int ==

=== module game ===
FROM left
VAR route: interface<IItem> = left
== main ==
~ temp value: int = {route}::score(1)
-> END

=== module left implements IItem ===
== function score(amount: int) => int ==
~ return amount
