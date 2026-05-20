=== module game ===

STRUCT NoiseSource {
    name: string
    zone: int
    volume: int
    active: bool
}

STRUCT Enemy {
    name: string
    zone: int
    awareness: int
    alert_threshold: int
    nearby: bool
}

STRUCT Door {
    name: string
    zone_a: int
    zone_b: int
    state: string
}

VAR noise_sources: NoiseSource[] = [
    %NoiseSource{name: "Soft heel scrape", zone: 0, volume: 2, active: false},
    %NoiseSource{name: "Metal hinge rattle", zone: 0, volume: 4, active: false},
    %NoiseSource{name: "Torch chain clink", zone: 1, volume: 3, active: false},
    %NoiseSource{name: "Falling crate", zone: 2, volume: 7, active: false}
]

VAR enemies: Enemy[] = [
    %Enemy{name: "Sentinel Rune", zone: 1, awareness: 14, alert_threshold: 55, nearby: false},
    %Enemy{name: "Rook Guard", zone: 2, awareness: 8, alert_threshold: 48, nearby: false},
    %Enemy{name: "Vault Warden", zone: 3, awareness: 3, alert_threshold: 65, nearby: false}
]

VAR doors: Door[] = [
    %Door{name: "Antechamber Gate", zone_a: 0, zone_b: 1, state: "closed"},
    %Door{name: "Iron Gate", zone_a: 1, zone_b: 2, state: "locked"},
    %Door{name: "Vault Door", zone_a: 2, zone_b: 3, state: "closed"}
]

VAR active_step: int = 1
VAR ambush_risk: int = 0

== main ==
An infiltration route is planned with quiet and caution.
~ report_state("Initial")
~ run_steps(1)
~ final_report()
-> DONE

== function run_steps(step: int) => void ==
{ if step > 4:
    ~ return
- else:
    ~ active_step = step
    ~ execute_step(step)
    ~ update_enemy_awareness(0)
    ~ ambush_risk = assess_ambush_risk(0)
    ~ report_state("After step")
    ~ decay_awareness(0)
    ~ clear_sources(0)
    ~ run_steps(step + 1)
}

== function execute_step(step: int) => void ==
{ if step == 1:
    You move from the stairs to the entrance walkway.
    ~ activate_source(0)
- else:
    { if step == 2:
        You brace the Antechamber Gate and pull it open.
        ~ set_door_state(0, "open")
        ~ activate_source(1)
    - else:
        { if step == 3:
            You force the Iron Gate and pull it open with effort.
            ~ set_door_state(1, "open")
            ~ activate_source(2)
        - else:
            { if step == 4:
                A crate shifts on a support beam above the vault hall.
                ~ activate_source(3)
            - else:
                ~ return
            }
        }
    }
}

== function update_enemy_awareness(enemy_index: int) => void ==
{ if enemy_index >= LEN(enemies):
    ~ return
- else:
    ~ apply_noise_sources(0, enemy_index)
    ~ update_enemy_awareness(enemy_index + 1)
}

== function apply_noise_sources(source_index: int, enemy_index: int) => void ==
{ if source_index >= LEN(noise_sources):
    ~ return
- else:
    { if noise_sources[source_index].active:
        ~ apply_noise_to_enemy(source_index, enemy_index)
    }
    ~ apply_noise_sources(source_index + 1, enemy_index)
}

== function apply_noise_to_enemy(source_index: int, enemy_index: int) => void ==
{ if source_index >= LEN(noise_sources):
    ~ return
- else:
    ~ temp source: NoiseSource = noise_sources[source_index]
    ~ temp enemy: Enemy = enemies[enemy_index]
    ~ temp spread: int = route_volume(source.zone, enemy.zone, source.volume)
    { if spread <= 0:
        {source.name} does not reach {enemy.name}.
    - else:
        ~ temp increased: int = enemy.awareness + spread
        { if spread >= 6:
            The noise is sharp and visible.
            ~ increased = increased + 4
        }
        { if increased > 100:
            ~ increased = 100
        }
        ~ enemies[enemy_index].awareness = increased
        ~ enemies[enemy_index].nearby = increased >= enemy.alert_threshold
        {source.name} reaches {enemy.name}, adding {spread} awareness.
    }
}

== function route_volume(source_zone: int, enemy_zone: int, volume: int) => int ==
{ if source_zone == enemy_zone:
    ~ return volume
- else:
    { if source_zone < enemy_zone:
        ~ return traverse_forward(source_zone, enemy_zone, volume)
    - else:
        ~ return traverse_backward(source_zone, enemy_zone, volume)
    }
}

== function traverse_forward(current: int, target: int, volume: int) => int ==
{ if current == target:
    ~ return volume
- else:
    ~ temp attenuated: int = apply_door_loss(volume, current)
    ~ return traverse_forward(current + 1, target, attenuated)
}

== function traverse_backward(current: int, target: int, volume: int) => int ==
{ if current == target:
    ~ return volume
- else:
    ~ temp attenuated: int = apply_door_loss(volume, current - 1)
    ~ return traverse_backward(current - 1, target, attenuated)
}

== function apply_door_loss(volume: int, door_index: int) => int ==
{ if doors[door_index].state == "open":
    ~ return volume
- else:
    { if doors[door_index].state == "closed":
        ~ return volume / 2
    - else:
        ~ return 0
    }
}

== function assess_ambush_risk(index: int) => int ==
{ if index >= LEN(enemies):
    ~ return 0
- else:
    ~ temp enemy: Enemy = enemies[index]
    ~ temp risk: int = 3

    { if enemy.awareness >= enemy.alert_threshold:
        ~ risk = 35
    - else:
        { if enemy.awareness >= enemy.alert_threshold - 12:
            ~ risk = 18
        }
    }

    { if enemy.nearby:
        ~ risk = risk + 8
    }

    ~ return risk + assess_ambush_risk(index + 1)
}

== function decay_awareness(index: int) => void ==
{ if index >= LEN(enemies):
    ~ return
- else:
    ~ temp enemy: Enemy = enemies[index]
    ~ temp decayed: int = enemy.awareness - 2
    { if decayed < 0:
        ~ decayed = 0
    }
    ~ enemies[index].awareness = decayed
    ~ enemies[index].nearby = false
    ~ decay_awareness(index + 1)
}

== function clear_sources(index: int) => void ==
{ if index >= LEN(noise_sources):
    ~ return
- else:
    ~ noise_sources[index].active = false
    ~ clear_sources(index + 1)
}

== function set_door_state(index: int, state: string) => void ==
~ temp old_state: string = doors[index].state
~ doors[index].state = state
~ temp door_name: string = doors[index].name
The {door_name} shifts from {old_state} to {state}.

== function activate_source(index: int) => void ==
~ noise_sources[index].active = true
~ temp source_name: string = noise_sources[index].name
Noise source {source_name} is active.

== function report_state(label: string) => void ==
Phase {active_step}: {label}
~ report_doors(0)
~ report_enemies(0)
Current ambush risk: {ambush_risk}
{ if ambush_risk >= 40:
    The route is compromised.
- else:
    { if ambush_risk >= 20:
        Alert is mounting.
    - else:
        Noise stays below alarm threshold.
    }
}

== function report_doors(index: int) => void ==
{ if index >= LEN(doors):
    ~ return
- else:
    ~ temp door: Door = doors[index]
    {door.name}: {door.state}
    ~ report_doors(index + 1)
}

== function report_enemies(index: int) => void ==
{ if index >= LEN(enemies):
    ~ return
- else:
    ~ temp enemy: Enemy = enemies[index]
    ~ temp zone: string = zone_name(enemy.zone)
    {enemy.name} ({zone}) awareness {enemy.awareness} = {nearby_label(enemy.nearby)}
    ~ report_enemies(index + 1)
}

== function nearby_label(nearby: bool) => string ==
{ if nearby:
    ~ return "nearby"
- else:
    ~ return "distant"
}

== function zone_name(zone: int) => string ==
{ if zone == 0:
    ~ return "entrance"
- else:
    { if zone == 1:
        ~ return "anteroom"
    - else:
        { if zone == 2:
            ~ return "vault hall"
        - else:
            { if zone == 3:
                ~ return "inner vault"
            - else:
                ~ return "unknown"
            }
        }
    }
}

== function final_report() => void ==
Infiltration ends.
~ report_state("Final")
