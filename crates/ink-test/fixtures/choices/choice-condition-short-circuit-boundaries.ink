=== module game ===

VAR hits: int = 0

== function mark(value: bool) => bool ==
~ hits = hits + 1
~ return value

== main ==
* {false}{mark(true)} separate blocks stay eager
  Hidden.
  -> END
* {true && mark(true)} inner and evaluates right
  Inner and.
  -> END
* {true || mark(true)} inner or short-circuits
  Inner or.
  -> END
