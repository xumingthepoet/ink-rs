=== module game ===
FROM state IMPORT score

== main ==
{state::score}
~ state::score += 2
{state::score}
~ state::score = state::score + 3
{state::score}
-> END

=== module state ===
VAR score: int = 1

== helper ==
-> END
