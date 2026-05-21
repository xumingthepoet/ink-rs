=== module game ===

ENUM TurnPhase { NeedFirst NeedSecond Won }

STRUCT Card {
    id: int
    matched: bool
}

CONST rows: int[] = [0, 1]
CONST cols: int[] = [0, 1, 2]
CONST initial_cards: Card[] = [
    %Card{ id: 1, matched: false },
    %Card{ id: 2, matched: false },
    %Card{ id: 3, matched: false },
    %Card{ id: 1, matched: false },
    %Card{ id: 2, matched: false },
    %Card{ id: 3, matched: false }
]
CONST card_labels: Dict<int, string> = %{1: "A", 2: "B", 3: "C"}
CONST pair_goals: Dict<int, int> = %{1: 2, 2: 2, 3: 2}

VAR cards: Card[] = initial_cards
VAR first_choice: int = -1
VAR moves: int = 0
VAR streak: int = 0
VAR mistakes: int = 0
VAR phase: TurnPhase = TurnPhase.NeedFirst
VAR last_message: string = "Pick the first card."

== main ==
Text memory match 3x2.
-> board_prompt

== board_prompt ==
Move {moves}
Streak: {to_str(streak)}
Mistakes: {to_str(mistakes)}
{ for row in rows:
{row_cells(row)}
}
Matched: {to_str(matched_count())}/{to_str(LEN(cards))}
Pairs: {pair_progress()}
{last_message}
{ if phase == TurnPhase.Won:
    Result: all pairs matched.
    * Restart
        -> restart
- else:
    * Slot 1
        -> choose_slot(0)
    * Slot 2
        -> choose_slot(1)
    * Slot 3
        -> choose_slot(2)
    * Slot 4
        -> choose_slot(3)
    * Slot 5
        -> choose_slot(4)
    * Slot 6
        -> choose_slot(5)
    * Restart
        -> restart
}

== restart ==
~ cards = initial_cards
~ first_choice = -1
~ moves = 0
~ streak = 0
~ mistakes = 0
~ phase = TurnPhase.NeedFirst
~ last_message = "Restarted."
-> board_prompt

== choose_slot(index: int) ==
{ if cards[index].matched:
    ~ last_message = "Slot " + slot_label(index) + " is already matched."
- else:
    { switch phase:
    - TurnPhase.NeedFirst:
        ~ first_choice = index
        ~ phase = TurnPhase.NeedSecond
        ~ last_message = "Slot " + slot_label(index) + " opened as " + card_label(cards[index].id) + "; pick a second card."
    - TurnPhase.NeedSecond:
        ~ moves += 1
        { if index == first_choice:
            ~ last_message = "Slot " + slot_label(index) + " is already open."
        - else:
            { if cards[index].id == cards[first_choice].id:
                ~ cards[index].matched = true
                ~ cards[first_choice].matched = true
                ~ streak += 1
                ~ last_message = "Slots " + slot_label(first_choice) + " and " + slot_label(index) + " match " + card_label(cards[index].id) + "."
            - else:
                ~ mistakes += 1
                ~ streak = 0
                ~ last_message = "Slots " + slot_label(first_choice) + " and " + slot_label(index) + " reveal " + card_label(cards[first_choice].id) + "/" + card_label(cards[index].id) + "."
            }
            ~ first_choice = -1
            ~ phase = TurnPhase.NeedFirst
            ~ check_win()
        }
    - else:
        ~ last_message = "Game is complete."
    }
}
-> board_prompt

== function check_win() => void ==
{ if matched_count() == LEN(cards):
    ~ phase = TurnPhase.Won
    ~ last_message = last_message + " Memory cleared."
}

== function row_cells(row: int) => string ==
~ temp text: string = ""
{ for col in cols:
    ~ temp index: int = row * LEN(cols) + col
    { if col == 0:
        ~ text = display_card(index)
    - else:
        ~ text = text + " " + display_card(index)
    }
}
~ return text

== function display_card(index: int) => string ==
{ if cards[index].matched || index == first_choice:
    ~ return card_label(cards[index].id)
- else:
    ~ return slot_label(index)
}

== function pair_progress() => string ==
~ temp text: string = ""
{ for card_id, goal in pair_goals:
    { if text == "":
        ~ text = card_label(card_id) + " " + to_str(matched_count_for(card_id)) + "/" + to_str(goal)
    - else:
        ~ text = text + ", " + card_label(card_id) + " " + to_str(matched_count_for(card_id)) + "/" + to_str(goal)
    }
}
~ return text

== function matched_count_for(card_id: int) => int ==
~ temp count: int = 0
{ for card in cards:
    { if card.id == card_id && card.matched:
        ~ count += 1
    }
}
~ return count

== function matched_count() => int ==
~ temp count: int = 0
{ for card in cards:
    { if card.matched:
        ~ count += 1
    }
}
~ return count

== function card_label(card_id: int) => string ==
~ return card_labels[card_id]

== function slot_label(index: int) => string ==
{ switch index:
- 0:
    ~ return "1"
- 1:
    ~ return "2"
- 2:
    ~ return "3"
- 3:
    ~ return "4"
- 4:
    ~ return "5"
- else:
    ~ return "6"
}
