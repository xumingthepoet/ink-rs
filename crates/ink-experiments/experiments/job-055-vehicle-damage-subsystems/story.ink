=== module game ===

STRUCT Starship {
    name: string
    engine: int
    hull: int
    sensors: int
}

VAR ship: Starship = %Starship{
    name: "Arius Cutter",
    engine: 2,
    hull: 1,
    sensors: 3
}

VAR repair_tokens: int = 8

== main ==
Preflight systems for {ship.name}.
~ status_report("Pre-damage baseline")
~ field_damage()
~ status_report("After debris strike")
~ storm_damage()
~ status_report("After ion storm")
~ allocate_repairs(repair_tokens)
~ status_report("After allocated repairs")
~ mission_assessment()
-> DONE

== function field_damage() => void ==
~ ship.engine = ship.engine + 3
~ ship.hull = ship.hull + 2
~ ship.sensors = ship.sensors + 1
Hull plating scrapes. Engine and hull absorb direct impact.
Engine damage {ship.engine}.
Hull damage {ship.hull}.
Sensor damage {ship.sensors}.

== function storm_damage() => void ==
~ ship.sensors = ship.sensors + 3
~ ship.hull = ship.hull + 2
Plasma rain punches through thin plating.
Sensors now {ship.sensors}; hull rises to {ship.hull}.

== function allocate_repairs(points: int) => void ==
{ if points <= 0:
    ~ return
- else:
    ~ allocate_one_point()
    ~ allocate_repairs(points - 1)
}

== function allocate_one_point() => void ==
~ temp target: string = repair_target()
{ if target == "engine":
    ~ ship.engine = ship.engine - 1
    Repair to engine. Remaining damage {ship.engine}.
- else:
    { if target == "hull":
        ~ ship.hull = ship.hull - 1
        Repair to hull. Remaining damage {ship.hull}.
    - else:
        ~ ship.sensors = ship.sensors - 1
        Repair to sensors. Remaining damage {ship.sensors}.
    }
}

== function repair_target() => string ==
~ temp engine_load: int = ship.engine * 5
~ temp hull_load: int = ship.hull * 4
~ temp sensor_load: int = ship.sensors * 3

{ if engine_load >= hull_load:
    { if engine_load >= sensor_load:
        { if ship.engine > 0:
            ~ return "engine"
        - else:
            { if hull_load >= sensor_load:
                { if ship.hull > 0:
                    ~ return "hull"
                - else:
                    ~ return "sensors"
                }
            - else:
                ~ return "sensors"
            }
        }
    - else:
        { if ship.sensors > 0:
            ~ return "sensors"
        - else:
            ~ return "hull"
        }
    }
- else:
    { if hull_load > sensor_load:
        { if ship.hull > 0:
            ~ return "hull"
        - else:
            { if ship.engine > 0:
                ~ return "engine"
            - else:
                ~ return "sensors"
            }
        }
    - else:
        { if ship.sensors > 0:
            ~ return "sensors"
        - else:
            { if ship.engine > 0:
                ~ return "engine"
            - else:
                ~ return "hull"
            }
        }
    }
}

== function status_report(phase: string) => void ==
Status {phase}:
Engine damage: {ship.engine}
Hull damage: {ship.hull}
Sensor damage: {ship.sensors}
Readiness score: {readiness_score()}
Estimated readiness: {readiness_band()}

== function readiness_score() => int ==
~ return 100 - ship.engine * 5 - ship.hull * 4 - ship.sensors * 3

== function readiness_band() => string ==
~ temp score: int = readiness_score()
{ if score >= 75:
    ~ return "Green - full launch window."
- else:
    { if score >= 55:
        ~ return "Amber - launch, then slow and conservative."
    - else:
        ~ return "Red - launch unsafe."
    }
}

== function mission_assessment() => void ==
~ temp band: string = readiness_band()
Mission status:
{ if band == "Green - full launch window.":
    Full readiness.
    You launch at combat thrust, full systems online.
- else:
    { if band == "Amber - launch, then slow and conservative.":
        Reduced launch profile accepted.
        You keep range low and avoid sensor-only scouting.
    - else:
        Launch scrapped.
        You require major repair before attempting the mission.
    }
}
Crew recommendation:
{ if readiness_score() >= 60:
    Keep the mission on schedule.
- else:
    Delay one cycle and reroute to repair dock.
}
