=== interface IItem ===
== target(amount: int) ==

=== module game ===
FROM left
FROM right

VAR route: interface<IItem> = left
VAR alternates: interface<IItem>[] = [right]

== main ==
-> {{route}::target}(3)

=== module left implements IItem ===
== target(amount: int) ==
Left {amount}.
-> END

=== module right implements IItem ===
== target(amount: int) ==
Right {amount}.
-> END
