=== module puzzle ===

VAR input: int[] = []
VAR failed_attempts: int = 0
VAR solved: bool = false

== gate ==
The mine gate waits for three runes.
Input: {sequence_label()}.
* Sun
    -> choose_rune(1)
* Moon
    -> choose_rune(2)
* Star
    -> choose_rune(3)

== choose_rune(rune_id: int) ==
~ ARRAY_PUSH(input, rune_id)
{ if LEN(input) < 3:
    Rune {rune_label(rune_id)} lights.
    -> gate
- else:
    { if input[0] == 1 && input[1] == 3 && input[2] == 2:
        ~ solved = true
        Gate sequence sun-star-moon opens the mine.
        ->->
    - else:
        ~ failed_attempts += 1
        ~ input = []
        The gate rejects the sequence. Attempts: {attempt_label(failed_attempts)}.
        -> gate
    }
}

== function sequence_label() => string ==
~ temp text: string = ""
{ for rune_id in input:
    { if text == "":
        ~ text = rune_label(rune_id)
    - else:
        ~ text = text + "-" + rune_label(rune_id)
    }
}
{ if text == "":
    ~ return "empty"
- else:
    ~ return text
}

== function rune_label(rune_id: int) => string ==
{ switch rune_id:
- 1:
    ~ return "sun"
- 2:
    ~ return "moon"
- else:
    ~ return "star"
}

== function attempt_label(value: int) => string ==
{ if value == 1:
    ~ return "1"
- else:
    ~ return "2"
}
