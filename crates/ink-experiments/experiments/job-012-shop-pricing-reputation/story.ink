=== module game ===
VAR item_base_price: Dict<string, int> = %{
    "Potion": 18,
    "Chainmail": 140,
    "Hunting Bow": 110,
    "Nightshade": 35
}

VAR item_faction: Dict<string, string> = %{
    "Potion": "Merchants",
    "Chainmail": "Guild",
    "Hunting Bow": "Frontiersmen",
    "Nightshade": "Outlaws"
}

VAR discount_flags: Dict<string, bool> = %{
    "Potion": true,
    "Chainmail": false,
    "Hunting Bow": false,
    "Nightshade": true
}

VAR faction_reputation: Dict<string, int> = %{
    "Merchants": 4,
    "Guild": 7,
    "Frontiersmen": 0,
    "Outlaws": -6
}
VAR sale_event: bool = true

== main ==
Shop pricing with reputation and discounts:
Initial pricing
Potion (Merchants, rep {faction_reputation["Merchants"]})
Base: {item_base_price["Potion"]}
Discounted: {discount_flags["Potion"]}
Price: {compute_price("Potion")}
Chainmail (Guild, rep {faction_reputation["Guild"]})
Base: {item_base_price["Chainmail"]}
Discounted: {discount_flags["Chainmail"]}
Price: {compute_price("Chainmail")}
Hunting Bow (Frontiersmen, rep {faction_reputation["Frontiersmen"]})
Base: {item_base_price["Hunting Bow"]}
Discounted: {discount_flags["Hunting Bow"]}
Price: {compute_price("Hunting Bow")}
Nightshade (Outlaws, rep {faction_reputation["Outlaws"]})
Base: {item_base_price["Nightshade"]}
Discounted: {discount_flags["Nightshade"]}
Price: {compute_price("Nightshade")}
~ faction_reputation["Outlaws"] = faction_reputation["Outlaws"] + 8
~ faction_reputation["Guild"] = faction_reputation["Guild"] + 2
~ discount_flags["Chainmail"] = true
~ sale_event = false
After reputation shifts
Potion (Merchants, rep {faction_reputation["Merchants"]})
Base: {item_base_price["Potion"]}
Discounted: {discount_flags["Potion"]}
Price: {compute_price("Potion")}
Chainmail (Guild, rep {faction_reputation["Guild"]})
Base: {item_base_price["Chainmail"]}
Discounted: {discount_flags["Chainmail"]}
Price: {compute_price("Chainmail")}
Hunting Bow (Frontiersmen, rep {faction_reputation["Frontiersmen"]})
Base: {item_base_price["Hunting Bow"]}
Discounted: {discount_flags["Hunting Bow"]}
Price: {compute_price("Hunting Bow")}
Nightshade (Outlaws, rep {faction_reputation["Outlaws"]})
Base: {item_base_price["Nightshade"]}
Discounted: {discount_flags["Nightshade"]}
Price: {compute_price("Nightshade")}

-> DONE

== function compute_price(item_name: string) => int ==
~ temp base: int = item_base_price[item_name]
~ temp faction: string = item_faction[item_name]
~ temp rep: int = faction_reputation[faction]
~ temp percent: int = 100

{ if rep >= 8:
    ~ percent = 80
- else:
    { if rep >= 5:
        ~ percent = 90
    - else:
        { if rep >= 0:
            ~ percent = 100
        - else:
            { if rep >= -3:
                ~ percent = 125
            - else:
                ~ percent = 150
            }
        }
    }
}

~ temp scaled_price: int = base * percent
~ temp final_price: int = scaled_price / 100

{ if discount_flags[item_name]:
    ~ scaled_price = final_price * 85
    ~ final_price = scaled_price / 100
}
{ if sale_event:
    ~ scaled_price = final_price * 90
    ~ final_price = scaled_price / 100
}

{ if final_price < 1:
    ~ final_price = 1
}

~ return final_price
