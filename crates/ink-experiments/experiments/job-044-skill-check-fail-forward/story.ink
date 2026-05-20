=== module game ===

STRUCT Check {
    name: string
    actor: string
    skill: int
    roll: int
    difficulty: int
    consequence: string
}

VAR check_sequence: Check[] = [
    %Check{
        name: "Rope Traverse",
        actor: "Scout Lina",
        skill: 7,
        roll: 6,
        difficulty: 9,
        consequence: "You keep moving toward the gate."
    },
    %Check{
        name: "Silent Entry",
        actor: "Sage Venn",
        skill: 9,
        roll: 2,
        difficulty: 10,
        consequence: "You cut the night watch off at one alley."
    },
    %Check{
        name: "Lockpick Bolt",
        actor: "Rook",
        skill: 8,
        roll: 1,
        difficulty: 10,
        consequence: "The mechanism gives under pressure."
    },
    %Check{
        name: "Arc Bridge Run",
        actor: "Scout Lina",
        skill: 7,
        roll: 4,
        difficulty: 11,
        consequence: "You cross before the alarm cycle changes."
    },
    %Check{
        name: "Crowd Bluff",
        actor: "Sage Venn",
        skill: 10,
        roll: 4,
        difficulty: 12,
        consequence: "The crowd is confused and gives way."
    }
]

VAR morale: int = 7
VAR stamina: int = 12
VAR supplies: int = 6
VAR injuries: int = 0
VAR success_count: int = 0
VAR partial_count: int = 0
VAR fail_forward_count: int = 0

== main ==
Failure is expected. Failure-forward is the plan.
~ run_check(check_sequence[0], 1)
~ run_check(check_sequence[1], 2)
~ run_check(check_sequence[2], 3)
~ run_check(check_sequence[3], 4)
~ run_check(check_sequence[4], 5)
~ campaign_report()
-> DONE

== function run_check(check: Check, index: int) => void ==
-- Check {index}: {check.name} --
{check.actor} attempts a difficulty {check.difficulty} roll.
~ temp total: int = check.skill + check.roll
~ temp margin: int = total - check.difficulty
Roll outcome: base {check.skill} + roll {check.roll} = {total}, margin {margin}.
{ if margin >= 4:
    Full success.
    ~ success_count = success_count + 1
    ~ morale = morale + 2
    ~ supplies = supplies + 1
    ~ stamina = stamina + 1
    Reward unlocked: {check.consequence}
- else:
    { if margin >= 0:
        Partial success.
        ~ partial_count = partial_count + 1
        ~ morale = morale + 1
        ~ stamina = stamina + 0
        ~ supplies = supplies - 1
        You keep going, but the plan is slower.
    - else:
        Fail-forward.
        ~ fail_forward_count = fail_forward_count + 1
        ~ morale = morale - 1
        ~ stamina = stamina - 2
        ~ injuries = injuries + 1
        ~ supplies = supplies - 1
        You fail the strict target but still trigger {check.consequence}
    }
}
~ clamp_state()

== function clamp_state() => void ==
{ if morale < 0:
    ~ morale = 0
- else:
    { if morale > 20:
        ~ morale = 20
    }
}
{ if stamina < 0:
    ~ stamina = 0
- else:
    { if stamina > 20:
        ~ stamina = 20
    }
}
{ if supplies < 0:
    ~ supplies = 0
}

== function campaign_report() => void ==
Final state:
Checks: {success_count} full, {partial_count} partial, {fail_forward_count} fail-forward.
Morale: {morale}
Stamina: {stamina}
Supplies: {supplies}
Injuries: {injuries}
{ if morale >= 10:
    Your team keeps enough coherence for a steady campaign.
- else:
    { if morale >= 6:
        Tension is high, but momentum remains.
    - else:
        Morale is cracking. You should expect another complication.
    }
}
