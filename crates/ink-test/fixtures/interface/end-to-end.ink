=== interface IItem ===
== target(amount: int) ==
== function score(amount: int) => int ==

=== interface ILabel ===
== function label() => string ==

=== module game ===
FROM left
FROM right
FROM bonus IMPORT value, describe

VAR route: interface<IItem> = left
VAR labeler: interface<ILabel> = right
VAR routes: interface<IItem>[] = [left, right]

== main ==
{bonus::describe()} {bonus::value}.
Default {get_score()}.
Array {array_score()}.
Label {get_label()}.
~ route = right
Switched {get_score()}.
-> {{route}::target}(2)

== function get_score() => int ==
~ return {route}::score(bonus::value)

== function array_score() => int ==
~ return {routes[1]}::score(1)

== function get_label() => string ==
~ return {labeler}::label()

=== module bonus ===
VAR value: int = 3

== function describe() => string ==
~ return "Bonus"

=== module left implements IItem ===
== target(amount: int) ==
Left target {amount}.

== function score(amount: int) => int ==
~ return amount + 10

=== module right implements IItem, ILabel ===
== target(amount: int) ==
Right target {amount}.

== function score(amount: int) => int ==
~ return amount + 20

== function label() => string ==
~ return "right"
