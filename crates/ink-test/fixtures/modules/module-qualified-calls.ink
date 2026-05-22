=== module game ===
FROM math IMPORT add
FROM audio IMPORT play

== main ==
{math::add(2, 3)}
{audio::play("intro")}

=== module math ===
== function add(left: int, right: int) => int ==
~ return left + right

=== module audio ===
EXTERNAL play(name: string) => int

== helper ==
