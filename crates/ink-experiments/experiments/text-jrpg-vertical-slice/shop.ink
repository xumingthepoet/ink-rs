=== module shop ===
FROM items IMPORT item_name, item_price, item_count, spend_gold, add_item, gold, count_label

STRUCT ShopStock {
    item_id: int
    unlock_flag: int
}

CONST SHOP_VILLAGE: int = 1
CONST village_stock: ShopStock[] = [
    %ShopStock{ item_id: 1, unlock_flag: 0 },
    %ShopStock{ item_id: 2, unlock_flag: 0 },
    %ShopStock{ item_id: 4, unlock_flag: 8 }
]

== show_village_shop ==
Village shop stock.
Gold: {items::count_label(items::gold)}
{ for stock in village_stock:
{items::item_name(stock.item_id)} price {items::count_label(items::item_price(stock.item_id))} owned {items::count_label(items::item_count(stock.item_id))}
}
->->

== buy(item_id: int) ==
{ if items::spend_gold(items::item_price(item_id)):
    ~ items::add_item(item_id, 1)
    Bought {items::item_name(item_id)} for {items::count_label(items::item_price(item_id))} gold.
- else:
    Could not buy {items::item_name(item_id)}; need {items::count_label(items::item_price(item_id))} gold.
}
Gold now {items::count_label(items::gold)}.
->->
