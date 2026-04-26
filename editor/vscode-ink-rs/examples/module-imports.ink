=== module game ===
IMPORT price, describe FROM shop
VAR gold: int = 5

== main ==
{shop::describe()}
Gold: {gold}
Price: {shop::price}
-> END

=== module shop ===
VAR price: int = 3

== function describe() => string ==
~ return "The shop is open."

// Removed syntax highlighted as invalid by the grammar:
INCLUDE old-file.ink
