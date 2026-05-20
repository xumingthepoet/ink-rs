=== module game ===

STRUCT Card {
    name: string
    card_type: string
    cost: int
    power: int
}

VAR draft_pool: Card[] = [
    %Card{ name: "Ember Duelist", card_type: "Flame", cost: 2, power: 5 },
    %Card{ name: "Coral Channeler", card_type: "Tide", cost: 3, power: 6 },
    %Card{ name: "Wildroot Guardian", card_type: "Grove", cost: 2, power: 4 },
    %Card{ name: "Arc Relay", card_type: "Spark", cost: 1, power: 3 },
    %Card{ name: "Cinder Cyclone", card_type: "Flame", cost: 4, power: 8 },
    %Card{ name: "Reef Tactician", card_type: "Tide", cost: 2, power: 5 },
    %Card{ name: "Canopy Warden", card_type: "Grove", cost: 3, power: 7 },
    %Card{ name: "Storm Diagram", card_type: "Spark", cost: 3, power: 6 },
    %Card{ name: "Lava Cartographer", card_type: "Flame", cost: 3, power: 7 },
    %Card{ name: "Mist Archivist", card_type: "Tide", cost: 1, power: 2 }
]

VAR pick_order: int[] = [2, 0, 6, 1, 8, 3, 5, 7]
VAR draw_order: int[] = [0, 3, 1, 4, 2, 6, 5, 7]

VAR deck_cards: Card[] = []

VAR flame_count: int = 0
VAR tide_count: int = 0
VAR grove_count: int = 0
VAR spark_count: int = 0

VAR deck_power_total: int = 0
VAR type_synergy_total: int = 0
VAR combo_total: int = 0
VAR tempo_total: int = 0
VAR last_draw_type: string = ""
VAR opponent_target: int = 92

== main ==
Trading-card deck synergy simulation begins.
~ build_deck(0)
~ type_synergy_total = compute_type_synergy()
~ run_draw_sequence(0)
~ print_deck_list(0)
~ print_match_result()
-> DONE

== function build_deck(slot: int) => void ==
{ if slot >= LEN(pick_order):
    Deck assembly complete.
- else:
    ~ temp pick_index: int = pick_order[slot]
    ~ temp card: Card = draft_pool[pick_index]
    ~ ARRAY_PUSH(deck_cards, card)
    ~ deck_power_total = deck_power_total + card.power
    ~ add_type_count(card.card_type)
    Pick {slot + 1}: {card.name} [{card.card_type}] cost {card.cost}, power {card.power}
    ~ build_deck(slot + 1)
}

== function add_type_count(card_type: string) => void ==
{ if card_type == "Flame":
    ~ flame_count = flame_count + 1
- else:
    { if card_type == "Tide":
        ~ tide_count = tide_count + 1
    - else:
        { if card_type == "Grove":
            ~ grove_count = grove_count + 1
        - else:
            { if card_type == "Spark":
                ~ spark_count = spark_count + 1
            }
        }
    }
}

== function compute_type_synergy() => int ==
~ temp same_type: int = pair_bonus(flame_count) + pair_bonus(tide_count) + pair_bonus(grove_count) + pair_bonus(spark_count)
~ temp cross_type: int = cross_bonus(flame_count, tide_count, 4) + cross_bonus(grove_count, spark_count, 3) + cross_bonus(flame_count, grove_count, 2) + cross_bonus(tide_count, spark_count, 2)
~ return same_type + cross_type

== function pair_bonus(count: int) => int ==
~ return (count * (count - 1) / 2) * 3

== function cross_bonus(a: int, b: int, weight: int) => int ==
~ temp smaller: int = min_int(a, b)
~ return smaller * weight

== function min_int(a: int, b: int) => int ==
{ if a < b:
    ~ return a
- else:
    ~ return b
}

== function run_draw_sequence(step: int) => void ==
{ if step >= LEN(draw_order):
    Draw sequence complete.
- else:
    ~ temp slot: int = draw_order[step]
    ~ temp card: Card = deck_cards[slot]
    ~ temp combo_gain: int = combo_gain_for(last_draw_type, card.card_type)
    ~ temp tempo: int = card.power - card.cost
    ~ combo_total = combo_total + combo_gain
    ~ tempo_total = tempo_total + tempo
    Draw {step + 1}: {card.name} [{card.card_type}]
    Combo gain: {combo_gain}
    Tempo swing: {tempo}
    ~ last_draw_type = card.card_type
    ~ run_draw_sequence(step + 1)
}

== function combo_gain_for(previous_type: string, current_type: string) => int ==
{ if previous_type == "":
    ~ return 0
- else:
    { if previous_type == current_type:
        ~ return 2
    - else:
        { if (previous_type == "Flame" and current_type == "Tide") or (previous_type == "Tide" and current_type == "Flame"):
            ~ return 5
        - else:
            { if (previous_type == "Grove" and current_type == "Spark") or (previous_type == "Spark" and current_type == "Grove"):
                ~ return 4
            - else:
                { if (previous_type == "Tide" and current_type == "Grove") or (previous_type == "Grove" and current_type == "Tide"):
                    ~ return 3
                - else:
                    ~ return 1
                }
            }
        }
    }
}

== function print_deck_list(index: int) => void ==
{ if index == 0:
    Final deck list:
}
{ if index >= LEN(deck_cards):
    ~ return
- else:
    ~ temp card: Card = deck_cards[index]
    Slot {index + 1}: {card.name} [{card.card_type}] cost {card.cost} power {card.power}
    ~ print_deck_list(index + 1)
}

== function print_match_result() => void ==
~ temp final_score: int = deck_power_total + type_synergy_total + combo_total + tempo_total
Match report:
Flame cards: {flame_count}
Tide cards: {tide_count}
Grove cards: {grove_count}
Spark cards: {spark_count}
Deck power total: {deck_power_total}
Type synergy total: {type_synergy_total}
Draw combo total: {combo_total}
Tempo total: {tempo_total}
Opponent target: {opponent_target}
Final match score: {final_score}
{ if final_score >= opponent_target + 10:
    Result: decisive win through synergy pressure.
- else:
    { if final_score >= opponent_target:
        Result: narrow win after stable draw execution.
    - else:
        Result: loss; deck needs better curve and combo density.
    }
}
