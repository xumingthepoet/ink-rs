EXTERNAL externalFunction(x: int, y: int, z: int) => int

The value is {externalFunction(1, 2, 3)}.
-> END

=== function externalFunction(x: int, y: int, z: int) => int ===
// Usually external functions can only return placeholder
// results, otherwise they'd be defined in ink!
~ return 0
