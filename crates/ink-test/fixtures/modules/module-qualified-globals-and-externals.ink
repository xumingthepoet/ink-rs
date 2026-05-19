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
  -> END

=== module left ===
VAR level: int = 1

== helper ==
-> END

=== module right ===
VAR level: int = 2

== helper ==
-> END

=== module audio ===
EXTERNAL play(name: string) => int

== helper ==
-> END

=== module video ===
EXTERNAL play(name: string) => int

== helper ==
-> END
