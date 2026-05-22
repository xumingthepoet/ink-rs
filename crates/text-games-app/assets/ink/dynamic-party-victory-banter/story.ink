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
-> party_banter(result.allies)

== party_banter(members: Ally[]) ==
{ for member in members:
    {member.name}: {member.victory_line}
}
