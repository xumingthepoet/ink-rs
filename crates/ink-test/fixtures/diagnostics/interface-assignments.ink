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
-> END

=== module left implements IItem ===
== target ==
-> END

=== module right implements IItem ===
== target ==
-> END

== function helper() => string ==
~ return "helper"

=== module wrong ===
== target ==
-> END
