STRUCT Player {
    hp: int
}

VAR party: Player[] = [{ hp: 7 }]
VAR seed_values: int[] = [2, 3]

{add(2, 3)}|{label("Ada")}|{first_hp(party)}
~ temp values: int[] = identity(seed_values)
{values[0]}|{values[1]}
-> DONE

== function add(a: int, b: int) -> int ==
~ return a + b

== function label(name: string) -> string ==
~ return name + "!"

== function first_hp(players: Player[]) -> int ==
~ return players[0].hp

== function identity(values: int[]) -> int[] ==
~ return values
