=== module prison ===

STRUCT PatrolWindow {
    zone: string
    start_hour: int
    end_hour: int
    note: string
}

STRUCT DistractionWindow {
    zone: string
    start_hour: int
    end_hour: int
    note: string
}

STRUCT EscapeRoute {
    name: string
    zone: string
    required_key: string
    difficulty: int
}

VAR patrol_windows: PatrolWindow[] = [
    %PatrolWindow{
        zone: "inner_cell",
        start_hour: 2,
        end_hour: 5,
        note: "Cell block checks every quarter-hour sweep."
    },
    %PatrolWindow{
        zone: "maintenance_bay",
        start_hour: 6,
        end_hour: 7,
        note: "Tool room round when the locks rotate."
    },
    %PatrolWindow{
        zone: "service_tunnel",
        start_hour: 6,
        end_hour: 8,
        note: "Tunnel patrol walks with flash markers."
    },
    %PatrolWindow{
        zone: "shoreline_ramp",
        start_hour: 10,
        end_hour: 12,
        note: "Dock watch shifts near the launch deck."
    }
]

VAR distraction_windows: DistractionWindow[] = [
    %DistractionWindow{
        zone: "inner_cell",
        start_hour: 5,
        end_hour: 6,
        note: "Cell alarm reset drill masks footsteps."
    },
    %DistractionWindow{
        zone: "service_tunnel",
        start_hour: 7,
        end_hour: 8,
        note: "Pipe burst alert pulls patrol attention away."
    },
    %DistractionWindow{
        zone: "shoreline_ramp",
        start_hour: 12,
        end_hour: 13,
        note: "Distant thud on hull delays dock report."
    }
]

VAR escape_routes: EscapeRoute[] = [
    %EscapeRoute{
        name: "Inner Cell Tunnel",
        zone: "inner_cell",
        required_key: "warden_key",
        difficulty: 1
    },
    %EscapeRoute{
        name: "Service Tunnel",
        zone: "service_tunnel",
        required_key: "maintenance_key",
        difficulty: 2
    },
    %EscapeRoute{
        name: "Shoreline Ramp",
        zone: "shoreline_ramp",
        required_key: "deck_key",
        difficulty: 3
    }
]

VAR key_inventory: Dict<string, bool> = %{
    "warden_key": false,
    "maintenance_key": false,
    "deck_key": false
}

VAR attempt_count: int = 0
VAR successful_escapes: int = 0

== main ==
Prison break simulation.
~ print_schedule()
~ attempt_escape_route(0, 4)
~ collect_warden_key(4)
~ collect_warden_key(5)
~ attempt_escape_route(0, 5)
~ attempt_escape_route(1, 6)
~ collect_maintenance_key(6)
~ collect_maintenance_key(7)
~ attempt_escape_route(1, 7)
~ attempt_escape_route(2, 11)
~ collect_deck_key(11)
~ collect_deck_key(12)
~ attempt_escape_route(2, 12)
~ print_summary()
-> DONE

== function print_schedule() => void ==
Patrol and distraction map for the shift.
~ print_patrol_windows(0)
~ print_distraction_windows(0)
~ print_routes(0)

== function print_patrol_windows(index: int) => void ==
{ if index < LEN(patrol_windows):
    ~ temp window: PatrolWindow = patrol_windows[index]
    { window.zone } patrol {window.start_hour}-{window.end_hour}: {window.note}
    ~ print_patrol_windows(index + 1)
}

== function print_distraction_windows(index: int) => void ==
{ if index < LEN(distraction_windows):
    ~ temp window: DistractionWindow = distraction_windows[index]
    { window.zone } distraction {window.start_hour}-{window.end_hour}: {window.note}
    ~ print_distraction_windows(index + 1)
}

== function print_routes(index: int) => void ==
{ if index < LEN(escape_routes):
    ~ temp route: EscapeRoute = escape_routes[index]
    Route {route.name} to {route.zone}, key {route.required_key}, base risk {route.difficulty}
    ~ print_routes(index + 1)
}

== function attempt_escape_route(route_index: int, hour: int) => void ==
~ attempt_count = attempt_count + 1
~ temp route: EscapeRoute = escape_routes[route_index]
Attempt {attempt_count}: {route.name} at hour {hour}
~ print_zone_state(route.zone, hour)
~ temp key_ready: bool = key_inventory[route.required_key]
{ if !key_ready:
    {route.required_key} is missing. Route cannot be started.
- else:
    ~ temp risk: int = route_risk(route, hour)
    { if risk <= 3:
        ~ successful_escapes = successful_escapes + 1
        Route cleared. Escape can proceed toward {route.name}.
    - else:
        Guards catch the delay and the route becomes dangerous.
    }
    Risk score: {risk}
}

== function route_risk(route: EscapeRoute, hour: int) => int ==
~ temp risk: int = route.difficulty
{ if is_guarded(route.zone, hour):
    ~ risk = risk + 2
}
{ if is_distracted(route.zone, hour):
    ~ risk = risk - 1
}
~ return risk

== function print_zone_state(zone: string, hour: int) => void ==
{ if is_guarded(zone, hour):
    Patrol noise and light checks remain in {zone}.
- else:
    No active guards in {zone} at hour {hour}.
}
{ if is_distracted(zone, hour):
    {zone} is currently under a distraction cue.
- else:
    No active distraction in {zone}.
}

== function collect_warden_key(hour: int) => void ==
Warden key drill at hour {hour}.
{ if key_inventory["warden_key"]:
    Warden key already held.
- else:
    { if hour >= 2 && hour < 6:
        { if is_guarded("inner_cell", hour):
            Guard visibility blocks inner_cell collection.
        - else:
            ~ key_inventory["warden_key"] = true
            Warden key retrieved.
        }
    - else:
        The inner_cell lock is not exposed at hour {hour}.
    }
}

== function collect_maintenance_key(hour: int) => void ==
Maintenance key drill at hour {hour}.
{ if key_inventory["maintenance_key"]:
    Maintenance key already held.
- else:
    { if hour >= 7 && hour < 10:
        { if is_guarded("maintenance_bay", hour):
            Maintenance routine is too visible at hour {hour}.
        - else:
            ~ key_inventory["maintenance_key"] = true
            Maintenance key retrieved.
        }
    - else:
        The maintenance cache is inaccessible at hour {hour}.
    }
}

== function collect_deck_key(hour: int) => void ==
Deck key drill at hour {hour}.
{ if key_inventory["deck_key"]:
    Deck key already held.
- else:
    { if hour >= 11 && hour < 14:
        { if is_guarded("shoreline_ramp", hour):
            Dock control blocks deck key access at hour {hour}.
        - else:
            ~ key_inventory["deck_key"] = true
            Deck key retrieved.
        }
    - else:
        The deck lock chamber is not open at hour {hour}.
    }
}

== function is_guarded(zone: string, hour: int) => bool ==
~ return is_guarded_at(zone, hour, 0)

== function is_guarded_at(zone: string, hour: int, index: int) => bool ==
{ if index >= LEN(patrol_windows):
    ~ return false
- else:
    ~ temp patrol: PatrolWindow = patrol_windows[index]
    { if patrol.zone == zone && hour >= patrol.start_hour && hour < patrol.end_hour:
        ~ return true
    - else:
        ~ return is_guarded_at(zone, hour, index + 1)
    }
}

== function is_distracted(zone: string, hour: int) => bool ==
~ return is_distracted_at(zone, hour, 0)

== function is_distracted_at(zone: string, hour: int, index: int) => bool ==
{ if index >= LEN(distraction_windows):
    ~ return false
- else:
    ~ temp distraction: DistractionWindow = distraction_windows[index]
    { if distraction.zone == zone && hour >= distraction.start_hour && hour < distraction.end_hour:
        ~ return true
    - else:
        ~ return is_distracted_at(zone, hour, index + 1)
    }
}

== function print_summary() => void ==
Escape attempts: {attempt_count}
Escapes cleared: {successful_escapes}
-- Keys --
{ if key_inventory["warden_key"]:
    warden_key: secured
- else:
    warden_key: unavailable
}
{ if key_inventory["maintenance_key"]:
    maintenance_key: secured
- else:
    maintenance_key: unavailable
}
{ if key_inventory["deck_key"]:
    deck_key: secured
- else:
    deck_key: unavailable
}
-- Overall Outcome --
{ if successful_escapes == 3:
    All routes were made walkable with the right timing.
- else:
    { if successful_escapes == 2:
        One route remained blocked by timing or access.
    - else:
        Multiple routes were compromised by guard pressure.
    }
}
