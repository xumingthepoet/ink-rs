VAR x: float = 0.0
~ x = lerp(2.0, 8.0, 0.4)
  The value of x is {x}.
  -> END
  
  === function lerp(a: float, b: float, k: float) -> float ===
      ~ return ((b - a) * k) + a