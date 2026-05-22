=== module game ===
VAR x: string = ""

== main ==
// Historical issue fixture.
// The correct output has to be:
// This is a test
// X is set

This is a test
SET_X:
{ if:
  - x == "":
    -> x_not_set
  - else:
    -> x_is_set
}
-> END

= x_not_set
X is not set!
-> END

= x_is_set
X is set
-> END
