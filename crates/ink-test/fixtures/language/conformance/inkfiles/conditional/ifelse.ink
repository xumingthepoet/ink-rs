=== module game ===
        VAR x: int = 0
        VAR y: int = 3

== main ==
        { x > 0:
            ~ y = x - 1
        - else:
            ~ y = x + 1
        }
        The value is {y}. -> END