=== module game ===
IMPORT add FROM math
IMPORT play FROM audio

== main ==
{math::add(2, 3)}
{audio::play("intro")}
-> END

=== module math ===
== function add(left: int, right: int) => int ==
~ return left + right

=== module audio ===
EXTERNAL play(name: string) => int

== helper ==
-> END
