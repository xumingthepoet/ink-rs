=== module game ===

STRUCT ForageNode {
    name: string
    preferred_season: string
    base_yield: int
    spoilage_risk: int
    meal_value: int
}

VAR current_season: string = "Autumn"
VAR damp_weather: int = 15

VAR forage_nodes: ForageNode[] = [
    %ForageNode{ name: "Riverbank Reeds", preferred_season: "Autumn", base_yield: 18, spoilage_risk: 12, meal_value: 3 },
    %ForageNode{ name: "Pinecap Mushrooms", preferred_season: "Summer", base_yield: 15, spoilage_risk: 20, meal_value: 4 },
    %ForageNode{ name: "Frostroot Patch", preferred_season: "Winter", base_yield: 12, spoilage_risk: 10, meal_value: 5 },
    %ForageNode{ name: "Sunfruit Thicket", preferred_season: "Spring", base_yield: 16, spoilage_risk: 26, meal_value: 2 },
    %ForageNode{ name: "Stoneleaf Terrace", preferred_season: "Autumn", base_yield: 14, spoilage_risk: 8, meal_value: 4 }
]

VAR node_raw: int[] = [0, 0, 0, 0, 0]
VAR node_spoiled: int[] = [0, 0, 0, 0, 0]
VAR node_usable: int[] = [0, 0, 0, 0, 0]
VAR node_meal_points: int[] = [0, 0, 0, 0, 0]

VAR total_raw: int = 0
VAR total_spoiled: int = 0
VAR total_usable: int = 0
VAR total_meal_points: int = 0
VAR preserved_stores: int = 0

== main ==
Wilderness forage rotation begins.
Season on entry: {current_season}
Damp weather pressure: {damp_weather}
~ run_forage(0)
~ report_node_ledger(0)
~ cook_camp_meals()
-> DONE

== function run_forage(index: int) => void ==
{ if index >= LEN(forage_nodes):
    Foraging routes complete.
- else:
    ~ temp node: ForageNode = forage_nodes[index]
    ~ temp availability: int = availability_for(node.preferred_season)
    ~ temp gathered: int = (node.base_yield * availability) / 100
    ~ temp spoilage_rate: int = clamp_percent(node.spoilage_risk + damp_weather / 3)
    ~ temp spoiled: int = (gathered * spoilage_rate) / 100
    ~ temp usable: int = gathered - spoiled
    ~ temp preserved: int = usable / 3
    ~ temp fresh_cook: int = usable - preserved
    ~ temp meal_points: int = fresh_cook * node.meal_value + preserved * (node.meal_value + 1)

    ~ node_raw[index] = gathered
    ~ node_spoiled[index] = spoiled
    ~ node_usable[index] = usable
    ~ node_meal_points[index] = meal_points

    ~ total_raw = total_raw + gathered
    ~ total_spoiled = total_spoiled + spoiled
    ~ total_usable = total_usable + usable
    ~ total_meal_points = total_meal_points + meal_points
    ~ preserved_stores = preserved_stores + preserved

    Node {index + 1}: {node.name}
    Preferred season: {node.preferred_season}
    Seasonal availability: {availability}%
    Gathered bundles: {gathered}
    Spoilage rate: {spoilage_rate}%
    Spoiled bundles: {spoiled}
    Usable bundles: {usable}
    Preserved at camp: {preserved}
    Meal points from node: {meal_points}

    ~ run_forage(index + 1)
}

== function availability_for(preferred: string) => int ==
{ if preferred == current_season:
    ~ return 100
- else:
    { if preferred == "Summer" or preferred == "Winter":
        ~ return 68
    - else:
        ~ return 40
    }
}

== function clamp_percent(value: int) => int ==
{ if value < 0:
    ~ return 0
- else:
    { if value > 95:
        ~ return 95
    - else:
        ~ return value
    }
}

== function report_node_ledger(index: int) => void ==
{ if index == 0:
    Node forage ledger:
}
{ if index >= LEN(forage_nodes):
    ~ return
- else:
    ~ temp node: ForageNode = forage_nodes[index]
    {node.name}: raw {node_raw[index]}, spoiled {node_spoiled[index]}, usable {node_usable[index]}, meal points {node_meal_points[index]}
    ~ report_node_ledger(index + 1)
}

== function cook_camp_meals() => void ==
~ temp camp_meals: int = total_meal_points / 7
~ temp leftovers: int = total_meal_points - camp_meals * 7
~ temp spoilage_percent: int = 0
{ if total_raw > 0:
    ~ spoilage_percent = total_spoiled * 100 / total_raw
}
Camp kitchen summary:
Total raw bundles: {total_raw}
Total spoiled bundles: {total_spoiled}
Total usable bundles: {total_usable}
Preserved stores: {preserved_stores}
Spoilage share: {spoilage_percent}%
Camp meals prepared: {camp_meals}
Meal points left over: {leftovers}
{ if camp_meals >= 16 and spoilage_percent <= 22:
    Outcome: camp fed for the week with reserves.
- else:
    { if camp_meals >= 12:
        Outcome: camp fed, but rationing starts before week end.
    - else:
        Outcome: foraging shortfall; hunters must supplement stores.
    }
}
