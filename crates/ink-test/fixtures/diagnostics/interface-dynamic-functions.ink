=== interface IItem ===
== target(amount: int) ==
== function score(amount: int) => int ==

=== module game ===
FROM left
VAR route: interface<IItem> = left
VAR label: string = "x"
== main ==
~ temp badBase: int = {label}::score(1)
~ temp missing: int = {route}::missing()
~ temp knotCall: int = {route}::target(1)
~ temp wrongCount: int = {route}::score()
~ temp wrongType: int = {route}::score("bad")

=== module left implements IItem ===
== target(amount: int) ==
== function score(amount: int) => int ==
~ return amount
