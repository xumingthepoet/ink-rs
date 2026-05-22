=== interface IItem ===
== target(amount: int) ==
== fallback ==
== function score(amount: int) => int ==

=== module game ===
FROM left
FROM right

VAR route: interface<IItem> = left
VAR routes: interface<IItem>[] = [left, right]

== main ==
-> {{route}::target}(3)

== alt ==
-> {{routes[1]}::fallback}

== function calc() => int ==
~ return {route}::score(5)

=== module left implements IItem ===
== target(amount: int) ==

== fallback ==

== function score(amount: int) => int ==
~ return amount

=== module right implements IItem ===
== target(amount: int) ==

== fallback ==

== function score(amount: int) => int ==
~ return amount + 1
