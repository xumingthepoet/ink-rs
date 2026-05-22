=== interface IEventHandler ===
== target ==

=== module game ===
FROM heal_event
FROM damage_event

VAR chosen: interface<IEventHandler> = heal_event
VAR picked: interface<IEventHandler> = damage_event

== main ==
~ picked = choose(heal_event)
-> register(damage_event) -> after_register

== after_register ==
{chosen}|{picked}

== register(handler: interface<IEventHandler>) ==
~ chosen = handler
->->

== function choose(handler: interface<IEventHandler>) => interface<IEventHandler> ==
~ return handler

=== module heal_event implements IEventHandler ===
== target ==

=== module damage_event implements IEventHandler ===
== target ==
