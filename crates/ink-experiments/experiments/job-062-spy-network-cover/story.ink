=== module game ===

STRUCT SpyAgent {
    name: string
    cover: int
    suspicion: int
    mission: string
    status: string
}

STRUCT SpyMission {
    operative: string
    label: string
    difficulty: int
    cover_cost: int
    cover_gain: int
    suspicion_pressure: int
}

VAR agents: SpyAgent[] = [
    %SpyAgent{
        name: "Nia Voss",
        cover: 16,
        suspicion: 3,
        mission: "Standby",
        status: "covered"
    },
    %SpyAgent{
        name: "Kade Morrow",
        cover: 12,
        suspicion: 5,
        mission: "Standby",
        status: "covered"
    },
    %SpyAgent{
        name: "Liora Haze",
        cover: 10,
        suspicion: 7,
        mission: "Standby",
        status: "watchful"
    },
    %SpyAgent{
        name: "Rooken Vale",
        cover: 8,
        suspicion: 2,
        mission: "Standby",
        status: "covered"
    }
]

VAR mission_log: SpyMission[] = [
    %SpyMission{
        operative: "Nia Voss",
        label: "Canal Courier",
        difficulty: 11,
        cover_cost: 3,
        cover_gain: 5,
        suspicion_pressure: 2
    },
    %SpyMission{
        operative: "Kade Morrow",
        label: "Council Archive Copy",
        difficulty: 14,
        cover_cost: 4,
        cover_gain: 4,
        suspicion_pressure: 4
    },
    %SpyMission{
        operative: "Liora Haze",
        label: "Iron Gate Watch Shift",
        difficulty: 13,
        cover_cost: 2,
        cover_gain: 2,
        suspicion_pressure: 3
    },
    %SpyMission{
        operative: "Nia Voss",
        label: "Border Ledger Drop",
        difficulty: 10,
        cover_cost: 2,
        cover_gain: 3,
        suspicion_pressure: 1
    },
    %SpyMission{
        operative: "Rooken Vale",
        label: "False Permit Planting",
        difficulty: 16,
        cover_cost: 5,
        cover_gain: 5,
        suspicion_pressure: 5
    }
]

VAR successful_missions: int = 0
VAR partial_missions: int = 0
VAR failed_missions: int = 0
VAR exposure_events: int = 0

== main ==
Spy network coverage run begins.
~ print_agents("Initial")
~ conduct_missions(0)
~ print_agents("Final")
~ print_network_summary()
-> DONE

== function print_agents(label: string) => void ==
Network report: {label}
~ print_agent_rows(0)

== function print_agent_rows(index: int) => void ==
{ if index >= LEN(agents):
    ~ return
- else:
    ~ temp agent: SpyAgent = agents[index]
    {agent.name} | cover {agent.cover} | suspicion {agent.suspicion} | mission "{agent.mission}" | exposure {agent.status}
    ~ print_agent_rows(index + 1)
}

== function conduct_missions(index: int) => void ==
{ if index >= LEN(mission_log):
    ~ return
- else:
    ~ temp mission: SpyMission = mission_log[index]
    ~ temp mission_no: int = index + 1
    Mission {mission_no}: {mission.label}
    {mission.label} starts for {mission.operative}.
    ~ temp operative_index: int = locate_operative(mission.operative, 0)
    { if operative_index == -1:
        No operative matches {mission.operative}. Mission aborted.
    - else:
        ~ process_mission(operative_index, mission)
    }
    ~ conduct_missions(index + 1)
}

== function locate_operative(name: string, index: int) => int ==
{ if index >= LEN(agents):
    ~ return -1
- else:
    { if agents[index].name == name:
        ~ return index
    - else:
        ~ return locate_operative(name, index + 1)
    }
}

== function process_mission(operative_index: int, mission: SpyMission) => void ==
~ temp operative: SpyAgent = agents[operative_index]
~ temp readiness: int = operative.cover + mission.cover_gain - mission.difficulty - operative.suspicion

~ temp post_cover: int = operative.cover - mission.cover_cost
~ temp post_suspicion: int = operative.suspicion + mission.suspicion_pressure

{ if readiness >= 11:
    {mission.label} executed with clear cover.
    ~ successful_missions = successful_missions + 1
    ~ post_cover = post_cover + 4
    ~ post_suspicion = post_suspicion + 1
- else:
    { if readiness >= 6:
        {mission.label} partly covered, but heat is rising.
        ~ partial_missions = partial_missions + 1
        ~ post_cover = post_cover + 1
        ~ post_suspicion = post_suspicion + 3
    - else:
        {mission.label} compromised the cell.
        ~ failed_missions = failed_missions + 1
        ~ post_cover = post_cover - 1
        ~ post_suspicion = post_suspicion + 7
    }
}

~ temp final_cover: int = clamp_cover(post_cover)
~ temp final_suspicion: int = clamp_suspicion(post_suspicion)
~ temp before_status: string = operative.status
~ temp after_status: string = exposure_level(final_cover, final_suspicion)
~ agents[operative_index].cover = final_cover
~ agents[operative_index].suspicion = final_suspicion
~ agents[operative_index].mission = mission.label
~ agents[operative_index].status = after_status
{ if after_status == "burned":
    ~ exposure_events = exposure_events + 1
}
Mission outcome:
{mission.operative}: {before_status} to {after_status}
Cover delta {final_cover - operative.cover}, suspicion delta {final_suspicion - operative.suspicion}.
~ print_agent_rows_single(operative_index)

== function print_agent_rows_single(index: int) => void ==
{ if index >= LEN(agents):
    ~ return
- else:
    ~ temp agent: SpyAgent = agents[index]
    {agent.name}: cover {agent.cover}, suspicion {agent.suspicion}, status {agent.status}
}

== function print_network_summary() => void ==
Operational summary:
Completed clean missions: {successful_missions}
Partial missions: {partial_missions}
Compromised missions: {failed_missions}
Exposure events: {exposure_events}
~ temp covered: int = count_status("covered", 0)
~ temp watchful: int = count_status("watchful", 0)
~ temp compromised: int = count_status("compromised", 0)
~ temp burned: int = count_status("burned", 0)
Status tally:
Covered {covered}
Watchful {watchful}
Compromised {compromised}
Burned {burned}
~ temp total_cover: int = sum_cover(0)
~ temp total_suspicion: int = sum_suspicion(0)
~ temp avg_cover: int = total_cover / LEN(agents)
~ temp avg_suspicion: int = total_suspicion / LEN(agents)
Average cover: {avg_cover}, average suspicion: {avg_suspicion}.
{ if burned > 0:
    Heat remains on the network. Immediate counter-intelligence is required.
- else:
    { if compromised > 0:
        A layer of suspicion remains. Contain and rotate coverage plans.
    - else:
        Network remains stable with manageable watch.
    }
}

== function count_status(target: string, index: int) => int ==
{ if index >= LEN(agents):
    ~ return 0
- else:
    { if agents[index].status == target:
        ~ return 1 + count_status(target, index + 1)
    - else:
        ~ return count_status(target, index + 1)
    }
}

== function sum_cover(index: int) => int ==
{ if index >= LEN(agents):
    ~ return 0
- else:
    ~ return agents[index].cover + sum_cover(index + 1)
}

== function sum_suspicion(index: int) => int ==
{ if index >= LEN(agents):
    ~ return 0
- else:
    ~ return agents[index].suspicion + sum_suspicion(index + 1)
}

== function exposure_level(cover: int, suspicion: int) => string ==
{ if suspicion >= 16:
    ~ return "burned"
- else:
    { if suspicion >= 10:
        ~ return "compromised"
    - else:
        { if suspicion >= 6:
            ~ return "watchful"
        - else:
            { if cover >= 12:
                ~ return "covered"
            - else:
                ~ return "watchful"
            }
        }
    }
}

== function clamp_cover(value: int) => int ==
{ if value < 0:
    ~ return 0
- else:
    { if value > 20:
        ~ return 20
    - else:
        ~ return value
    }
}

== function clamp_suspicion(value: int) => int ==
{ if value < 0:
    ~ return 0
- else:
    { if value > 20:
        ~ return 20
    - else:
        ~ return value
    }
}
