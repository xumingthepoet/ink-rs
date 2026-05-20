=== module game ===
STRUCT Recipe {
    name: string
    ingredient_a: string
    amount_a: int
    ingredient_b: string
    amount_b: int
    output: string
}

VAR recipes: Recipe[] = [
    %Recipe{
        name: "Potion",
        ingredient_a: "Herb",
        amount_a: 2,
        ingredient_b: "Water",
        amount_b: 1,
        output: "Potion"
    },
    %Recipe{
        name: "Smoke Bomb",
        ingredient_a: "Thread",
        amount_a: 1,
        ingredient_b: "Ash",
        amount_b: 1,
        output: "Smoke_Bomb"
    },
    %Recipe{
        name: "Arcane Orb",
        ingredient_a: "Potion",
        amount_a: 1,
        ingredient_b: "Essence",
        amount_b: 2,
        output: "Arcane_Orb"
    }
]

VAR inventory: Dict<string, int> = %{
    "Herb": 4,
    "Water": 2,
    "Thread": 1,
    "Ash": 1,
    "Essence": 2,
    "Potion": 0,
    "Smoke_Bomb": 0,
    "Arcane_Orb": 0
}

== main ==
Crafting session begins.
-> show_inventory(0)
-> craft_recipes(0)
-> show_inventory(0)
-> DONE

== craft_recipes(index: int) ==
{ if index >= LEN(recipes):
    No more recipes.
    ->->
- else:
    ~ temp recipe: Recipe = recipes[index]
    { if can_craft(recipe):
        Crafting {recipe.name}.
        ~ inventory[recipe.ingredient_a] = inventory[recipe.ingredient_a] - recipe.amount_a
        ~ inventory[recipe.ingredient_b] = inventory[recipe.ingredient_b] - recipe.amount_b
        ~ inventory[recipe.output] = inventory[recipe.output] + 1
    - else:
        {recipe.name} blocked by inventory.
    }
    -> craft_recipes(index + 1)
}

== function can_craft(recipe: Recipe) => bool ==
~ return inventory[recipe.ingredient_a] >= recipe.amount_a and inventory[recipe.ingredient_b] >= recipe.amount_b

== show_inventory(index: int) ==
~ temp names: string[] = ["Herb", "Water", "Thread", "Ash", "Essence", "Potion", "Smoke_Bomb", "Arcane_Orb"]
{ if index >= LEN(names):
    ->->
- else:
    { names[index] + ": " + INT(inventory[names[index]])}
    -> show_inventory(index + 1)
}
