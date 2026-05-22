=== interface IRoute ===
== arrive(stops: int) ==
== function fare(base: int) => int ==

=== module game ===
FROM left
FROM right
FROM rates IMPORT base

VAR route: interface<IRoute> = left

== main ==
Default fare {current_fare()}.
~ route = right
Switched fare {current_fare()}.
-> {{route}::arrive}(2)

== function current_fare() => int ==
~ return {route}::fare(rates::base)

=== module rates ===
VAR base: int = 3

=== module left implements IRoute ===
== arrive(stops: int) ==
Left route {stops}.

== function fare(base: int) => int ==
~ return base + 1

=== module right implements IRoute ===
== arrive(stops: int) ==
Right route {stops}.

== function fare(base: int) => int ==
~ return base + 2
