=== interface IItem ===
== function score(amount: int) => int ==

=== module game ===
FROM left
FROM right

VAR route: interface<IItem> = left
VAR alternates: interface<IItem>[] = [right]

== main ==
Score {get_score()}.
-> END

== function get_score() => int ==
~ return {route}::score(3)

=== module left implements IItem ===
== function score(amount: int) => int ==
~ return amount

=== module right implements IItem ===
== function score(amount: int) => int ==
~ return amount + 1
