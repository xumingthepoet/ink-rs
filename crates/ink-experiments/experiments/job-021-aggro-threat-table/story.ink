=== module game ===
STRUCT ThreatActor {
    id: string
    threat: int
    alive: bool
}

STRUCT ThreatAction {
    source: string
    target: string
    delta: int
}

VAR threat_table: ThreatActor[] = [
    %ThreatActor{id: "goblin", threat: 0, alive: true},
    %ThreatActor{id: "skeleton", threat: 0, alive: true},
    %ThreatActor{id: "golem", threat: 0, alive: true},
    %ThreatActor{id: "shadow", threat: 0, alive: true}
]

VAR threat_events: ThreatAction[] = [
    %ThreatAction{source: "Rogue", target: "goblin", delta: 8},
    %ThreatAction{source: "Mage", target: "skeleton", delta: 8},
    %ThreatAction{source: "Healer", target: "golem", delta: 10},
    %ThreatAction{source: "Rogue", target: "skeleton", delta: 3},
    %ThreatAction{source: "Mage", target: "goblin", delta: 2},
    %ThreatAction{source: "Rogue", target: "shadow", delta: 15},
    %ThreatAction{source: "Healer", target: "golem", delta: 5}
]

== main ==
Aggro threat table probe.
~ print_threat_table("Initial")
~ process_threat_events(0)
~ print_threat_table("After actions")
~ mark_dead("skeleton")
~ print_top_target()
~ print_threat_table("After silence on skeleton")
-> DONE

== function process_threat_events(index: int) => void ==
{ if index < LEN(threat_events):
    ~ temp action: ThreatAction = threat_events[index]
    {action.source} applies {action.delta} threat to {action.target}.
    ~ temp target_index: int = find_actor_index(action.target, 0)
    { if target_index == -1:
        Unknown actor {action.target}.
    - else:
        ~ threat_table[target_index].threat = threat_table[target_index].threat + action.delta
        {action.target} now at {threat_table[target_index].threat} threat.
        ~ print_top_target()
    }
    ~ process_threat_events(index + 1)
}

== function mark_dead(id: string) => void ==
~ temp dead_index: int = find_actor_index(id, 0)
{ if dead_index == -1:
    {id} was not found.
- else:
    {id} drops from the fight.
    ~ threat_table[dead_index].alive = false
}

== function print_top_target() => void ==
~ temp winner: int = select_top_threat_target()
{ if winner == -1:
    No viable threat target.
- else:
    Current aggro target is {threat_table[winner].id} with {threat_table[winner].threat} threat.
}

== function print_threat_table(label: string) => void ==
Threat table: {label}
~ print_threat_rows(0)

== function print_threat_rows(index: int) => void ==
{ if index < LEN(threat_table):
    ~ temp actor: ThreatActor = threat_table[index]
    { if actor.alive:
        {actor.id}: threat {actor.threat} alive.
    - else:
        {actor.id}: threat {actor.threat} down.
    }
    ~ print_threat_rows(index + 1)
}

== function select_top_threat_target() => int ==
~ temp initial: int = first_alive_actor(0)
{ if initial == -1:
    ~ return -1
- else:
    ~ return best_threat_actor(initial, initial + 1)
}

== function first_alive_actor(index: int) => int ==
{ if index >= LEN(threat_table):
    ~ return -1
- else:
    { if threat_table[index].alive:
        ~ return index
    - else:
        ~ return first_alive_actor(index + 1)
    }
}

== function best_threat_actor(best: int, index: int) => int ==
{ if index >= LEN(threat_table):
    ~ return best
- else:
    { if !threat_table[index].alive:
        ~ return best_threat_actor(best, index + 1)
    - else:
        { if threat_table[index].threat > threat_table[best].threat:
            ~ return best_threat_actor(index, index + 1)
        - else:
            ~ return best_threat_actor(best, index + 1)
        }
    }
}

== function find_actor_index(id: string, index: int) => int ==
{ if index >= LEN(threat_table):
    ~ return -1
- else:
    { if threat_table[index].id == id:
        ~ return index
    - else:
        ~ return find_actor_index(id, index + 1)
    }
}
