=== module game ===
IMPORT {
    price, describe, market_day,
} FROM shop
STRUCT VisitState {
    gold: int
    shop_open: bool
}
CONST default_state: VisitState = { gold: 1, shop_open: true }
VAR visit_state: VisitState = { gold: 5, shop_open: true }
CONST states: VisitState[] = [
    {
        gold: 1,
        shop_open: true,
    },
]
VAR gold: int = 5
VAR day_stage: int = 1

== main ==
{shop::describe()}
Gold: {gold}
Price: {shop::price}
{ if shop::price > 0 and gold > 0:
The purse still has weight.
- else:
The purse is empty.
}
{ switch day_stage:
- 0: Dawn breaks over the market.
- 1:
    -> shop::market_day ->
- else: Night closes the shutters.
}
* {not shop::closed} Visit the market -> shop::market_day

=== module shop ===
VAR price: int = 3
VAR closed: bool = false

== function describe() => string ==
~ return "The shop is open."

== market_day ==
Noon crowds fill the street.
->->

// Removed syntax highlighted as invalid by the grammar:
INCLUDE old-file.ink
