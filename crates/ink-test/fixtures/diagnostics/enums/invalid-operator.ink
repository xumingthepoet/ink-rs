=== module game ===
ENUM State { Idle Busy }
VAR ordered: bool = State.Idle > State.Busy

== main ==
