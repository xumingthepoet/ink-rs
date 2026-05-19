=== interface IItem ===
== target ==

=== module game ===
FROM left
FROM right IMPORT target
FROM wrong

VAR route: interface<IItem> = left

== main ==
~ route = right
~ route = wrong
-> END

=== module left implements IItem ===
== target ==
-> END

=== module right implements IItem ===
== target ==
-> END

=== module wrong ===
== target ==
-> END
