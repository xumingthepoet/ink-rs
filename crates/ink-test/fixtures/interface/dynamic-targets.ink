=== interface IItem ===
== target(amount: int) ==
== fallback ==

=== module game ===
FROM left
FROM right

VAR route: interface<IItem> = left
VAR routes: interface<IItem>[] = [left, right]

== main ==
-> {{route}::target}(3)

== alt ==
-> {{routes[1]}::fallback}

=== module left implements IItem ===
== target(amount: int) ==
-> END

== fallback ==
-> END

=== module right implements IItem ===
== target(amount: int) ==
-> END

== fallback ==
-> END
