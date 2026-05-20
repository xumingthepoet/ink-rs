=== module game ===

STRUCT Enemy {
    id: string
    hp: int
    stamina: int
    state: string
    attack_cd: int
    special_cd: int
    alive: bool
}

STRUCT Intention {
    slot: int
    enemy_id: string
    action: string
    priority: int
    cooldown_type: string
}

VAR enemies: Enemy[] = [
    %Enemy{id: "Varg", hp: 16, stamina: 7, state: "aggressive", attack_cd: 0, special_cd: 2, alive: true},
    %Enemy{id: "Shade", hp: 12, stamina: 6, state: "berserk", attack_cd: 1, special_cd: 0, alive: true},
    %Enemy{id: "Skewer", hp: 9, stamina: 5, state: "normal", attack_cd: 0, special_cd: 3, alive: true},
    %Enemy{id: "Pup", hp: 5, stamina: 4, state: "normal", attack_cd: 2, special_cd: 4, alive: true}
]

VAR intent_stack: Intention[] = [
    %Intention{slot: 0, enemy_id: "", action: "", priority: 0, cooldown_type: ""},
    %Intention{slot: 1, enemy_id: "", action: "", priority: 0, cooldown_type: ""},
    %Intention{slot: 2, enemy_id: "", action: "", priority: 0, cooldown_type: ""},
    %Intention{slot: 3, enemy_id: "", action: "", priority: 0, cooldown_type: ""}
]

VAR stack_depth: int = 0
VAR current_turn: int = 1
VAR hero_hp: int = 54

== main ==
Enemy AI intention stack simulation.
~ print_roster("Initial")
~ run_turn(1)
~ print_roster("Final")
Hero hp left: {hero_hp}.
-> DONE

== function run_turn(turn: int) => void ==
{ if turn > 4:
    Combat round complete.
- else:
    ~ current_turn = turn
    { if hero_hp <= 0:
        Hero defeated. Combat stops before Turn {turn}.
    - else:
        Turn {turn}
        ~ stack_depth = 0
        ~ build_intention_stack(0)
        ~ sort_plan(stack_depth - 1)
        ~ print_turn_plan()
        ~ execute_plan(stack_depth - 1)
        ~ print_turn_roster(turn)
        ~ apply_end_of_turn_effects(0)
        ~ run_turn(turn + 1)
    }
}

== function build_intention_stack(index: int) => void ==
{ if index < LEN(enemies):
    ~ temp enemy: Enemy = enemies[index]
    { if enemy.alive:
        ~ temp action: string = "guard"
        ~ temp priority: int = 35 + index
        ~ temp cooldown_type: string = ""
        { if enemy.hp <= 3:
            ~ action = "retreat"
            ~ priority = 95 + index
        - else:
            { if enemy.state == "berserk" && enemy.special_cd == 0 && enemy.stamina >= 4:
                ~ action = "rupture"
                ~ priority = 90 + index
                ~ cooldown_type = "special_cd"
            - else:
                { if enemy.attack_cd == 0 && enemy.stamina >= 3:
                    { if enemy.state == "aggressive":
                        ~ action = "cleave"
                    - else:
                        ~ action = "slash"
                    }
                    ~ priority = 75 + index
                    ~ cooldown_type = "attack_cd"
                - else:
                    { if enemy.stamina < 4:
                        ~ action = "rest"
                        ~ priority = 22 + index
                    - else:
                        ~ action = "guard"
                        ~ priority = 38 + index
                    }
                }
            }
        }
        ~ intent_stack[stack_depth].slot = index
        ~ intent_stack[stack_depth].enemy_id = enemy.id
        ~ intent_stack[stack_depth].action = action
        ~ intent_stack[stack_depth].priority = priority
        ~ intent_stack[stack_depth].cooldown_type = cooldown_type
        ~ stack_depth = stack_depth + 1
    - else:
        {enemy.id} is down.
    }
    ~ build_intention_stack(index + 1)
}

== function sort_plan(end: int) => void ==
{ if end > 0:
    ~ bubble_plan(0, end)
    ~ sort_plan(end - 1)
}

== function bubble_plan(index: int, end: int) => void ==
{ if index < end:
    ~ temp left: Intention = intent_stack[index]
    ~ temp right: Intention = intent_stack[index + 1]
    { if left.priority < right.priority:
        ~ intent_stack[index] = right
        ~ intent_stack[index + 1] = left
    }
    ~ bubble_plan(index + 1, end)
}

== function print_turn_plan() => void ==
-- Turn {current_turn} plan (highest first) --
~ print_plan_entry(0)

== function print_plan_entry(index: int) => void ==
{ if index < stack_depth:
    ~ temp intent: Intention = intent_stack[index]
    { if intent.action == "":
        {intent.enemy_id}
    - else:
        {intent.enemy_id} -> {intent.action} (priority {intent.priority})
    }
    ~ print_plan_entry(index + 1)
}

== function execute_plan(index: int) => void ==
{ if index >= 0:
    ~ temp intent: Intention = intent_stack[index]
    { if intent.action != "":
        ~ temp actor_index: int = find_enemy(intent.enemy_id, 0)
        { if actor_index == -1:
            Missing actor {intent.enemy_id}.
        - else:
            ~ temp actor: Enemy = enemies[actor_index]
            { if intent.action == "cleave":
                {actor.id} uses cleave.
                ~ hero_hp = hero_hp - 8
                ~ enemies[actor_index].stamina = actor.stamina - 2
                ~ enemies[actor_index].state = "aggressive"
                ~ enemies[actor_index].attack_cd = 2
            - else:
                { if intent.action == "slash":
                    {actor.id} uses slash.
                    ~ hero_hp = hero_hp - 6
                    ~ enemies[actor_index].stamina = actor.stamina - 1
                    ~ enemies[actor_index].state = "aggressive"
                    ~ enemies[actor_index].attack_cd = 2
                - else:
                    { if intent.action == "rupture":
                        {actor.id} uses rupture.
                        ~ hero_hp = hero_hp - 11
                        ~ enemies[actor_index].stamina = actor.stamina - 3
                        ~ enemies[actor_index].state = "berserk"
                        ~ enemies[actor_index].special_cd = 3
                    - else:
                        { if intent.action == "guard":
                            {actor.id} guards.
                            ~ enemies[actor_index].stamina = actor.stamina + 1
                            ~ enemies[actor_index].state = "guarding"
                        - else:
                            { if intent.action == "rest":
                                {actor.id} rests.
                                ~ enemies[actor_index].stamina = actor.stamina + 2
                                ~ enemies[actor_index].state = "resting"
                            - else:
                                {actor.id} retreats to recover.
                                ~ enemies[actor_index].stamina = actor.stamina + 1
                                ~ enemies[actor_index].state = "retreating"
                            }
                        }
                    }
                }
            }
            { if enemies[actor_index].stamina < 0:
                ~ enemies[actor_index].stamina = 0
            - else:
                { if enemies[actor_index].stamina > 8:
                    ~ enemies[actor_index].stamina = 8
                }
            }
        }
        { if hero_hp < 0:
            ~ hero_hp = 0
        }
    }
    ~ execute_plan(index - 1)
}

== function apply_end_of_turn_effects(index: int) => void ==
{ if index < LEN(enemies):
    ~ temp enemy: Enemy = enemies[index]
    { if enemy.alive:
        { if enemy.attack_cd > 0:
            ~ enemies[index].attack_cd = enemy.attack_cd - 1
        - else:
            ~ enemies[index].attack_cd = 0
        }
        { if enemy.special_cd > 0:
            ~ enemies[index].special_cd = enemy.special_cd - 1
        - else:
            ~ enemies[index].special_cd = 0
        }
        { if enemy.hp <= 0:
            ~ enemies[index].alive = false
            ~ enemies[index].state = "down"
        - else:
            { if enemy.state == "guarding":
                ~ enemies[index].state = "normal"
            - else:
                { if enemy.state == "resting":
                    ~ enemies[index].state = "normal"
                - else:
                    { if enemy.state == "retreating":
                        ~ enemies[index].state = "normal"
                    - else:
                        { if enemy.state == "critical":
                            ~ enemies[index].state = "critical"
                        - else:
                            { if enemy.hp < 4:
                                ~ enemies[index].state = "critical"
                            - else:
                                ~ enemies[index].state = "normal"
                            }
                        }
                    }
                }
            }
        }
        { if enemies[index].stamina > 8:
            ~ enemies[index].stamina = 8
        }
    }
    ~ apply_end_of_turn_effects(index + 1)
}

== function print_roster(label: string) => void ==
-- {label} --
~ print_enemy_entry(0)

== function print_turn_roster(turn: int) => void ==
Turn {turn} roster:
~ print_enemy_entry(0)

== function print_enemy_entry(index: int) => void ==
{ if index < LEN(enemies):
    ~ temp enemy: Enemy = enemies[index]
    { if enemy.alive:
        {enemy.id} | hp {enemy.hp} | stamina {enemy.stamina} | {enemy.state} | a{enemy.attack_cd} s{enemy.special_cd}
    - else:
        {enemy.id} | down
    }
    ~ print_enemy_entry(index + 1)
}

== function find_enemy(id: string, index: int) => int ==
{ if index >= LEN(enemies):
    ~ return -1
- else:
    { if enemies[index].id == id:
        ~ return index
    - else:
        ~ return find_enemy(id, index + 1)
    }
}
