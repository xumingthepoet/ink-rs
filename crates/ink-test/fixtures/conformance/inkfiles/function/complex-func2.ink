~ derp(2, 3)
    The values are {x} and {y} and {z}.
    -> END
    
    === function derp(a: int, b: int) -> void ===  
    VAR x: int = 0
   ~ x = a - b
    VAR y: int = 3
    {
      - x == 0:
        ~ y = 0
      - x > 0:
        ~ y = x - 1
      - else:
        ~ y = x + 1
    }
    VAR z: int = 1