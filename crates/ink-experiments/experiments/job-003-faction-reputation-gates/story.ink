=== module game ===
STRUCT FactionGate {
    name: string
    min_guild: int
    min_bandits: int
    min_scholars: int
}

VAR reputation: Dict<string, int> = %{
    "Guild": 5,
    "Bandits": 1,
    "Scholars": 12
}

VAR gates: FactionGate[] = [
    %FactionGate{ name: "Harbor", min_guild: 3, min_bandits: 0, min_scholars: 10 },
    %FactionGate{ name: "Cathedral", min_guild: 4, min_bandits: 4, min_scholars: 5 },
    %FactionGate{ name: "Spires", min_guild: 10, min_bandits: 2, min_scholars: 20 }
]

== main ==
Faction gate audit:
-> evaluate_gates(0) ->
~ reputation["Guild"] = reputation["Guild"] + 3
~ reputation["Bandits"] = reputation["Bandits"] + 3
~ reputation["Scholars"] = reputation["Scholars"] - 4
After reputation changes:
-> evaluate_gates(0) ->
-> DONE

== evaluate_gates(index: int) ==
{ if index >= LEN(gates):
    ->->
- else:
    ~ temp gate: FactionGate = gates[index]
    { if gate_ok(gate):
        {gate.name}: OPEN
    - else:
        {gate.name}: LOCKED
    }
    -> evaluate_gates(index + 1)
}

== function gate_ok(gate: FactionGate) => bool ==
~ return reputation["Guild"] >= gate.min_guild and reputation["Bandits"] >= gate.min_bandits and reputation["Scholars"] >= gate.min_scholars
