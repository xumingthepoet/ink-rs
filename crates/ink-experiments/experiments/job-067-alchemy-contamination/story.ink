=== module game ===

STRUCT Reagent {
    name: string
    purity: int
    contamination: int
}

STRUCT BrewPlan {
    potion: string
    core: string
    binder: string
    catalyst: string
    heat: int
}

VAR reagents: Reagent[] = [
    %Reagent{name: "Moonflower Petal", purity: 86, contamination: 8},
    %Reagent{name: "Iron Bloom", purity: 74, contamination: 12},
    %Reagent{name: "Nightwater", purity: 68, contamination: 14},
    %Reagent{name: "Willow Bark", purity: 91, contamination: 4},
    %Reagent{name: "Copper Salt", purity: 79, contamination: 7}
]

VAR reagent_index: Dict<string, int> = %{
    "Moonflower Petal": 0,
    "Iron Bloom": 1,
    "Nightwater": 2,
    "Willow Bark": 3,
    "Copper Salt": 4
}

VAR brew_plan: BrewPlan[] = [
    %BrewPlan{
        potion: "Heal Draught",
        core: "Moonflower Petal",
        binder: "Willow Bark",
        catalyst: "Copper Salt",
        heat: 4
    },
    %BrewPlan{
        potion: "Nightfall Tonic",
        core: "Nightwater",
        binder: "Iron Bloom",
        catalyst: "Copper Salt",
        heat: 7
    },
    %BrewPlan{
        potion: "Wound Salve",
        core: "Iron Bloom",
        binder: "Moonflower Petal",
        catalyst: "Willow Bark",
        heat: 5
    }
]

VAR contamination_pressure: int = 10
VAR stable_outcomes: int = 0
VAR unstable_outcomes: int = 0
VAR failed_outcomes: int = 0
VAR question_outcomes: int = 0

== main ==
Alchemist lab log starts.
~ print_lab_roster()
~ brew_all(0)
~ print_lab_report()
-> DONE

== function brew_all(index: int) => void ==
{ if index >= LEN(brew_plan):
    Brewing cycle complete.
- else:
    ~ temp batch: BrewPlan = brew_plan[index]
    Brewing {batch.potion}
    ~ temp stability: int = run_brew(batch)
    ~ temp outcome: string = classify_stability(stability)
    ~ print_mixture_result(batch.potion, stability, outcome)
    ~ record_outcome(outcome)
    ~ brew_all(index + 1)
}

== function run_brew(batch: BrewPlan) => int ==
~ temp core: Reagent = reagents[reagent_index[batch.core]]
~ temp binder: Reagent = reagents[reagent_index[batch.binder]]
~ temp catalyst: Reagent = reagents[reagent_index[batch.catalyst]]
~ temp core_purity: int = core.purity
~ temp binder_purity: int = binder.purity
~ temp catalyst_purity: int = catalyst.purity
~ temp core_contam: int = core.contamination
~ temp binder_contam: int = binder.contamination
~ temp catalyst_contam: int = catalyst.contamination
~ temp raw_purity: int = (core_purity + binder_purity + catalyst_purity) / 3
~ temp contamination_sum: int = core_contam + binder_contam + catalyst_contam + contamination_pressure
~ temp heat: int = batch.heat * 3
~ temp stability: int = raw_purity - contamination_sum + heat
~ temp outcome: string = classify_stability(stability)

Reagent profile before boil:
{core.name}: purity {core_purity}, contamination {core_contam}
{binder.name}: purity {binder_purity}, contamination {binder_contam}
{catalyst.name}: purity {catalyst_purity}, contamination {catalyst_contam}
Base stability: {stability}, profile: {outcome}

~ adjust_reagents_for_batch(batch.core, batch.binder, batch.catalyst, stability)

{ if contamination_pressure > 24:
    { if stability >= 75:
        ~ contamination_pressure = contamination_pressure - 2
    - else:
        ~ contamination_pressure = contamination_pressure - 1
    }
- else:
    ~ contamination_pressure = contamination_pressure + 2
}
~ clamp_pressure()
~ return stability

== function adjust_reagents_for_batch(core_name: string, binder_name: string, catalyst_name: string, stability: int) => void ==
~ temp core: Reagent = reagents[reagent_index[core_name]]
~ temp binder: Reagent = reagents[reagent_index[binder_name]]
~ temp catalyst: Reagent = reagents[reagent_index[catalyst_name]]
~ adjust_single_reagent(core_name, core, stability)
~ adjust_single_reagent(binder_name, binder, stability)
~ adjust_single_reagent(catalyst_name, catalyst, stability)

== function adjust_single_reagent(name: string, reagent: Reagent, stability: int) => void ==
{ if stability >= 70:
    ~ temp cleaner: int = reagent.contamination - 2
    { if cleaner < 0:
        ~ cleaner = 0
    }
    ~ temp purity_gain: int = reagent.purity + 1
    { if purity_gain > 100:
        ~ purity_gain = 100
    }
    ~ reagent.contamination = cleaner
    ~ reagent.purity = purity_gain
    {name} rests and recovers.
- else:
    { if stability >= 35:
        ~ temp dirty: int = reagent.contamination + 1
        { if dirty > 40:
            ~ dirty = 40
        }
        ~ reagent.contamination = dirty
        ~ temp purity_loss: int = reagent.purity - 1
        { if purity_loss < 0:
            ~ purity_loss = 0
        }
        ~ reagent.purity = purity_loss
        {name} strains under heat.
    - else:
        ~ temp dirty: int = reagent.contamination + 3
        { if dirty > 60:
            ~ dirty = 60
        }
        ~ reagent.contamination = dirty
        ~ temp purity_loss: int = reagent.purity - 2
        { if purity_loss < 0:
            ~ purity_loss = 0
        }
        ~ reagent.purity = purity_loss
        {name} chars and degrades.
    }
}

== function classify_stability(stability: int) => string ==
{ if stability >= 80:
    ~ return "Pure"
- else:
    { if stability >= 60:
        ~ return "Potent"
    - else:
        { if stability >= 35:
            ~ return "Unstable"
        - else:
            ~ return "Failed"
        }
    }
}

== function print_mixture_result(potion: string, stability: int, outcome: string) => void ==
Potion: {potion}
Stability score: {stability}
Outcome: {outcome}
~ print_reagent_state(potion)

== function print_reagent_state(potion: string) => void ==
{ if potion == "Heal Draught":
    ~ print_selected_reagents("Moonflower Petal", "Willow Bark", "Copper Salt")
- else:
    { if potion == "Nightfall Tonic":
        ~ print_selected_reagents("Nightwater", "Iron Bloom", "Copper Salt")
    - else:
        ~ print_selected_reagents("Iron Bloom", "Moonflower Petal", "Willow Bark")
    }
}

== function print_selected_reagents(a: string, b: string, c: string) => void ==
~ temp r1: Reagent = reagents[reagent_index[a]]
~ temp r2: Reagent = reagents[reagent_index[b]]
~ temp r3: Reagent = reagents[reagent_index[c]]
Post-brew reagent states:
{r1.name}: purity {r1.purity}, contamination {r1.contamination}
{r2.name}: purity {r2.purity}, contamination {r2.contamination}
{r3.name}: purity {r3.purity}, contamination {r3.contamination}
Ambient pressure: {contamination_pressure}

== function record_outcome(outcome: string) => void ==
{ if outcome == "Pure":
    ~ stable_outcomes = stable_outcomes + 1
- else:
    { if outcome == "Potent":
        ~ unstable_outcomes = unstable_outcomes + 1
    - else:
        { if outcome == "Unstable":
            ~ question_outcomes = question_outcomes + 1
        - else:
            ~ failed_outcomes = failed_outcomes + 1
        }
    }
}

== function print_lab_roster() => void ==
Reagent inventory:
~ print_reagent(0)

== function print_reagent(index: int) => void ==
{ if index >= LEN(reagents):
    ~ return
- else:
    ~ temp reagent: Reagent = reagents[index]
    {reagent.name}: purity {reagent.purity}, contamination {reagent.contamination}
    ~ print_reagent(index + 1)
}

== function print_lab_report() => void ==
Alchemy lab report.
Outcomes:
Pure: {stable_outcomes}
Potent: {unstable_outcomes}
Unstable: {question_outcomes}
Failed: {failed_outcomes}
Ambient contamination pressure: {contamination_pressure}
~ print_reagent(0)

== function clamp_pressure() => void ==
{ if contamination_pressure < 0:
    ~ contamination_pressure = 0
- else:
    { if contamination_pressure > 60:
        ~ contamination_pressure = 60
    - else:
        ~ return
    }
}
