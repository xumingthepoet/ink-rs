=== module game ===
VAR groups: string[][] = [["A", "B"], ["C"]]

== main ==
* [i, group in groups] Group {i}
    { for item in group:
    Item {item}.
    }
    -> DONE
