=== module game ===
STRUCT Player {
hp: int
name: string
}
VAR state: Player = %Player{ hp: 4, name: "Ada" }
VAR items: int[] = [1]

== main ==
~ state.hp += 1
~ state.name += "!"
~ items[0] += 1
{state.hp}|{state.name}|{items[0]}
-> DONE
