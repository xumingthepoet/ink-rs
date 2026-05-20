=== module siege ===

STRUCT RationPolicy {
    name: string
    food_per_unit: int
    water_per_unit: int
    morale_delta: int
    disease_delta: int
}

STRUCT SiegeDay {
    day: int
    policy: int
    forage_food: int
    hauled_water: int
    contamination: int
    bombardment: int
}

VAR policies: RationPolicy[] = [
    %RationPolicy{name: "Full Ration", food_per_unit: 3, water_per_unit: 3, morale_delta: 2, disease_delta: -1},
    %RationPolicy{name: "Measured Ration", food_per_unit: 2, water_per_unit: 2, morale_delta: 0, disease_delta: 2},
    %RationPolicy{name: "Hard Ration", food_per_unit: 1, water_per_unit: 1, morale_delta: -3, disease_delta: 5}
]

VAR schedule: SiegeDay[] = [
    %SiegeDay{day: 1, policy: 0, forage_food: 20, hauled_water: 20, contamination: 0, bombardment: 1},
    %SiegeDay{day: 2, policy: 1, forage_food: 14, hauled_water: 12, contamination: 2, bombardment: 3},
    %SiegeDay{day: 3, policy: 1, forage_food: 8, hauled_water: 10, contamination: 4, bombardment: 4},
    %SiegeDay{day: 4, policy: 2, forage_food: 6, hauled_water: 5, contamination: 6, bombardment: 5},
    %SiegeDay{day: 5, policy: 2, forage_food: 4, hauled_water: 3, contamination: 8, bombardment: 6},
    %SiegeDay{day: 6, policy: 1, forage_food: 10, hauled_water: 6, contamination: 3, bombardment: 4},
    %SiegeDay{day: 7, policy: 1, forage_food: 2, hauled_water: 2, contamination: 7, bombardment: 7}
]

VAR stock_tags: Dict<string, int> = %{
    "food": 240,
    "water": 260
}

VAR defender_units: int = 24
VAR morale: int = 70
VAR disease_risk: int = 18
VAR food_shortage_total: int = 0
VAR water_shortage_total: int = 0
VAR surrender_pressure: int = 0
VAR breakout_pressure: int = 0
VAR disease_peak: int = 18
VAR morale_floor: int = 70

== main ==
Siege rationing ledger begins.
~ run_days(0)
~ print_final_assessment()
-> DONE

== function run_days(index: int) => void ==
{ if index >= LEN(schedule):
    Siege ration cycle complete.
- else:
    ~ temp day: SiegeDay = schedule[index]
    ~ process_day(day)
    ~ run_days(index + 1)
}

== function process_day(day: SiegeDay) => void ==
~ temp policy: RationPolicy = policies[day.policy]
Day {day.day} policy: {policy.name}
~ stock_tags["food"] = stock_tags["food"] + day.forage_food
~ stock_tags["water"] = stock_tags["water"] + day.hauled_water
Incoming stores: food +{day.forage_food}, water +{day.hauled_water}

~ temp required_food: int = policy.food_per_unit * defender_units
~ temp required_water: int = policy.water_per_unit * defender_units
~ temp served_food: int = min_two(stock_tags["food"], required_food)
~ temp served_water: int = min_two(stock_tags["water"], required_water)
~ temp shortage_food: int = required_food - served_food
~ temp shortage_water: int = required_water - served_water

~ stock_tags["food"] = stock_tags["food"] - served_food
~ stock_tags["water"] = stock_tags["water"] - served_water

~ food_shortage_total = food_shortage_total + shortage_food
~ water_shortage_total = water_shortage_total + shortage_water

~ temp morale_shift: int = policy.morale_delta - day.bombardment - shortage_penalty(shortage_food, shortage_water)
~ morale = clamp_hundred(morale + morale_shift)
~ temp disease_shift: int = policy.disease_delta + day.contamination + (day.bombardment / 3) + (shortage_food / 4) + (shortage_water / 4)
~ disease_risk = clamp_hundred(disease_risk + disease_shift)

~ surrender_pressure = surrender_pressure + (day.bombardment / 2)
{ if shortage_food > 0 || shortage_water > 0:
    ~ surrender_pressure = surrender_pressure + 4 + (shortage_food / 8) + (shortage_water / 8)
}
{ if morale < 45:
    ~ surrender_pressure = surrender_pressure + 2
}
{ if disease_risk > 60:
    ~ surrender_pressure = surrender_pressure + 2
}

~ breakout_pressure = breakout_pressure + breakout_gain(shortage_food, shortage_water, day.bombardment)

{ if disease_risk > disease_peak:
    ~ disease_peak = disease_risk
}
{ if morale < morale_floor:
    ~ morale_floor = morale
}

Rations required: food {required_food}, water {required_water}
Rations served: food {served_food}, water {served_water}
Shortages: food {shortage_food}, water {shortage_water}
Morale now: {morale}
Disease risk now: {disease_risk}
Surrender pressure: {surrender_pressure}
Breakout pressure: {breakout_pressure}
Stores remaining: food {stock_tags["food"]}, water {stock_tags["water"]}

== function print_final_assessment() => void ==
Siege final assessment.
Remaining food: {stock_tags["food"]}
Remaining water: {stock_tags["water"]}
Total food shortage: {food_shortage_total}
Total water shortage: {water_shortage_total}
Morale floor: {morale_floor}
Current morale: {morale}
Disease peak: {disease_peak}
Current disease risk: {disease_risk}
Surrender pressure: {surrender_pressure}
Breakout pressure: {breakout_pressure}
~ print_outcome()

== function print_outcome() => void ==
{ if surrender_pressure >= 32 || morale <= 18:
    Outcome: council accepts surrender terms.
- else:
    { if disease_risk >= 70 && morale <= 35 && (stock_tags["food"] <= 10 || stock_tags["water"] <= 10):
        Outcome: garrison attempts a breakout at dusk.
    - else:
        Outcome: siege lines hold through ration discipline.
    }
}

== function shortage_penalty(food_shortage: int, water_shortage: int) => int ==
~ temp penalty: int = (food_shortage / 4) + (water_shortage / 4)
{ if penalty > 6:
    ~ return 6
- else:
    ~ return penalty
}

== function breakout_gain(food_shortage: int, water_shortage: int, bombardment: int) => int ==
~ temp gain: int = (food_shortage / 3) + (water_shortage / 3) + (bombardment / 2)
{ if morale < 40:
    ~ gain = gain + 1
}
~ return gain

== function clamp_hundred(value: int) => int ==
{ if value < 0:
    ~ return 0
- else:
    { if value > 100:
        ~ return 100
    - else:
        ~ return value
    }
}

== function min_two(a: int, b: int) => int ==
{ if a < b:
    ~ return a
- else:
    ~ return b
}
