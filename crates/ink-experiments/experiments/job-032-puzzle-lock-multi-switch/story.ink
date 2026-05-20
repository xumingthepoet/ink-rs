=== module game ===
STRUCT LockAttempt {
    label: string
    switch_0: int
    switch_1: int
    switch_2: int
}

VAR lock_solution: int[] = [1, 0, 2]
VAR lock_switches: int[] = [0, 0, 0]
VAR lock_attempts: LockAttempt[] = [
    %LockAttempt{label: "first pull", switch_0: 0, switch_1: 2, switch_2: 1},
    %LockAttempt{label: "backtrack", switch_0: 2, switch_1: 2, switch_2: 0},
    %LockAttempt{label: "aligned reset", switch_0: 1, switch_1: 0, switch_2: 2},
    %LockAttempt{label: "late guess", switch_0: 0, switch_1: 1, switch_2: 0},
    %LockAttempt{label: "panic try", switch_0: 2, switch_1: 1, switch_2: 2}
]
VAR switch_state_label: Dict<int, string> = %{
    0: "up",
    1: "middle",
    2: "down"
}
VAR lock_attempt_count: int = 0
VAR lock_solved: bool = false

== main ==
Multi-switch lock attempt sequence.
Current tumbler states: [{switch_state_label[0]}, {switch_state_label[1]}, {switch_state_label[2]}]
~ run_attempts(0)
{ if lock_solved:
    Lock opened in {lock_attempt_count} ordered attempts.
- else:
    Lock remains sealed after all {lock_attempt_count} attempts.
}
-> DONE

== function run_attempts(index: int) => void ==
{ if lock_solved:
    { if index < LEN(lock_attempts):
        Remaining attempts hidden by successful solve.
    }
- else:
    { if index >= LEN(lock_attempts):
        No further attempts remain.
    - else:
        ~ temp attempt: LockAttempt = lock_attempts[index]
        ~ lock_attempt_count = lock_attempt_count + 1
        Attempt {lock_attempt_count}: {attempt.label} sets [{switch_state_label[attempt.switch_0]}, {switch_state_label[attempt.switch_1]}, {switch_state_label[attempt.switch_2]}].
        ~ lock_switches[0] = attempt.switch_0
        ~ lock_switches[1] = attempt.switch_1
        ~ lock_switches[2] = attempt.switch_2
        Current tumblers: [{switch_state_label[lock_switches[0]]}, {switch_state_label[lock_switches[1]]}, {switch_state_label[lock_switches[2]]}]
        ~ temp matched: bool = lock_switches[0] == lock_solution[0] && lock_switches[1] == lock_solution[1] && lock_switches[2] == lock_solution[2]
        { if matched:
            Feedback: all switches align.
            ~ lock_solved = true
        - else:
            Feedback: no alignment.
        }
        { if !lock_solved:
            ~ run_attempts(index + 1)
        }
    }
}

