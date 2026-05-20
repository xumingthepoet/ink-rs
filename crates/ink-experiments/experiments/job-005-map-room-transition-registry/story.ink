=== module game ===
VAR route_history: int[] = [10, 20, 30, 40]
VAR room_names: Dict<int, string> = %{10: "Crossroads", 20: "Ruins", 30: "Armory", 40: "Sanctum"}
VAR room_targets: Dict<int, ->> = %{
    10: -> room_crossroads,
    20: -> room_ruins,
    30: -> room_armory,
    40: -> room_sanctum
}
VAR next_route_step: int = 0

== main ==
Map route registry playback begins.
-> travel_route(0) ->
-> DONE

== travel_route(step: int) ==
{ if step >= LEN(route_history):
    Route finished.
    ->->
- else:
    ~ temp room_id: int = route_history[step]
    Entering route node {room_id} ({room_names[room_id]}).
    ~ next_route_step = step + 1
    ~ temp next: -> = room_targets[room_id]
    -> {next}
}

== resume_route ==
-> travel_route(next_route_step)

== room_crossroads ==
Crossroads is damp and open to east.
-> resume_route

== room_ruins ==
Ruins are quiet, and the old gates still stand.
-> resume_route

== room_armory ==
Armory offers stacked spears and broken armor.
-> resume_route

== room_sanctum ==
Sanctum seals are sealed with a bright green glyph.
-> resume_route
