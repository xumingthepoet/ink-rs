=== module equipment ===

STRUCT EquipmentDef {
    name: string
    slot: int
    attack: int
    defense: int
}

CONST SLOT_WEAPON: int = 1
CONST SLOT_ARMOR: int = 2
CONST SLOT_CHARM: int = 3
CONST EQUIP_WORN_SABER: int = 1
CONST EQUIP_GUARD_BADGE: int = 2

CONST equipment_defs: Dict<int, EquipmentDef> = %{
    1: %EquipmentDef{ name: "Worn Saber", slot: 1, attack: 2, defense: 0 },
    2: %EquipmentDef{ name: "Guard Badge", slot: 3, attack: 0, defense: 1 }
}

VAR equipped: Dict<int, int> = %{1: 1}

== function equip(actor_id: int, equipment_id: int) => void ==
~ equipped[actor_id] = equipment_id

== function equipped_name(actor_id: int) => string ==
{ if DICT_HAS(equipped, actor_id):
    ~ return equipment_defs[equipped[actor_id]].name
- else:
    ~ return "none"
}

== function attack_bonus(actor_id: int) => int ==
{ if DICT_HAS(equipped, actor_id):
    ~ return equipment_defs[equipped[actor_id]].attack
- else:
    ~ return 0
}

== function equipment_summary() => string ==
~ temp text: string = ""
{ for actor_id, equipment_id in equipped:
    { if text == "":
        ~ text = actor_label(actor_id) + "=" + equipment_defs[equipment_id].name
    - else:
        ~ text = text + ", " + actor_label(actor_id) + "=" + equipment_defs[equipment_id].name
    }
}
~ return text

== function actor_label(actor_id: int) => string ==
{ if actor_id == 1:
    ~ return "Lio"
- else:
    ~ return "Ren"
}
