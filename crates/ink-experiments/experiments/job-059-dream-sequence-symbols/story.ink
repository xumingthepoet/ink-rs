=== module game ===

VAR dream_symbols: string[] = [
    "moon",
    "key",
    "river",
    "moon",
    "fox",
    "ash",
    "star",
    "river",
    "key"
]

VAR dream_labels: string[] = [
    "first",
    "second",
    "third",
    "fourth",
    "fifth",
    "sixth",
    "seventh",
    "eighth",
    "ninth"
]

VAR symbol_tally: Dict<string, int> = %{
    "moon": 0,
    "key": 0,
    "river": 0,
    "fox": 0,
    "star": 0,
    "ash": 0
}

VAR waking_outcome: string = "undecided"

== main ==
Night watch is stitched from nine symbols.
~ print_symbol_tally("Initial reading")
~ collect_dreams(0)
~ analyze_combinations()
~ log_waking_consequence()
-> DONE

== function collect_dreams(index: int) => void ==
{ if index >= LEN(dream_symbols):
    ~ return
}

~ temp stage: string = dream_labels[index]
~ temp symbol: string = dream_symbols[index]
Dream {stage} vision appears: {symbol}.
~ add_symbol(symbol)

~ collect_dreams(index + 1)

== function add_symbol(symbol: string) => void ==
~ temp current: int = symbol_tally[symbol]
~ symbol_tally[symbol] = current + 1

== function print_symbol_tally(label: string) => void ==
Symbol tally snapshot - {label}:
Moon: {symbol_tally["moon"]}
Key: {symbol_tally["key"]}
River: {symbol_tally["river"]}
Fox: {symbol_tally["fox"]}
Star: {symbol_tally["star"]}
Ash: {symbol_tally["ash"]}

== function analyze_combinations() => void ==
~ temp moon: int = symbol_tally["moon"]
~ temp key: int = symbol_tally["key"]
~ temp river: int = symbol_tally["river"]
~ temp fox: int = symbol_tally["fox"]
~ temp star: int = symbol_tally["star"]
~ temp ash: int = symbol_tally["ash"]

The symbols are reviewed in pattern order.
~ print_symbol_tally("After collection")

{ if moon >= 2 and key >= 2 and fox >= 1:
    ~ waking_outcome = "moon_gate"
    The moon-key-fox triad glows. A hidden gate reads your name.
- else:
    { if moon >= 2 and star >= 1 and river >= 2:
        ~ waking_outcome = "river_star"
        The moon thins; moon and star form a tide map and river lines sharpen.
    - else:
        { if key >= 2 and ash >= 1 and river >= 1:
            ~ waking_outcome = "forge_warning"
            Keys are heavy and ash clings. The dream warns of brittle luck.
        - else:
            ~ waking_outcome = "balanced_drift"
            Symbols stay mixed. You keep a balanced reading and no single omen dominates.
        }
    }
}

{ if moon >= 1 and key >= 1 and fox >= 1:
    This is the fox-key link, so hidden knowledge is likely reliable.
- else:
    The fox did not lock with key, so hidden knowledge remains uncertain.
}

{ if star >= 1 and river >= 1:
    Tide and star align, suggesting a route is open before dawn.
- else:
    No stable tide-star line appears.
}

{ if ash >= 2:
    Repeated ash suggests lingering bad intent; expect a cost.
- else:
    { if ash == 1:
        A trace of ash warns of small sacrifice.
    - else:
        No visible ruin is marked in the symbols.
    }
}

== function log_waking_consequence() => void ==
You wake before the bell.
{ if waking_outcome == "moon_gate":
    Consequence selected:
    The hidden gate opens under first light.
    You walk a secret corridor to the archive vault and recover the lost charter.
- else:
    { if waking_outcome == "river_star":
        Consequence selected:
        The river map reveals a northern route.
        You rise, cross by the old sluice, and avoid the city curfew.
    - else:
        { if waking_outcome == "forge_warning":
            Consequence selected:
            A forge-warning sends you to reinforce defenses at the temple gate.
            You spend the morning repairing brittle seams before acting on anything else.
        - else:
            Consequence selected:
            No clear omen controls the day.
            You wake with a neutral plan: monitor the city and wait for proof.
        }
    }
}

