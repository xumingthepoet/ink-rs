=== module game ===
VAR loot_items: string[] = ["Potion", "Elixir", "Relic", "Detritus"]
VAR loot_weights: int[] = [40, 30, 20, 10]
VAR deterministic_rolls: int[] = [7, 42, 65, 99]

== main ==
Deterministic weighted loot rolls:
-> open_chests(0) ->
-> DONE

== open_chests(chest: int) ==
{ if chest >= LEN(deterministic_rolls):
    All chests opened.
    ->->
- else:
    ~ temp roll: int = deterministic_rolls[chest]
    ~ temp picked: string = select_loot(roll, 0, 0)
    Chest {chest + 1}: roll {roll} selects {picked}.
    -> open_chests(chest + 1)
}

== function select_loot(roll: int, index: int, threshold: int) => string ==
{ if index >= LEN(loot_items):
    ~ return "No Loot"
- else:
    ~ temp next_threshold: int = threshold + loot_weights[index]
    { if roll < next_threshold:
        ~ return loot_items[index]
    - else:
        ~ return select_loot(roll, index + 1, next_threshold)
    }
}
