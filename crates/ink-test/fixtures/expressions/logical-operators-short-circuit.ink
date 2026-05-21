=== module game ===

VAR values: int[] = [7]
VAR effects: int = 0

== function mark(value: bool) => bool ==
~ effects = effects + 1
~ return value

== main ==
~ temp false_and_symbol: bool = false && values[1] == 7
~ temp true_or_symbol: bool = true || values[1] == 7
~ temp false_and_word: bool = false and values[1] == 7
~ temp true_or_word: bool = true or values[1] == 7
~ temp needed_and: bool = true && mark(true)
~ temp needed_or: bool = false || mark(true)
{false_and_symbol}
{true_or_symbol}
{false_and_word}
{true_or_word}
{needed_and}
{needed_or}
Effects: {effects}
-> END
