=== interface IItem ===
== target ==

=== module game ===
FROM left
VAR route: interface<IItem> = left
== main ==
-> {{route}::target}

=== module left implements IItem ===
== target ==
-> END
