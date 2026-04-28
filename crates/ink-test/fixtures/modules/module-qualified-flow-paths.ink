=== module game ===
IMPORT target, tunnel, value FROM routes

== main ==
{routes::value()}
-> routes::tunnel ->
After tunnel.
-> routes::target

=== module routes ===
== function value() => int ==
~ return 9

== tunnel ==
Tunnel.
->->

== target ==
Target.
-> END
