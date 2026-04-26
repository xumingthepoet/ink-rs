~ derp(2, 3, 4)
   The values are {x} and {y}.
   -> END
   
   === function derp(a: int, b: int, c: int) -> void ===
   VAR x: int = 0
   ~ x = a + b
   VAR y: int = 3
   { x == 5:
      ~ x = 6
   }
   ~ y = x + c