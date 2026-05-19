=== module game ===
FROM shop IMPORT price, describe
VAR gold: int = 5

== main ==
{shop::describe()}
Gold: {gold}
Price: {shop::price}
~ shop::price += 2
Updated price: {shop::price}
-> END

=== module shop ===
VAR price: int = 3

== function describe() => string ==
~ return "The shop is open."
