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
VAR config: Config = { route: right, routes: [left] }

== main ==
~ temp local: interface<IItem> = right
~ temp local_routes: interface<IItem>[] = [left, route]
~ temp local_config: Config = { route: route, routes: [right] }
{route}|{routes}|{config}|{default_route}|{local}|{local_routes}|{local_config}
-> DONE

=== module left implements IItem ===
== target ==
-> END

=== module right implements IItem ===
== target ==
-> END
