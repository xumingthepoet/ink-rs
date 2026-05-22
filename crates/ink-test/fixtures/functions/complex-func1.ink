=== module game ===
VAR x: int = 0
VAR y: int = 3

== main ==
~ derp(2, 3, 4)
   The values are {x} and {y}.


   == function derp(a: int, b: int, c: int) => void ==
   ~ x = a + b
   { if x == 5:
      ~ x = 6
   }
   ~ y = x + c
