=== module game ===

STRUCT RouteLeg {
    origin: string
    destination: string
    distance: int
    terrain: string
}

VAR supplies: Dict<string, int> = %{
    "food": 42,
    "water": 28,
    "fuel": 19
}

VAR terrain_cost: Dict<string, int> = %{
    "plains": 1,
    "forest": 2,
    "desert": 4,
    "mountain": 3,
    "river": 2
}

VAR shortage_warnings: int = 0

VAR route: RouteLeg[] = [
    %RouteLeg{ origin: "Ridgepost", destination: "Bracken Trail", distance: 4, terrain: "plains" },
    %RouteLeg{ origin: "Bracken Trail", destination: "Sandscar", distance: 3, terrain: "desert" },
    %RouteLeg{ origin: "Sandscar", destination: "Stonevale", distance: 5, terrain: "mountain" },
    %RouteLeg{ origin: "Stonevale", destination: "Wilderford", distance: 2, terrain: "forest" },
    %RouteLeg{ origin: "Wilderford", destination: "Rivergate", distance: 4, terrain: "river" }
]

== main ==
Caravan route simulation.
~ print_supplies("Start")
~ travel(0)
~ print_supplies("Final supplies")
Supply warnings: {shortage_warnings}.
{ if shortage_warnings == 0:
    Caravan arrived without shortages.
- else:
    Review warnings before next leg.
}
-> DONE

== function travel(leg_index: int) => void ==
{ if leg_index < LEN(route):
    ~ temp leg: RouteLeg = route[leg_index]
    ~ print_leg(leg_index + 1, leg.origin, leg.destination, leg.distance, leg.terrain)
    ~ temp terrain_factor: int = terrain_cost[leg.terrain]
    ~ temp food_need: int = (leg.distance * 2) + terrain_factor
    ~ temp water_need: int = (leg.distance * 3) + terrain_factor
    ~ temp fuel_need: int = (leg.distance + terrain_factor)
    { if leg.terrain == "desert":
        ~ water_need = water_need + 6
    - else:
        { if leg.terrain == "mountain":
            ~ fuel_need = fuel_need + 4
            ~ food_need = food_need + 2
        - else:
            { if leg.terrain == "river":
                ~ fuel_need = fuel_need + 2
            }
        }
    }
    ~ consume_food(leg.origin, leg.destination, food_need)
    ~ consume_water(leg.origin, leg.destination, water_need)
    ~ consume_fuel(leg.origin, leg.destination, fuel_need)
    ~ print_supplies("After leg")
    ~ travel(leg_index + 1)
}

== function consume_food(origin: string, destination: string, amount: int) => void ==
~ temp available: int = supplies["food"]
{ if available >= amount:
    ~ supplies["food"] = available - amount
    Food {amount} used from {origin} to {destination}.
- else:
    ~ shortage_warnings = shortage_warnings + 1
    ~ supplies["food"] = 0
    Warning: food shortage on {origin} to {destination}.
    Needed {amount}, had {available}.
}

== function consume_water(origin: string, destination: string, amount: int) => void ==
~ temp available: int = supplies["water"]
{ if available >= amount:
    ~ supplies["water"] = available - amount
    Water {amount} used from {origin} to {destination}.
- else:
    ~ shortage_warnings = shortage_warnings + 1
    ~ supplies["water"] = 0
    Warning: water shortage on {origin} to {destination}.
    Needed {amount}, had {available}.
}

== function consume_fuel(origin: string, destination: string, amount: int) => void ==
~ temp available: int = supplies["fuel"]
{ if available >= amount:
    ~ supplies["fuel"] = available - amount
    Fuel {amount} used from {origin} to {destination}.
- else:
    ~ shortage_warnings = shortage_warnings + 1
    ~ supplies["fuel"] = 0
    Warning: fuel shortage on {origin} to {destination}.
    Needed {amount}, had {available}.
}

== function print_supplies(label: string) => void ==
-- {label} --
Food {supplies["food"]}
Water {supplies["water"]}
Fuel {supplies["fuel"]}

== function print_leg(step: int, origin: string, destination: string, distance: int, terrain: string) => void ==
Leg {step}: {origin} to {destination}, distance {distance}, terrain {terrain}
