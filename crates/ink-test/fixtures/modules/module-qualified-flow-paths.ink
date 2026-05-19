=== module game ===
FROM routes IMPORT target, tunnel, value

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
