=== interface IItem ===
== target ==

=== module game ===
FROM left
FROM right

STRUCT Config {
route: interface<IItem>
routes: interface<IItem>[]
}

CONST default_route: interface<IItem> = right
VAR route: interface<IItem> = left
VAR routes: interface<IItem>[] = [left, right]
VAR config: Config = %Config{ route: right, routes: [left] }

== main ==
* Save point
  Done.

=== module left implements IItem ===
== target ==

=== module right implements IItem ===
== target ==
