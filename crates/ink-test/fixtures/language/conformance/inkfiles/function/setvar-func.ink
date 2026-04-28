=== module game ===
VAR x: int = 0

== main ==
~ herp(2, 3)
  The value is {x}.
  -> END


  == function herp(a: int, b: int) => void ==
  ~x = a * b