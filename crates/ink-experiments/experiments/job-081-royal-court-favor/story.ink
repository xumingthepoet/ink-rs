=== module game ===

STRUCT Faction {
    name: string
    favor: int
    expectation: int
    sensitivity: int
}

STRUCT Gift {
    recipient: string
    item: string
    value: int
    polite: bool
}

VAR court_factions: Faction[] = [
    %Faction{name: "Crown", favor: 68, expectation: 70, sensitivity: 3},
    %Faction{name: "Heir Apparent", favor: 58, expectation: 65, sensitivity: 2},
    %Faction{name: "Merchant Guild", favor: 52, expectation: 55, sensitivity: 2},
    %Faction{name: "Temple Order", favor: 60, expectation: 62, sensitivity: 3}
]

VAR faction_lookup: Dict<string, int> = %{
    "Crown": 0,
    "Heir Apparent": 1,
    "Merchant Guild": 2,
    "Temple Order": 3
}

VAR court_gifts: Gift[] = [
    %Gift{
        recipient: "Crown",
        item: "a carved silver seal",
        value: 9,
        polite: true
    },
    %Gift{
        recipient: "Heir Apparent",
        item: "a cedar flute case",
        value: 4,
        polite: false
    },
    %Gift{
        recipient: "Merchant Guild",
        item: "a caravan map with hidden routes",
        value: 7,
        polite: true
    },
    %Gift{
        recipient: "Temple Order",
        item: "healing cloth in temple blue",
        value: 6,
        polite: false
    },
    %Gift{
        recipient: "Crown",
        item: "a hand-hammered jade coin",
        value: 5,
        polite: true
    }
]

VAR etiquette_mistakes: int = 0
VAR favor_shifts: int = 0
VAR turn_count: int = 0

== main ==
The royal audience opens at dawn.
~ announce_faction_starts(0)
~ handle_gifts(0)
~ summarize_favor()
~ announce_outcome()
-> DONE

== function announce_faction_starts(index: int) => void ==
{ if index >= LEN(court_factions):
    ~ return
- else:
    ~ temp faction: Faction = court_factions[index]
    Faction {faction.name} starts at favor {faction.favor}, expected {faction.expectation}.
    ~ announce_faction_starts(index + 1)
}

== function handle_gifts(index: int) => void ==
{ if index >= LEN(court_gifts):
    ~ return
- else:
    ~ temp gift: Gift = court_gifts[index]
    ~ deliver_gift(gift.recipient, gift.item, gift.value, gift.polite)
    ~ handle_gifts(index + 1)
}

== function deliver_gift(recipient: string, item: string, value: int, polite: bool) => void ==
~ turn_count = turn_count + 1
~ temp index: int = faction_lookup[recipient]
~ temp actor: Faction = court_factions[index]
~ temp base_shift: int = value + actor.sensitivity
~ temp final_shift: int = base_shift
Gift {turn_count}: {actor.name} receives {item}.
{ if polite:
    Protocol phrasing and posture are correct.
- else:
    A misplaced bow creates an etiquette mistake.
    ~ etiquette_mistakes = etiquette_mistakes + 1
    ~ final_shift = final_shift - 5
}
~ final_shift = final_shift - etiquette_mistakes
{ if final_shift > 10:
    ~ final_shift = 10
- else:
    { if final_shift < -8:
        ~ final_shift = -8
    }
}
~ temp old_favor: int = actor.favor
~ temp new_favor: int = old_favor + final_shift
{ if new_favor > 100:
    ~ new_favor = 100
- else:
    { if new_favor < 0:
        ~ new_favor = 0
    }
}
~ court_factions[index].favor = new_favor
~ favor_shifts = favor_shifts + final_shift
Gift impact: {old_favor} -> {new_favor} (delta {final_shift}).

== function summarize_favor() => void ==
~ temp total_favor: int = total_favor_sum(0)
~ temp avg_favor: int = total_favor / LEN(court_factions)
~ temp min_favor: int = minimum_favor(0, 1000)
~ temp max_favor: int = maximum_favor(0, 0)
~ print_faction_states(0)
Audience readout:
Total favor shift: {favor_shifts}
Etiquette mistakes: {etiquette_mistakes}
Average favor: {avg_favor}
Lowest faction favor: {min_favor}
Highest faction favor: {max_favor}

== function announce_outcome() => void ==
~ temp total_favor: int = total_favor_sum(0)
~ temp avg_favor: int = total_favor / LEN(court_factions)
~ temp min_favor: int = minimum_favor(0, 1000)
~ temp outcome: string = outcome_label(avg_favor, etiquette_mistakes, min_favor)
Royal verdict:
{outcome}

== function print_faction_states(index: int) => void ==
{ if index >= LEN(court_factions):
    ~ return
- else:
    ~ temp faction: Faction = court_factions[index]
    {faction.name} now stands at {faction.favor}. {favor_status(faction.favor, faction.expectation)}
    ~ print_faction_states(index + 1)
}

== function favor_status(score: int, expectation: int) => string ==
{ if score >= expectation + 20:
    ~ return "deeply favoring the audience"
- else:
    { if score >= expectation:
        ~ return "satisfied and willing to grant favor"
    - else:
        { if score >= expectation - 10:
            ~ return "polite but cautious"
        - else:
            ~ return "offended enough to reduce access"
        }
    }
}

== function total_favor_sum(index: int) => int ==
{ if index >= LEN(court_factions):
    ~ return 0
- else:
    ~ temp faction: Faction = court_factions[index]
    ~ return faction.favor + total_favor_sum(index + 1)
}

== function minimum_favor(index: int, current_min: int) => int ==
{ if index >= LEN(court_factions):
    ~ return current_min
- else:
    ~ temp faction: Faction = court_factions[index]
    { if faction.favor < current_min:
        ~ return minimum_favor(index + 1, faction.favor)
    - else:
        ~ return minimum_favor(index + 1, current_min)
    }
}

== function maximum_favor(index: int, current_max: int) => int ==
{ if index >= LEN(court_factions):
    ~ return current_max
- else:
    ~ temp faction: Faction = court_factions[index]
    { if faction.favor > current_max:
        ~ return maximum_favor(index + 1, faction.favor)
    - else:
        ~ return maximum_favor(index + 1, current_max)
    }
}

== function outcome_label(avg_favor: int, mistakes: int, lowest_favor: int) => string ==
{ if mistakes >= 3:
    ~ return "Audience outcome: critical protocol breach. Access is revoked for this quarter."
- else:
    { if avg_favor >= 75 and mistakes == 0:
        ~ return "Audience outcome: honored and fully trusted. Multiple favors are likely."
    - else:
        { if avg_favor >= 66 and mistakes <= 1:
            ~ return "Audience outcome: acceptable standing. Favors are granted with written pledges."
        - else:
            { if avg_favor >= 58 and lowest_favor >= 55:
                ~ return "Audience outcome: conditional. Gifts helped, but etiquette remains under watch."
            - else:
                ~ return "Audience outcome: fragile relationship; one or more factions remain hostile."
            }
        }
    }
}
