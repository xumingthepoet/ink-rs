=== module shop ===
FROM events IMPORT has_flag, flag_label, FLAG_FOREST_ENCOUNTER_CLEARED
FROM items IMPORT item_name, item_price, item_count, spend_gold, add_item, gold, ITEM_POTION, ITEM_ANTIDOTE, ITEM_MINE_CHARM

STRUCT ShopStock {
    item_id: int
    unlock_flag: int
}

CONST SHOP_VILLAGE: int = 1
CONST village_stock: ShopStock[] = [
    %ShopStock{ item_id: items::ITEM_POTION, unlock_flag: 0 },
    %ShopStock{ item_id: items::ITEM_ANTIDOTE, unlock_flag: 0 },
    %ShopStock{ item_id: items::ITEM_MINE_CHARM, unlock_flag: events::FLAG_FOREST_ENCOUNTER_CLEARED }
]

== show_village_shop ==
Village shop stock.
Gold: {to_str(items::gold)}
{ for stock in village_stock:
{ if stock_is_available(stock):
{items::item_name(stock.item_id)} price {to_str(items::item_price(stock.item_id))} owned {to_str(items::item_count(stock.item_id))}
- else:
{items::item_name(stock.item_id)} locked by {events::flag_label(stock.unlock_flag)}
}
}
->->

== buy(item_id: int) ==
{ if !item_is_stocked(item_id):
    {items::item_name(item_id)} is not stocked yet.
- else:
    { if items::spend_gold(items::item_price(item_id)):
        ~ items::add_item(item_id, 1)
        Bought {items::item_name(item_id)} for {to_str(items::item_price(item_id))} gold.
    - else:
        Could not buy {items::item_name(item_id)}; need {to_str(items::item_price(item_id))} gold.
    }
}
Gold now {to_str(items::gold)}.
->->

== function stock_is_available(stock: ShopStock) => bool ==
{ if stock.unlock_flag == 0:
    ~ return true
- else:
    ~ return events::has_flag(stock.unlock_flag)
}

== function item_is_stocked(item_id: int) => bool ==
~ temp stocked: bool = false
{ for stock in village_stock:
    { if stock.item_id == item_id && stock_is_available(stock):
        ~ stocked = true
    }
}
~ return stocked
