=== interface ISkillEffect ===
== apply(actor_id: int, target_slot: int) ==
== function label() => string ==

=== module skills ===
FROM spark_effect
FROM guard_effect
FROM strike_effect

STRUCT SkillDef {
    name: string
    mp_cost: int
    target_kind: string
}

CONST SKILL_SPARK: int = 1
CONST SKILL_GUARD: int = 2
CONST SKILL_STRIKE: int = 3
CONST skill_defs: Dict<int, SkillDef> = %{
    1: %SkillDef{ name: "Spark", mp_cost: 2, target_kind: "enemy" },
    2: %SkillDef{ name: "Guard", mp_cost: 0, target_kind: "self" },
    3: %SkillDef{ name: "Strike", mp_cost: 0, target_kind: "enemy" }
}
VAR handlers: Dict<int, interface<ISkillEffect>> = %{1: spark_effect, 2: guard_effect, 3: strike_effect}

== use_skill(skill_id: int, actor_id: int, target_slot: int) ==
Skill {skill_defs[skill_id].name} dispatches to {{handlers[skill_id]}::label()}.
-> {{handlers[skill_id]}::apply}(actor_id, target_slot) ->
->->

=== module spark_effect implements ISkillEffect ===
== function label() => string ==
~ return "spark effect"

== apply(actor_id: int, target_slot: int) ==
Spark hits enemy slot {target_slot}.
->->

=== module guard_effect implements ISkillEffect ===
== function label() => string ==
~ return "guard effect"

== apply(actor_id: int, target_slot: int) ==
Guard raises actor {actor_id}'s shield.
->->

=== module strike_effect implements ISkillEffect ===
== function label() => string ==
~ return "strike effect"

== apply(actor_id: int, target_slot: int) ==
Strike hits enemy slot {target_slot}.
->->
