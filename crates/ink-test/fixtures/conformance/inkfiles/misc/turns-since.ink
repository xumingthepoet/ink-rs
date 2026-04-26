-> start

=== start ===
    {beats(-> start)}
    {beats(-> start)}
    *   [Choice]  -> next
= next
    {beats(-> start)}
    -> END

=== function beats(x: ->) => int ===
    ~ return TURNS_SINCE(x)
