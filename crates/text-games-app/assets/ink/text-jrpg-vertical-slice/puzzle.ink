=== module puzzle ===

CONST rune_choices: int[] = [1, 2, 3]
VAR input: int[] = []
VAR failed_attempts: int = 0
VAR solved: bool = false

== gate ==
The mine gate waits for three runes.
Input: {sequence_label()}.
* [rune_id in rune_choices] {rune_choice_label(rune_id)}
    -> choose_rune(rune_id)

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
        The gate rejects the sequence. Attempts: {to_str(failed_attempts)}.
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

== function input_snapshot() => int[] ==
~ return input

== function failed_attempt_count() => int ==
~ return failed_attempts

== function is_solved() => bool ==
~ return solved

== function restore_state(saved_input: int[], saved_failed_attempts: int, saved_solved: bool) => void ==
~ input = saved_input
~ failed_attempts = saved_failed_attempts
~ solved = saved_solved

== function rune_label(rune_id: int) => string ==
{ switch rune_id:
- 1:
    ~ return "sun"
- 2:
    ~ return "moon"
- else:
    ~ return "star"
}

== function rune_choice_label(rune_id: int) => string ==
{ switch rune_id:
- 1:
    ~ return "Sun"
- 2:
    ~ return "Moon"
- else:
    ~ return "Star"
}
