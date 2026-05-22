=== interface IItem ===
== target ==

=== module game ===
FROM left
FROM right IMPORT helper
FROM wrong

VAR route: interface<IItem> = left

== main ==
{right::helper()}
~ route = right
~ route = wrong

=== module left implements IItem ===
== target ==

=== module right implements IItem ===
== target ==

== function helper() => string ==
~ return "helper"

=== module wrong ===
== target ==
