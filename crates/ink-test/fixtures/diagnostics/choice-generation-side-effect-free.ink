=== module game ===

EXTERNAL unsafe_label() => string
EXTERNAL unsafe_items() => string[]

VAR options: string[] = ["Ada"]

== function mark(value: bool) => bool ==
~ return value

== main ==
* {mark(true)}: Function condition
    Picked.
* Random {RANDOM(1, 10)}
    Picked.
* Label {unsafe_label()}
    Picked.
* [item in unsafe_items()] Item {item}
    Picked.
* [item in options] Item {unsafe_label()}
    Picked.
