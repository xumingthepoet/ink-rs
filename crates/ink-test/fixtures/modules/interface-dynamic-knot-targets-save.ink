=== interface IItem ===
== target(amount: int) ==

=== module game ===
FROM left
FROM right

VAR route: interface<IItem> = left
VAR alternates: interface<IItem>[] = [right]

== main ==
* Go
  -> after_prompt

== after_prompt ==
-> {{route}::target}(3)

=== module left implements IItem ===
== target(amount: int) ==
Left {amount}.

=== module right implements IItem ===
== target(amount: int) ==
Right {amount}.
