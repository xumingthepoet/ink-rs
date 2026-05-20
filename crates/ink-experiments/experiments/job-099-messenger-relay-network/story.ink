=== module game ===

STRUCT RelayStation {
    name: string
    queue_delay: int
    security: int
    jitter: int
}

STRUCT RelayHop {
    from_index: int
    to_index: int
    transit_delay: int
    weather_delay: int
    probe_risk: int
}

STRUCT Incident {
    hop_number: int
    station: string
    delay_penalty: int
    risk_penalty: int
    repair_cycles: int
}

VAR stations: RelayStation[] = [
    %RelayStation{ name: "Relay-A", queue_delay: 2, security: 82, jitter: 1 },
    %RelayStation{ name: "Relay-B", queue_delay: 3, security: 68, jitter: 2 },
    %RelayStation{ name: "Relay-C", queue_delay: 4, security: 61, jitter: 3 },
    %RelayStation{ name: "Relay-D", queue_delay: 2, security: 74, jitter: 1 },
    %RelayStation{ name: "Relay-E", queue_delay: 1, security: 88, jitter: 1 }
]

VAR hops: RelayHop[] = [
    %RelayHop{ from_index: 0, to_index: 1, transit_delay: 4, weather_delay: 1, probe_risk: 14 },
    %RelayHop{ from_index: 1, to_index: 2, transit_delay: 5, weather_delay: 2, probe_risk: 18 },
    %RelayHop{ from_index: 2, to_index: 3, transit_delay: 6, weather_delay: 3, probe_risk: 23 },
    %RelayHop{ from_index: 3, to_index: 4, transit_delay: 4, weather_delay: 1, probe_risk: 16 }
]

VAR incidents: Incident[] = [
    %Incident{
        hop_number: 2,
        station: "Relay-B",
        delay_penalty: 4,
        risk_penalty: 12,
        repair_cycles: 2
    }
]

VAR station_online: Dict<string, bool> = %{
    "Relay-A": true,
    "Relay-B": true,
    "Relay-C": true,
    "Relay-D": true,
    "Relay-E": true
}

VAR station_load: Dict<string, int> = %{
    "Relay-A": 1,
    "Relay-B": 2,
    "Relay-C": 3,
    "Relay-D": 1,
    "Relay-E": 0
}

VAR station_hops: Dict<string, int> = %{
    "Relay-A": 0,
    "Relay-B": 0,
    "Relay-C": 0,
    "Relay-D": 0,
    "Relay-E": 0
}

VAR station_exposure: Dict<string, int> = %{
    "Relay-A": 0,
    "Relay-B": 0,
    "Relay-C": 0,
    "Relay-D": 0,
    "Relay-E": 0
}

VAR total_delay: int = 0
VAR cumulative_risk: int = 0
VAR max_hop_risk: int = 0
VAR intercepted_attempts: int = 0
VAR compromised_hops: int = 0
VAR incidents_seen: int = 0
VAR repairs_completed: int = 0
VAR repair_cycles_spent: int = 0

VAR pending_delay_penalty: int = 0
VAR pending_risk_penalty: int = 0

== main ==
Messenger relay network simulation.
~ run_hops(0)
~ print_summary()
-> DONE

== function run_hops(index: int) => void ==
{ if index >= LEN(hops):
    Relay traversal complete.
- else:
    ~ temp hop_number: int = index + 1
    ~ pending_delay_penalty = 0
    ~ pending_risk_penalty = 0
    ~ apply_incidents(hop_number, 0)
    ~ process_hop(index)
    ~ complete_incident_repairs(hop_number, 0)
    ~ run_hops(index + 1)
}

== function apply_incidents(hop_number: int, index: int) => void ==
{ if index >= LEN(incidents):
    ~ return
- else:
    ~ temp incident: Incident = incidents[index]
    { if incident.hop_number == hop_number:
        ~ station_online[incident.station] = false
        ~ pending_delay_penalty = pending_delay_penalty + incident.delay_penalty
        ~ pending_risk_penalty = pending_risk_penalty + incident.risk_penalty
        ~ incidents_seen = incidents_seen + 1
        Incident on hop {hop_number}: {incident.station} outage.
    }
    ~ apply_incidents(hop_number, index + 1)
}

== function process_hop(index: int) => void ==
~ temp hop_number: int = index + 1
~ temp hop: RelayHop = hops[index]
~ temp from_station: RelayStation = stations[hop.from_index]
~ temp to_station: RelayStation = stations[hop.to_index]
~ temp from_name: string = from_station.name
~ temp to_name: string = to_station.name

~ temp load: int = station_load[from_name]
~ temp online: bool = station_online[from_name]

~ temp delay: int = hop.transit_delay + hop.weather_delay + from_station.queue_delay + load + pending_delay_penalty
~ temp risk: int = hop.probe_risk + ((100 - from_station.security) / 6) + from_station.jitter + pending_risk_penalty

{ if online == false:
    ~ delay = delay + 6
    ~ risk = risk + 14
}

~ total_delay = total_delay + delay
~ cumulative_risk = cumulative_risk + risk
{ if risk > max_hop_risk:
    ~ max_hop_risk = risk
}

~ station_hops[from_name] = station_hops[from_name] + 1
~ station_exposure[from_name] = station_exposure[from_name] + risk

~ station_load[from_name] = clamp_non_negative(load - 1)
~ station_load[to_name] = station_load[to_name] + 1

Hop {hop_number}: {from_name} to {to_name}
Delay this hop: {delay}
Interception risk this hop: {risk}

{ if risk >= 45:
    ~ intercepted_attempts = intercepted_attempts + 1
    ~ compromised_hops = compromised_hops + 1
    Interception status: channel compromised.
- else:
    { if risk >= 30:
        ~ intercepted_attempts = intercepted_attempts + 1
        Interception status: active probe detected.
    - else:
        Interception status: no active probe.
    }
}

== function complete_incident_repairs(hop_number: int, index: int) => void ==
{ if index >= LEN(incidents):
    ~ return
- else:
    ~ temp incident: Incident = incidents[index]
    { if incident.hop_number == hop_number:
        ~ station_online[incident.station] = true
        ~ repairs_completed = repairs_completed + 1
        ~ repair_cycles_spent = repair_cycles_spent + incident.repair_cycles
        Repair complete for {incident.station} after {incident.repair_cycles} cycles.
    }
    ~ complete_incident_repairs(hop_number, index + 1)
}

== function print_summary() => void ==
Relay network summary:
Total delay: {total_delay}
Cumulative interception risk: {cumulative_risk}
Max hop risk: {max_hop_risk}
Intercepted attempts: {intercepted_attempts}
Compromised hops: {compromised_hops}
Incidents seen: {incidents_seen}
Repairs completed: {repairs_completed}
Repair cycles spent: {repair_cycles_spent}
~ print_station_summary(0)
~ print_delivery_outcome()

== function print_station_summary(index: int) => void ==
{ if index >= LEN(stations):
    ~ return
- else:
    ~ temp station: RelayStation = stations[index]
    ~ temp name: string = station.name
    ~ temp load: int = station_load[name]
    ~ temp hops_handled: int = station_hops[name]
    ~ temp exposure: int = station_exposure[name]
    ~ temp online: bool = station_online[name]
    {name}: online {online}, hops {hops_handled}, exposure {exposure}, load {load}
    ~ print_station_summary(index + 1)
}

== function print_delivery_outcome() => void ==
{ if compromised_hops == 0:
    { if total_delay <= 40:
        Delivery outcome: delivered intact and on schedule.
    - else:
        { if total_delay <= 60:
            Delivery outcome: delivered intact but delayed.
        - else:
            Delivery outcome: delivery failed due to extreme delay.
        }
    }
- else:
    { if compromised_hops <= 1:
        { if total_delay <= 60:
            Delivery outcome: delivered with delays and partial exposure.
        - else:
            Delivery outcome: delivery failed after compromised routing delay.
        }
    - else:
        Delivery outcome: delivery failed due to repeated interception compromise.
    }
}

== function clamp_non_negative(value: int) => int ==
{ if value < 0:
    ~ return 0
- else:
    ~ return value
}
