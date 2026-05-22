=== interface IItem ===
== target ==

=== module game ===
FROM wrong
VAR missingImport: interface<IItem> = left
VAR unknownRoute: interface<IItem> = missing
VAR wrongRoute: interface<IItem> = wrong
VAR defaultRoute: interface<IItem>
== main ==

=== module left implements IItem ===
== target ==

=== module wrong ===
== target ==
