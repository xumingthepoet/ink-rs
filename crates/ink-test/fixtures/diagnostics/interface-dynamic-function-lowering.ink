=== interface IItem ===
== function score(amount: int) => int ==

=== module game ===
FROM left
VAR route: interface<IItem> = left
== main ==
~ get_score()
-> END
== function get_score() => int ==
~ return {route}::score(1)

=== module left implements IItem ===
== function score(amount: int) => int ==
~ return amount
