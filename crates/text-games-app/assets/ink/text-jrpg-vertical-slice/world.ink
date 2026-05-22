=== module world ===

STRUCT LocationDef {
    name: string
    route: string
}

CONST LOC_VILLAGE: int = 1
CONST LOC_FOREST: int = 2
CONST LOC_GATE: int = 3
CONST LOC_MINE: int = 4

CONST locations: Dict<int, LocationDef> = %{
    1: %LocationDef{ name: "Brindle Village", route: "hub" },
    2: %LocationDef{ name: "Moonlit Forest", route: "field" },
    3: %LocationDef{ name: "Old Mine Gate", route: "gate" },
    4: %LocationDef{ name: "Old Mine", route: "dungeon" }
}

VAR current_location: int = LOC_VILLAGE

== function travel_to(location_id: int) => void ==
~ current_location = location_id

== function location_name(location_id: int) => string ==
~ return locations[location_id].name

== function current_location_name() => string ==
~ return location_name(current_location)

== function current_location_id() => int ==
~ return current_location

== function travel_summary() => string ==
~ temp text: string = ""
{ for location_id, location in locations:
    { if text == "":
        ~ text = location.name
    - else:
        ~ text = text + " -> " + location.name
    }
}
~ return text
