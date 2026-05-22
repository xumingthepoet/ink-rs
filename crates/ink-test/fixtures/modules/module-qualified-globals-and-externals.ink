=== module game ===
FROM left IMPORT level
FROM right IMPORT level
FROM audio IMPORT play
FROM video IMPORT play

== main ==
{audio::play("intro")}|{video::play("intro")}
~ left::level += 10
~ right::level += 20
Ready.
* Continue
  {left::level}|{right::level}

=== module left ===
VAR level: int = 1

== helper ==

=== module right ===
VAR level: int = 2

== helper ==

=== module audio ===
EXTERNAL play(name: string) => int

== helper ==

=== module video ===
EXTERNAL play(name: string) => int

== helper ==
