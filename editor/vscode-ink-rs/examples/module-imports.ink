=== module game ===
IMPORT {
    price, describe, market_day,
} FROM shop
VAR gold: int = 5
VAR day_stage: int = 1

== main ==
{shop::describe()}
Gold: {gold}
Price: {shop::price}
{ if gold > 0:
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
-> END

=== module shop ===
VAR price: int = 3

== function describe() => string ==
~ return "The shop is open."

== market_day ==
Noon crowds fill the street.
->->

// Removed syntax highlighted as invalid by the grammar:
INCLUDE old-file.ink
