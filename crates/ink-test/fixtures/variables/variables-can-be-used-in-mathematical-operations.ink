=== module game ===
VAR a: int = 3
VAR b: int = 5
VAR c: int = 13
VAR f: float = 3.0
VAR cf: float = 13.0
VAR bf: float = 5.0

== main ==
Integer calculation does each step as integers, which may not be what you want.
({a} - {c}) / {a} + {b} = {(a - c) / a + b} which should be 1.66666...!

Float calculation works better:
({f} - {cf}) / {f} + {bf} = {(f - cf) / f + bf}!
