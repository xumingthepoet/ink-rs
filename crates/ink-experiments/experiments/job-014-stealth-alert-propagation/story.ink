=== module game ===
STRUCT AlertArea {
    name: string
    alert: int
    neighbor_a: int
    neighbor_b: int
}

VAR alert_map: AlertArea[] = [
    %AlertArea{
        name: "Gatehouse",
        alert: 0,
        neighbor_a: 1,
        neighbor_b: -1
    },
    %AlertArea{
        name: "Courtyard",
        alert: 0,
        neighbor_a: 0,
        neighbor_b: 2
    },
    %AlertArea{
        name: "Barracks",
        alert: 0,
        neighbor_a: 1,
        neighbor_b: -1
    }
]

== main ==
Patrol noise spread test begins.
Area alert snapshot: Initial
~ temp gate_alert: int = alert_map[0].alert
~ temp courtyard_alert: int = alert_map[1].alert
~ temp barracks_alert: int = alert_map[2].alert

{ if gate_alert <= 0:
    Gatehouse: calm.
- else:
    { if gate_alert <= 3:
        Gatehouse: alert {gate_alert}.
    - else:
        Gatehouse: HIGH ALERT {gate_alert}.
    }
}
{ if courtyard_alert <= 0:
    Courtyard: calm.
- else:
    { if courtyard_alert <= 3:
        Courtyard: alert {courtyard_alert}.
    - else:
        Courtyard: HIGH ALERT {courtyard_alert}.
    }
}
{ if barracks_alert <= 0:
    Barracks: calm.
- else:
    { if barracks_alert <= 3:
        Barracks: alert {barracks_alert}.
    - else:
        Barracks: HIGH ALERT {barracks_alert}.
    }
}

Guards hear footsteps at the Gatehouse.
~ temp noise_step_a: int = propagate_noise(0, 3, -1, "soft footsteps")
Area alert snapshot: After Gatehouse footsteps
~ temp gate_alert: int = alert_map[0].alert
~ temp courtyard_alert: int = alert_map[1].alert
~ temp barracks_alert: int = alert_map[2].alert

{ if gate_alert <= 0:
    Gatehouse: calm.
- else:
    { if gate_alert <= 3:
        Gatehouse: alert {gate_alert}.
    - else:
        Gatehouse: HIGH ALERT {gate_alert}.
    }
}
{ if courtyard_alert <= 0:
    Courtyard: calm.
- else:
    { if courtyard_alert <= 3:
        Courtyard: alert {courtyard_alert}.
    - else:
        Courtyard: HIGH ALERT {courtyard_alert}.
    }
}
{ if barracks_alert <= 0:
    Barracks: calm.
- else:
    { if barracks_alert <= 3:
        Barracks: alert {barracks_alert}.
    - else:
        Barracks: HIGH ALERT {barracks_alert}.
    }
}

An extra crate falls in the Barracks.
~ alert_map[2].alert = alert_map[2].alert + 1
~ temp noise_step_b: int = propagate_noise(2, 3, -1, "barrel impact")
Area alert snapshot: After Barracks impact
~ temp gate_alert: int = alert_map[0].alert
~ temp courtyard_alert: int = alert_map[1].alert
~ temp barracks_alert: int = alert_map[2].alert

{ if gate_alert <= 0:
    Gatehouse: calm.
- else:
    { if gate_alert <= 3:
        Gatehouse: alert {gate_alert}.
    - else:
        Gatehouse: HIGH ALERT {gate_alert}.
    }
}
{ if courtyard_alert <= 0:
    Courtyard: calm.
- else:
    { if courtyard_alert <= 3:
        Courtyard: alert {courtyard_alert}.
    - else:
        Courtyard: HIGH ALERT {courtyard_alert}.
    }
}
{ if barracks_alert <= 0:
    Barracks: calm.
- else:
    { if barracks_alert <= 3:
        Barracks: alert {barracks_alert}.
    - else:
        Barracks: HIGH ALERT {barracks_alert}.
    }
}
-> DONE

== function propagate_noise(origin: int, volume: int, previous_area: int, cause: string) => int ==
~ alert_map[origin].alert = alert_map[origin].alert + volume
{ if volume <= 0:
    ~ return 0
- else:
    {alert_map[origin].name} hears the {cause}. Alert level {alert_map[origin].alert}.
    ~ temp next_volume: int = volume - 1
    { if alert_map[origin].neighbor_a != -1 && alert_map[origin].neighbor_a != previous_area:
        ~ temp _ignored: int = propagate_noise(alert_map[origin].neighbor_a, next_volume, origin, cause)
    }
    { if alert_map[origin].neighbor_b != -1 && alert_map[origin].neighbor_b != previous_area:
        ~ temp _ignored: int = propagate_noise(alert_map[origin].neighbor_b, next_volume, origin, cause)
    }
    ~ return 0
}
