=== module game ===
VAR topics: string[] = ["East", "West"]
VAR details: string[][] = [["E1", "E2"], ["W1"]]

== main ==
* Topic menu
    ** [i, topic in topics] Topic {i}:{topic}
        topic {topic}.
        *** [detail in details[i]] Detail {topic}-{detail}
            detail {topic}:{detail}.
            -> DONE
