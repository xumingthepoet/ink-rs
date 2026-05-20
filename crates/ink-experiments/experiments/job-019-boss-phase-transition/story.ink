=== module game ===
STRUCT Boss {
    name: string
    hp: int
    phase: int
    phase_name: string
}

VAR boss: Boss = %Boss{
    name: "Obsidian Behemoth",
    hp: 120,
    phase: 0,
    phase_name: "calm"
}

VAR phase_one_triggered: bool = false
VAR phase_two_triggered: bool = false
VAR phase_three_triggered: bool = false
VAR phase_four_triggered: bool = false

== main ==
Boss phase transition test.
Phase {boss.phase} ({boss.phase_name}), hp {boss.hp}.
~ report("initial")
~ hit(30)
~ report("after first attack")
~ hit(35)
~ report("after second attack")
~ hit(40)
~ report("after third attack")
~ hit(20)
~ report("after final strike")
-> DONE

== function hit(amount: int) => void ==
~ temp next_hp: int = boss.hp - amount
{ if next_hp < 0:
    ~ boss.hp = 0
- else:
    ~ boss.hp = next_hp
}
~ apply_hp_phases()

== function apply_hp_phases() => void ==
Hero strike.
{ if boss.hp <= 80 && !phase_one_triggered:
    ~ phase_one_triggered = true
    ~ boss.phase = 1
    ~ boss.phase_name = "berserk"
    Phase 1: phase shift unlocked.
}
{ if boss.hp <= 60 && !phase_two_triggered:
    ~ phase_two_triggered = true
    ~ boss.phase = 2
    ~ boss.phase_name = "reckless"
    Phase 2: enrage and counter-attack.
}
{ if boss.hp <= 35 && !phase_three_triggered:
    ~ phase_three_triggered = true
    ~ boss.phase = 3
    ~ boss.phase_name = "desperate"
    Phase 3: rage bloom at close range.
}
{ if boss.hp <= 10 && !phase_four_triggered:
    ~ phase_four_triggered = true
    ~ boss.phase = 4
    ~ boss.phase_name = "final surge"
    Phase 4: final surge ignites.
}
{ if boss.hp == 0 && phase_four_triggered:
    Defeated.
}

== function report(label: string) => void ==
-- {label} --
{boss.name} at {boss.hp} hp, phase {boss.phase} ({boss.phase_name}).
~ report_phases()

== function report_phases() => void ==
{ if phase_one_triggered:
    Phase 1 trigger: berserk roar.
- else:
    Phase 1 not reached.
}
{ if phase_two_triggered:
    Phase 2 trigger: recklessness.
- else:
    Phase 2 not reached.
}
{ if phase_three_triggered:
    Phase 3 trigger: desperation.
- else:
    Phase 3 not reached.
}
{ if phase_four_triggered:
    Phase 4 trigger: final surge.
- else:
    Phase 4 not reached.
}
