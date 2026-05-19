=== interface IItem ===
== target(amount: int) ==
== function score() => int ==

=== module game ===
FROM left
VAR route: interface<IItem> = left
VAR label: string = "x"
== main ==
-> {{label}::target}
-> {{route}::missing}
-> {{route}::score}
-> {{route}::target}
-> {{route}::target}("bad")

=== module left implements IItem ===
== target(amount: int) ==
-> END
== function score() => int ==
~ return 1
