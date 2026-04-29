=== module game ===
VAR x: int = 0
VAR y: int = 3
VAR z: int = 1

== main ==
~ derp(2, 3)
    The values are {x} and {y} and {z}.
    -> END


    == function derp(a: int, b: int) => void ==
   ~ x = a - b
    { if:
      - x == 0:
        ~ y = 0
      - x > 0:
        ~ y = x - 1
      - else:
        ~ y = x + 1
    }
