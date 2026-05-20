=== module game ===

STRUCT Rite {
    name: string
    element: string
    vigor: int
    intent: int
    chant: int
}

VAR element_spirit: Dict<string, int> = %{
    "Water": 58,
    "Fire": 46,
    "Earth": 52,
    "Air": 41
}

VAR rites: Rite[] = [
    %Rite{ name: "Moonwell Tide Blessing", element: "Water", vigor: 12, intent: 1, chant: 4 },
    %Rite{ name: "Ashen Ward Circle", element: "Fire", vigor: 9, intent: 1, chant: 5 },
    %Rite{ name: "Soil Anchor Binding", element: "Earth", vigor: 13, intent: -1, chant: 3 },
    %Rite{ name: "Highwind Invocation", element: "Air", vigor: 7, intent: 1, chant: 6 },
    %Rite{ name: "Deep Root Supplication", element: "Earth", vigor: 14, intent: 1, chant: 4 },
    %Rite{ name: "Smoke Gate Offering", element: "Fire", vigor: 10, intent: -1, chant: 2 },
    %Rite{ name: "Riverglass Chime", element: "Water", vigor: 8, intent: 1, chant: 7 },
    %Rite{ name: "Cloud Hollow Lament", element: "Air", vigor: 11, intent: -1, chant: 1 }
]

VAR grove_imbalance: int = 0
VAR restoration_score: int = 0
VAR corruption_score: int = 0
VAR restored_events: int = 0
VAR corrupted_events: int = 0
VAR total_events: int = 0

== main ==
Druid grove balance cycle begins.
~ grove_imbalance = current_imbalance()
Opening grove state:
~ report_grove()
~ process_rites(0)
~ settle_grove()
-> DONE

== function process_rites(index: int) => void ==
{ if index >= LEN(rites):
    All rites complete.
- else:
    ~ execute_rite(index)
    ~ process_rites(index + 1)
}

== function execute_rite(index: int) => void ==
~ temp rite: Rite = rites[index]
Rite {index + 1}: {rite.name}
{ if rite.intent > 0:
    Intent: restorative.
- else:
    Intent: corrupting pressure.
}

~ temp before_imbalance: int = current_imbalance()
~ temp before_water: int = element_spirit["Water"]
~ temp before_fire: int = element_spirit["Fire"]
~ temp before_earth: int = element_spirit["Earth"]
~ temp before_air: int = element_spirit["Air"]

~ temp raw_delta: int = rite.vigor + rite.chant + (rite.intent * 3)
~ temp clamped_delta: int = clamp_delta(raw_delta)

~ temp current_level: int = element_spirit[rite.element]
~ temp updated_level: int = clamp_spirit(current_level + clamped_delta)
~ element_spirit[rite.element] = updated_level

~ total_events = total_events + 1
~ apply_neighbor_pressure(rite.element, rite.intent)

~ temp after_imbalance: int = current_imbalance()
~ grove_imbalance = after_imbalance
~ temp balance_change: int = before_imbalance - after_imbalance

~ temp element_delta: int = updated_level - current_level
Element {rite.element} moved by {element_delta} to {updated_level}.

{ if balance_change > 0:
    ~ restoration_score = restoration_score + balance_change
    ~ restored_events = restored_events + 1
    Balance improved by {balance_change}.
- else:
    { if balance_change < 0:
        ~ corruption_score = corruption_score + (-balance_change)
        ~ corrupted_events = corrupted_events + 1
        Balance worsened by {(-balance_change)}.
    - else:
        Balance held steady.
    }
}

~ report_grove()

== function apply_neighbor_pressure(target: string, intent: int) => void ==
{ if target == "Water":
    { if intent > 0:
        ~ element_spirit["Fire"] = clamp_spirit(element_spirit["Fire"] - 1)
        ~ element_spirit["Air"] = clamp_spirit(element_spirit["Air"] - 1)
    - else:
        ~ element_spirit["Fire"] = clamp_spirit(element_spirit["Fire"] + 2)
        ~ element_spirit["Air"] = clamp_spirit(element_spirit["Air"] + 2)
    }
- else:
    { if target == "Fire":
        { if intent > 0:
            ~ element_spirit["Earth"] = clamp_spirit(element_spirit["Earth"] - 1)
            ~ element_spirit["Air"] = clamp_spirit(element_spirit["Air"] - 1)
        - else:
            ~ element_spirit["Earth"] = clamp_spirit(element_spirit["Earth"] + 2)
            ~ element_spirit["Air"] = clamp_spirit(element_spirit["Air"] + 2)
        }
    - else:
        { if target == "Earth":
            { if intent > 0:
                ~ element_spirit["Water"] = clamp_spirit(element_spirit["Water"] - 1)
                ~ element_spirit["Fire"] = clamp_spirit(element_spirit["Fire"] - 1)
            - else:
                ~ element_spirit["Water"] = clamp_spirit(element_spirit["Water"] + 2)
                ~ element_spirit["Fire"] = clamp_spirit(element_spirit["Fire"] + 2)
            }
        - else:
            { if target == "Air":
                { if intent > 0:
                    ~ element_spirit["Water"] = clamp_spirit(element_spirit["Water"] - 1)
                    ~ element_spirit["Earth"] = clamp_spirit(element_spirit["Earth"] - 1)
                - else:
                    ~ element_spirit["Water"] = clamp_spirit(element_spirit["Water"] + 2)
                    ~ element_spirit["Earth"] = clamp_spirit(element_spirit["Earth"] + 2)
                }
            }
        }
    }
}

== function settle_grove() => void ==
Final grove assessment:
~ report_grove()
Restoration pressure: {restoration_score}
Corruption pressure: {corruption_score}
Restorative events: {restored_events}
Corruptive events: {corrupted_events}
Imbalance index: {grove_imbalance}

{ if grove_imbalance <= 20 && restoration_score > corruption_score:
    Outcome: grove restored.
- else:
    { if corruption_score > restoration_score + 8 || grove_imbalance >= 55:
        Outcome: grove corruption advances.
    - else:
        Outcome: grove remains fragile and unstable.
    }
}

== function report_grove() => void ==
~ temp water: int = element_spirit["Water"]
~ temp fire: int = element_spirit["Fire"]
~ temp earth: int = element_spirit["Earth"]
~ temp air: int = element_spirit["Air"]
~ temp current: int = current_imbalance()
Water: {water}
Fire: {fire}
Earth: {earth}
Air: {air}
Imbalance: {current}

== function current_imbalance() => int ==
~ temp water: int = element_spirit["Water"]
~ temp fire: int = element_spirit["Fire"]
~ temp earth: int = element_spirit["Earth"]
~ temp air: int = element_spirit["Air"]
~ temp total: int = imbalance_component(water) + imbalance_component(fire) + imbalance_component(earth) + imbalance_component(air)
~ return total

== function imbalance_component(value: int) => int ==
{ if value > 50:
    ~ return value - 50
- else:
    ~ return 50 - value
}

== function clamp_spirit(value: int) => int ==
{ if value < 0:
    ~ return 0
- else:
    { if value > 100:
        ~ return 100
    - else:
        ~ return value
    }
}

== function clamp_delta(value: int) => int ==
{ if value < -12:
    ~ return -12
- else:
    { if value > 12:
        ~ return 12
    - else:
        ~ return value
    }
}
