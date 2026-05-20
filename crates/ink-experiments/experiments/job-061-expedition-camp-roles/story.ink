=== module game ===

VAR explorers: string[] = ["Nora", "Ike", "Vale", "Rook", "Sera"]

VAR camp_roles: Dict<string, string> = %{
    "Nora": "",
    "Ike": "",
    "Vale": "",
    "Rook": "",
    "Sera": ""
}

VAR fatigue: Dict<string, int> = %{
    "Nora": 3,
    "Ike": 4,
    "Vale": 2,
    "Rook": 5,
    "Sera": 3
}

VAR morale: Dict<string, int> = %{
    "Nora": 8,
    "Ike": 7,
    "Vale": 9,
    "Rook": 6,
    "Sera": 8
}

VAR supplies: Dict<string, int> = %{
    "food": 48,
    "water": 39,
    "firewood": 16,
    "medicine": 6
}

VAR night_names: string[] = ["First watch", "Second watch", "Third watch"]
VAR night_hazards: string[] = ["storm", "wolf_pack", "flood"]

== main ==
Camp is set at the edge of marsh and cedar.
~ assign_roles()
~ print_roster()
~ run_nights(0)
~ evaluate_camp_morale()
-> DONE

== function assign_roles() => void ==
~ camp_roles["Nora"] = "Scout"
~ camp_roles["Ike"] = "Engineer"
~ camp_roles["Vale"] = "Sentinel"
~ camp_roles["Rook"] = "Medic"
~ camp_roles["Sera"] = "Quartermaster"

Roles are assigned:
Nora: Scout
Ike: Engineer
Vale: Sentinel
Rook: Medic
Sera: Quartermaster

== function print_roster() => void ==
~ print_status("Opening")

== function run_nights(index: int) => void ==
{ if index >= LEN(night_names):
    ~ return
}

~ temp label: string = night_names[index]
~ temp hazard: string = night_hazards[index]
Camp management - {label} with hazard {hazard}.
~ consume_supplies(hazard)
~ apply_night_fatigue(0)
~ apply_hazard(hazard, 0)
~ apply_night_legacy(hazard)
~ print_status(label)
~ run_nights(index + 1)

== function consume_supplies(hazard: string) => void ==
~ temp food: int = 5
~ temp water: int = 4
~ temp firewood: int = 1
~ temp medicine: int = 0

{ if hazard == "storm":
    ~ food = 6
    ~ water = 6
    ~ firewood = 3
- else:
    { if hazard == "wolf_pack":
        ~ food = 7
        ~ water = 5
    - else:
        ~ food = 6
        ~ water = 5
        ~ firewood = 2
    }
}

~ supplies["food"] = supplies["food"] - food
~ supplies["water"] = supplies["water"] - water
~ supplies["firewood"] = supplies["firewood"] - firewood
~ supplies["medicine"] = supplies["medicine"] - medicine

Supplies consumed:
Food: {food}
Water: {water}
Firewood: {firewood}
Medicine: {medicine}

== function apply_night_fatigue(member_index: int) => void ==
{ if member_index >= LEN(explorers):
    ~ return
}
~ temp explorer: string = explorers[member_index]
~ fatigue[explorer] = fatigue[explorer] + 1
~ apply_night_fatigue(member_index + 1)

== function apply_hazard(hazard: string, member_index: int) => void ==
{ if member_index >= LEN(explorers):
    ~ return
}

~ temp explorer: string = explorers[member_index]
~ temp role: string = camp_roles[explorer]
~ temp current_fatigue: int = fatigue[explorer]
~ temp current_morale: int = morale[explorer]
~ temp next_fatigue: int = current_fatigue
~ temp next_morale: int = current_morale

{ if hazard == "storm":
    { if role == "Scout":
        ~ next_fatigue = current_fatigue + 1
        ~ next_morale = current_morale + 1
    - else:
        { if role == "Engineer":
            ~ next_fatigue = current_fatigue + 2
            ~ next_morale = current_morale
        - else:
            { if role == "Medic":
                ~ next_fatigue = current_fatigue + 1
                ~ next_morale = current_morale + 0
            - else:
                ~ next_fatigue = current_fatigue + 2
                ~ next_morale = current_morale - 1
            }
        }
    }
- else:
    { if hazard == "wolf_pack":
        { if role == "Scout":
            ~ next_fatigue = current_fatigue + 1
            ~ next_morale = current_morale - 1
        - else:
            { if role == "Sentinel":
                ~ next_fatigue = current_fatigue + 1
                ~ next_morale = current_morale + 1
            - else:
                { if role == "Medic":
                    ~ next_fatigue = current_fatigue + 2
                    ~ next_morale = current_morale - 1
                - else:
                    ~ next_fatigue = current_fatigue + 1
                    ~ next_morale = current_morale + 0
                }
            }
        }
    - else:
        { if hazard == "flood":
            { if role == "Engineer":
                ~ next_fatigue = current_fatigue + 1
                ~ next_morale = current_morale + 1
            - else:
                { if role == "Quartermaster":
                    ~ next_fatigue = current_fatigue + 2
                    ~ next_morale = current_morale - 1
                - else:
                    ~ next_fatigue = current_fatigue + 2
                    ~ next_morale = current_morale
                }
            }
        - else:
            ~ return
        }
    }
}

~ fatigue[explorer] = clamp_fatigue(next_fatigue)
~ morale[explorer] = clamp_morale(next_morale)

{explorer} ({role}) shifts to fatigue {fatigue[explorer]} and morale {morale[explorer]}.
~ apply_hazard(hazard, member_index + 1)

== function apply_night_legacy(hazard: string) => void ==
{ if hazard == "storm":
    Medicine usage after storm watch:
    ~ supplies["medicine"] = supplies["medicine"] - 1
- else:
    { if hazard == "wolf_pack":
        Medicine usage after wolf watch:
        ~ supplies["medicine"] = supplies["medicine"] - 0
    - else:
        Medicine usage after flood watch:
        ~ supplies["medicine"] = supplies["medicine"] - 1
    }
}

== function print_status(label: string) => void ==
-- {label} status --
~ list_member_status(0)
~ print_supplies()

== function list_member_status(index: int) => void ==
{ if index >= LEN(explorers):
    ~ return
}
~ temp explorer: string = explorers[index]
~ temp role: string = camp_roles[explorer]
~ temp explorer_fatigue: int = fatigue[explorer]
~ temp explorer_morale: int = morale[explorer]
{ explorer} ({role}): fatigue {explorer_fatigue}, morale {explorer_morale}
~ list_member_status(index + 1)

== function print_supplies() => void ==
Camp stores:
Food: {supplies["food"]}
Water: {supplies["water"]}
Firewood: {supplies["firewood"]}
Medicine: {supplies["medicine"]}

== function clamp_fatigue(value: int) => int ==
{ if value < 0:
    ~ return 0
- else:
    { if value > 12:
        ~ return 12
    - else:
        ~ return value
    }
}

== function clamp_morale(value: int) => int ==
{ if value < 0:
    ~ return 0
- else:
    { if value > 10:
        ~ return 10
    - else:
        ~ return value
    }
}

== function evaluate_camp_morale() => void ==
~ temp total_fatigue: int = total_fatigue(0, 0)
~ temp exhausted_members: int = exhausted_explorers(0, 0)
~ temp morale_score: int = total_morale(0, 0)

Final morale check:
Exhausted explorers: {exhausted_members}
Total fatigue load: {total_fatigue}
Total morale level: {morale_score}
~ temp average_fatigue: int = total_fatigue / LEN(explorers)
~ temp average_morale: int = morale_score / LEN(explorers)

The averages are {average_fatigue} fatigue and {average_morale} morale.

{ if average_fatigue >= 9:
    A forced rest is required.
    You move camp to higher ground in the morning.
- else:
    { if average_morale <= 3:
        Tension is critical.
        Night schedules are cut to a crawl for one day.
    - else:
        The camp holds.
        You continue with day plans and one more watch rotation.
    }
}

{ if supplies["food"] < 30 or supplies["water"] < 20:
    Supply stocks are tight; rationing begins immediately.
- else:
    { if supplies["firewood"] < 7:
        Fuel is low and morale dips at dusk.
    - else:
        Supplies remain stable for the next march.
    }
}

== function total_fatigue(index: int, acc: int) => int ==
{ if index >= LEN(explorers):
    ~ return acc
}
~ temp explorer: string = explorers[index]
~ temp current: int = fatigue[explorer]
~ return total_fatigue(index + 1, acc + current)

== function exhausted_explorers(index: int, acc: int) => int ==
{ if index >= LEN(explorers):
    ~ return acc
}
~ temp explorer: string = explorers[index]
~ temp current: int = fatigue[explorer]
{ if current >= 10:
    ~ return exhausted_explorers(index + 1, acc + 1)
- else:
    ~ return exhausted_explorers(index + 1, acc)
}

== function total_morale(index: int, acc: int) => int ==
{ if index >= LEN(explorers):
    ~ return acc
}
~ temp explorer: string = explorers[index]
~ temp current: int = morale[explorer]
~ return total_morale(index + 1, acc + current)
