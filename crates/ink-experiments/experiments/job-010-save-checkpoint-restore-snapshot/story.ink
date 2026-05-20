=== module game ===
STRUCT HeroState {
    name: string
    hp: int
    stamina: int
    gold: int
    inventory: Dict<string, int>
}

VAR hero: HeroState = %HeroState{
    name: "Mira",
    hp: 20,
    stamina: 12,
    gold: 35,
    inventory: %{
        "herb": 4,
        "elixir": 0,
        "coin": 35
    }
}

VAR checkpoint: HeroState

== main ==
Runtime session starts.
-> show_state("Baseline") ->
~ checkpoint = hero
Combat strikes for two rounds.
~ hero.hp = hero.hp - 7
~ hero.stamina = hero.stamina - 5
~ hero.gold = hero.gold - 12
~ hero.inventory["coin"] = hero.inventory["coin"] - 12
-> show_state("After combat") ->
Restoring checkpoint snapshot.
~ hero = checkpoint
-> show_state("Restored") ->
-> DONE

== show_state(label: string) ==
-- {label} --
HP: {hero.hp}
Stamina: {hero.stamina}
Gold: {hero.gold}
Herbs: {hero.inventory["herb"]}
Elixir: {hero.inventory["elixir"]}
Coins: {hero.inventory["coin"]}
->->
