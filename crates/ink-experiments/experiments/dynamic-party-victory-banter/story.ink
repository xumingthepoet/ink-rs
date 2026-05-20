=== module game ===
STRUCT Ally {
    name: string
    victory_line: string
}

STRUCT BattleResult {
    encounter: string
    allies: Ally[]
}

VAR result: BattleResult = %BattleResult{
    encounter: "warehouse ambush",
    allies: [
        %Ally{ name: "Mira", victory_line: "No one touches our medic twice." },
        %Ally{ name: "Tao", victory_line: "Next time we bring fewer sparks." },
        %Ally{ name: "Nia", victory_line: "Mark the east door as unsafe." }
    ]
}

== main ==
Victory: {result.encounter}
-> party_banter(result.allies, 0)

== party_banter(members: Ally[], index: int) ==
{ if index >= LEN(members):
    -> END
- else:
    {members[index].name}: {members[index].victory_line}
    -> party_banter(members, index + 1)
}
