=== module game ===
STRUCT Combatant {
    name: string
    zone: int
}

VAR party: Combatant[] = [
    %Combatant{name: "Warrior", zone: 0},
    %Combatant{name: "Rogue", zone: 2},
    %Combatant{name: "Mage", zone: 3}
]

VAR enemies: Combatant[] = [
    %Combatant{name: "Goblin Skirmisher", zone: 0},
    %Combatant{name: "Goblin Archer", zone: 1},
    %Combatant{name: "Ogre Bruiser", zone: 3}
]

== main ==
Tactical position zone prototype.
~ print_snapshot("Initial deployment")
~ move_party_unit(1, 0)
~ print_snapshot("After Rogue rushes forward")
~ move_enemy_unit(0, 2)
~ print_snapshot("After Skirmisher repositions")
~ move_enemy_unit(2, 2)
~ move_party_unit(0, 1)
~ print_snapshot("After Ogre pressure and Warrior advance")
-> DONE

== function print_snapshot(label: string) => void ==
{label}
~ print_party_positions(0)
~ print_enemy_positions(0)
~ print_engagement_text(0)

== function print_party_positions(index: int) => void ==
{ if index < LEN(party):
    {party[index].name}: {zone_name(party[index].zone)}
    ~ print_party_positions(index + 1)
}

== function print_enemy_positions(index: int) => void ==
{ if index < LEN(enemies):
    {enemies[index].name}: {zone_name(enemies[index].zone)}
    ~ print_enemy_positions(index + 1)
}

== function print_engagement_text(p_index: int) => void ==
{ if p_index < LEN(party):
    ~ print_engagements_for_party(p_index, 0)
    ~ print_engagement_text(p_index + 1)
}

== function print_engagements_for_party(p_index: int, e_index: int) => void ==
{ if e_index < LEN(enemies):
    {party[p_index].name} to {enemies[e_index].name}: {engagement_text(party[p_index].zone, enemies[e_index].zone)}
    ~ print_engagements_for_party(p_index, e_index + 1)
}

== function engagement_text(unit_zone: int, enemy_zone: int) => string ==
~ temp distance: int = zone_distance(unit_zone, enemy_zone)
{ if distance == 0:
    ~ return "Melee"
- else:
    { if distance == 1:
        ~ return "Short range"
    - else:
        { if distance == 2:
            ~ return "Mid range"
        - else:
            ~ return "Long range"
        }
    }
}

== function zone_distance(a: int, b: int) => int ==
{ if a < b:
    ~ return b - a
- else:
    ~ return a - b
}

== function zone_name(index: int) => string ==
{ if index == 0:
    ~ return "Front"
- else:
    { if index == 1:
        ~ return "Mid"
    - else:
        { if index == 2:
            ~ return "Flank"
        - else:
            ~ return "Rear"
        }
    }
}

== function move_party_unit(index: int, new_zone: int) => void ==
~ temp old_zone: int = party[index].zone
{ if old_zone == new_zone:
    {party[index].name} holds at {zone_name(old_zone)}.
- else:
    {party[index].name} moves from {zone_name(old_zone)} to {zone_name(new_zone)}.
    ~ party[index].zone = new_zone
}

== function move_enemy_unit(index: int, new_zone: int) => void ==
~ temp old_zone: int = enemies[index].zone
{ if old_zone == new_zone:
    {enemies[index].name} holds at {zone_name(old_zone)}.
- else:
    {enemies[index].name} slips from {zone_name(old_zone)} to {zone_name(new_zone)}.
    ~ enemies[index].zone = new_zone
}
