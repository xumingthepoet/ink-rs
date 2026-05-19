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
    { name: "Strike", kind: "skill", enabled: true, needs_target: true },
    { name: "Fireball", kind: "skill", enabled: false, needs_target: true },
    { name: "Potion", kind: "item", enabled: true, needs_target: false }
]
VAR targets: Target[] = [
    { name: "Slime A", hp: 12, alive: true },
    { name: "Slime B", hp: 8, alive: true },
    { name: "Slime C", hp: 0, alive: false }
]

== main ==
Turn start.
Focus: {focus}
-> dynamic_choices(LEN(actions), -> action_option, -1, 0)

== dynamic_choices(count: int, render: ->, context: int, index: int) ==
{ if index >= count:
    -> DONE
- else:
    <- {render}(context, index)
    -> dynamic_choices(count, render, context, index + 1)
}

== action_option(context: int, index: int) ==
* {actions[index].enabled}: {actions[index].name} ({actions[index].kind})
    -> choose_action(index)

== choose_action(action_index: int) ==
{ if actions[action_index].needs_target:
    Choose target for {actions[action_index].name}.
    -> dynamic_choices(LEN(targets), -> target_option, action_index, 0)
- else:
    -> resolve_action(action_index, -1)
}

== target_option(action_index: int, index: int) ==
* {targets[index].alive}: {targets[index].name} ({targets[index].hp} hp)
    -> resolve_action(action_index, index)

== resolve_action(action_index: int, target_index: int) ==
{ if target_index >= 0:
    {actor_name} uses {actions[action_index].name} on {targets[target_index].name}.
- else:
    {actor_name} uses {actions[action_index].name}.
}
-> END
