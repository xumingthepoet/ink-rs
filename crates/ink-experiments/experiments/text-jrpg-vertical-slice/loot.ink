=== module loot ===
FROM items IMPORT add_item, add_gold, item_name, count_label, ITEM_MOONLEAF, ITEM_MINE_CHARM

STRUCT LootEntry {
    item_id: int
    quantity: int
    gold: int
}

CONST LOOT_FOREST: int = 1
CONST LOOT_MINE: int = 2
CONST loot_tables: Dict<int, LootEntry[]> = %{
    1: [
        %LootEntry{ item_id: items::ITEM_MOONLEAF, quantity: 1, gold: 0 }
    ],
    2: [
        %LootEntry{ item_id: items::ITEM_MINE_CHARM, quantity: 1, gold: 5 }
    ]
}

== resolve(table_id: int) ==
Loot table {label(table_id)}.
{ for entry in loot_tables[table_id]:
    { if entry.item_id > 0:
        ~ items::add_item(entry.item_id, entry.quantity)
        Gained {items::item_name(entry.item_id)} x{items::count_label(entry.quantity)}.
    }
    { if entry.gold > 0:
        ~ items::add_gold(entry.gold)
        Gained {items::count_label(entry.gold)} gold.
    }
}
->->

== function label(table_id: int) => string ==
{ if table_id == LOOT_FOREST:
    ~ return "forest"
- else:
    ~ return "mine"
}
