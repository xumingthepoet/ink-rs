=== interface IItem ===
== function score(amount: int) => int ==

=== module game ===
FROM left
FROM right

VAR route: interface<IItem> = left
VAR alternates: interface<IItem>[] = [right]

== main ==
~ temp score: int = {route}::score(3)
Score {score}.
-> END

=== module left implements IItem ===
== function score(amount: int) => int ==
~ return amount

=== module right implements IItem ===
== function score(amount: int) => int ==
~ return amount + 1
