=== module game ===
STRUCT Action {
    name: string
    kind: string
    enabled: bool
    needs_target: bool
}

STRUCT Target {
    name: string
    hp: int
    alive: bool
}

VAR actor_name: string = "Rin"
VAR focus: int = 5
VAR actions: Action[] = [
    %Action{ name: "Strike", kind: "skill", enabled: true, needs_target: true },
    %Action{ name: "Fireball", kind: "skill", enabled: false, needs_target: true },
    %Action{ name: "Potion", kind: "item", enabled: true, needs_target: false }
]
VAR targets: Target[] = [
    %Target{ name: "Slime A", hp: 12, alive: true },
    %Target{ name: "Slime B", hp: 8, alive: true },
    %Target{ name: "Slime C", hp: 0, alive: false }
]

== main ==
Turn start.
Focus: {focus}
* [action_index, action in actions] {action.enabled}: {action.name} ({action.kind})
    -> choose_action(action_index)

== choose_action(action_index: int) ==
{ if actions[action_index].needs_target:
    Choose target for {actions[action_index].name}.
    * [target_index, target in targets] {target.alive}: {target.name} ({target.hp} hp)
        -> resolve_action(action_index, target_index)
- else:
    -> resolve_action(action_index, -1)
}

== resolve_action(action_index: int, target_index: int) ==
{ if target_index >= 0:
    {actor_name} uses {actions[action_index].name} on {targets[target_index].name}.
- else:
    {actor_name} uses {actions[action_index].name}.
}
