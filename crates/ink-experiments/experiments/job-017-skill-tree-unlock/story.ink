=== module game ===
STRUCT Skill {
    id: string
    name: string
    required_points: int
    prerequisite: int
    unlocked: bool
}

VAR skill_points: int = 3
VAR points_spent: int = 0

VAR skill_tree: Skill[] = [
    %Skill{
        id: "blade",
        name: "Blade Core",
        required_points: 0,
        prerequisite: -1,
        unlocked: true
    },
    %Skill{
        id: "surge",
        name: "Surge Strike",
        required_points: 2,
        prerequisite: 0,
        unlocked: false
    },
    %Skill{
        id: "whirl",
        name: "Whirlblade",
        required_points: 2,
        prerequisite: 1,
        unlocked: false
    },
    %Skill{
        id: "finale",
        name: "Blazing Finale",
        required_points: 2,
        prerequisite: 2,
        unlocked: false
    }
]

== main ==
Current skill points available: {points_remaining()}
~ print_tree("initial")
~ try_unlock(3)
~ gain_points(1)
~ try_unlock(1)
~ try_unlock(2)
~ try_unlock(3)
~ gain_points(3)
~ print_tree("after training")
~ unlock_chain(3)
~ print_tree("final")
-> DONE

== function points_remaining() => int ==
~ return skill_points - points_spent

== function can_unlock(index: int) => bool ==
~ temp skill: Skill = skill_tree[index]
{ if skill.unlocked:
    ~ return false
- else:
    { if skill.prerequisite == -1:
        ~ return points_remaining() >= skill.required_points
    - else:
        { if !skill_tree[skill.prerequisite].unlocked:
            ~ return false
        - else:
            ~ return points_remaining() >= skill.required_points
        }
    }
}

== function gain_points(amount: int) => void ==
~ skill_points = skill_points + amount
Gained {amount} training points. Total available: {points_remaining()}.

== function apply_unlock(index: int) => void ==
~ temp skill: Skill = skill_tree[index]
~ skill_tree[index].unlocked = true
~ points_spent = points_spent + skill.required_points
{skill.name} unlocked.
Remaining points: {points_remaining()}.

== function try_unlock(index: int) => void ==
~ temp skill: Skill = skill_tree[index]
{ if can_unlock(index):
    ~ apply_unlock(index)
- else:
    { if skill.unlocked:
        {skill.name} is already unlocked.
    - else:
        { if skill.prerequisite != -1:
            { if !skill_tree[skill.prerequisite].unlocked:
                {skill.name} is locked by prerequisite {skill_tree[skill.prerequisite].name}.
            - else:
                Not enough points for {skill.name}. Requires {skill.required_points}, remaining {points_remaining()}.
            }
        - else:
            Not enough points for {skill.name}. Requires {skill.required_points}, remaining {points_remaining()}.
        }
    }
}

== function unlock_chain(index: int) => void ==
~ temp skill: Skill = skill_tree[index]
{ if skill.unlocked:
    {skill.name} was already unlocked.
- else:
    { if can_unlock(index):
        ~ apply_unlock(index)
    - else:
        { if skill.prerequisite != -1:
            ~ unlock_chain(skill.prerequisite)
            { if can_unlock(index):
                ~ apply_unlock(index)
            - else:
                {skill.name} cannot be unlocked yet.
            }
        - else:
            {skill.name} cannot be unlocked yet.
        }
    }
}

== function print_tree(label: string) => void ==
-- {label} --
Budget: {points_remaining()} / {skill_points}
~ print_skill(0)

== function print_skill(index: int) => void ==
{ if index < LEN(skill_tree):
    ~ temp skill: Skill = skill_tree[index]
    { if skill.unlocked:
        [open] {skill.id} - {skill.name}
    - else:
        [closed] {skill.id} - {skill.name}
    }
    ~ print_skill(index + 1)
}
