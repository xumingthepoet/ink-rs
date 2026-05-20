=== module game ===
STRUCT Gear {
    slot: string
    name: string
    attack: int
    defense: int
    speed: int
}

VAR equipped_slots: Gear[] = [
    %Gear{ slot: "Head", name: "Iron Helm", attack: 0, defense: 2, speed: 0 },
    %Gear{ slot: "Body", name: "Leather Tunic", attack: 0, defense: 3, speed: 0 },
    %Gear{ slot: "Weapon", name: "Wooden Staff", attack: 4, defense: 0, speed: 1 },
    %Gear{ slot: "Boots", name: "Traveler Boots", attack: 0, defense: 1, speed: 2 },
    %Gear{ slot: "Ring", name: "Clear Opal", attack: 1, defense: 1, speed: 0 }
]

== main ==
Equipment loadout totals:
~ temp attack: int = total_attack(0, 0)
~ temp defense: int = total_defense(0, 0)
~ temp speed: int = total_speed(0, 0)
~ temp head: Gear = equipped_slots[0]
~ temp body: Gear = equipped_slots[1]
~ temp weapon: Gear = equipped_slots[2]
~ temp boots: Gear = equipped_slots[3]
~ temp ring: Gear = equipped_slots[4]
Head: {head.name} [A{head.attack} D{head.defense} S{head.speed}]
Body: {body.name} [A{body.attack} D{body.defense} S{body.speed}]
Weapon: {weapon.name} [A{weapon.attack} D{weapon.defense} S{weapon.speed}]
Boots: {boots.name} [A{boots.attack} D{boots.defense} S{boots.speed}]
Ring: {ring.name} [A{ring.attack} D{ring.defense} S{ring.speed}]
Totals: ATK {attack}, DEF {defense}, SPD {speed}.
~ equipped_slots[1] = %Gear{ slot: "Body", name: "Chainmail", attack: 1, defense: 7, speed: -1 }
Body now uses {equipped_slots[1].name}.
~ equipped_slots[3] = %Gear{ slot: "Boots", name: "Racer Greaves", attack: 0, defense: 0, speed: 5 }
Boots now uses {equipped_slots[3].name}.
~ head = equipped_slots[0]
~ body = equipped_slots[1]
~ weapon = equipped_slots[2]
~ boots = equipped_slots[3]
~ ring = equipped_slots[4]
~ attack = total_attack(0, 0)
~ defense = total_defense(0, 0)
~ speed = total_speed(0, 0)
Head: {head.name} [A{head.attack} D{head.defense} S{head.speed}]
Body: {body.name} [A{body.attack} D{body.defense} S{body.speed}]
Weapon: {weapon.name} [A{weapon.attack} D{weapon.defense} S{weapon.speed}]
Boots: {boots.name} [A{boots.attack} D{boots.defense} S{boots.speed}]
Ring: {ring.name} [A{ring.attack} D{ring.defense} S{ring.speed}]
Totals: ATK {attack}, DEF {defense}, SPD {speed}.
-> DONE

== function total_attack(index: int, cumulative: int) => int ==
{ if index >= LEN(equipped_slots):
    ~ return cumulative
- else:
    ~ return total_attack(index + 1, cumulative + equipped_slots[index].attack)
}

== function total_defense(index: int, cumulative: int) => int ==
{ if index >= LEN(equipped_slots):
    ~ return cumulative
- else:
    ~ return total_defense(index + 1, cumulative + equipped_slots[index].defense)
}

== function total_speed(index: int, cumulative: int) => int ==
{ if index >= LEN(equipped_slots):
    ~ return cumulative
- else:
    ~ return total_speed(index + 1, cumulative + equipped_slots[index].speed)
}
