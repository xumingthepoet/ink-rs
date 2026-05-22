=== module items ===

STRUCT ItemDef {
    name: string
    price: int
    kind: string
}

CONST ITEM_POTION: int = 1
CONST ITEM_ANTIDOTE: int = 2
CONST ITEM_MOONLEAF: int = 3
CONST ITEM_MINE_CHARM: int = 4

CONST item_defs: Dict<int, ItemDef> = %{
    1: %ItemDef{ name: "Potion", price: 6, kind: "heal" },
    2: %ItemDef{ name: "Antidote", price: 8, kind: "cure" },
    3: %ItemDef{ name: "Moonleaf", price: 0, kind: "quest" },
    4: %ItemDef{ name: "Mine Charm", price: 20, kind: "charm" }
}

VAR gold: int = 10
VAR inventory: Dict<int, int> = %{1: 1}

== function item_name(item_id: int) => string ==
~ return item_defs[item_id].name

== function item_price(item_id: int) => int ==
~ return item_defs[item_id].price

== function item_count(item_id: int) => int ==
{ if DICT_HAS(inventory, item_id):
    ~ return inventory[item_id]
- else:
    ~ return 0
}

== function add_item(item_id: int, quantity: int) => void ==
~ inventory[item_id] = item_count(item_id) + quantity

== function remove_item(item_id: int, quantity: int) => bool ==
{ if item_count(item_id) < quantity:
    ~ return false
- else:
    ~ inventory[item_id] = item_count(item_id) - quantity
    ~ return true
}

== function can_afford(cost: int) => bool ==
~ return gold >= cost

== function spend_gold(cost: int) => bool ==
{ if gold < cost:
    ~ return false
- else:
    ~ gold -= cost
    ~ return true
}

== function add_gold(amount: int) => void ==
~ gold += amount

== function set_gold(amount: int) => void ==
~ gold = amount

== function set_item_count(item_id: int, quantity: int) => void ==
~ inventory[item_id] = quantity

== function inventory_summary() => string ==
~ temp text: string = ""
{ for item_id, count in inventory:
    { if count > 0:
        { if text == "":
            ~ text = item_name(item_id) + " x" + to_str(count)
        - else:
            ~ text = text + ", " + item_name(item_id) + " x" + to_str(count)
        }
    }
}
{ if text == "":
    ~ return "empty"
- else:
    ~ return text
}
