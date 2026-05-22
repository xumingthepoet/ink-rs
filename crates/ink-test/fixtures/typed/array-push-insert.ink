=== module game ===
ENUM State { Idle Busy Done }
STRUCT Player {
hp: int
}
STRUCT Bag {
items: int[]
}
VAR items: int[] = []
VAR players: Player[] = []
VAR states: State[] = []
VAR lookups: Dict<string, int>[] = []
VAR lookup: Dict<string, int> = %{"ada": 10}
VAR nested: int[][] = []
VAR row: int[] = [1, 2]
VAR second_row: int[] = [3]
VAR bag: Bag = %Bag{ items: [] }

== main ==
before{ARRAY_PUSH(items, 1)}after|{LEN(items)}|{items[0]}
~ ARRAY_PUSH(items, 3)
~ ARRAY_INSERT(items, 1, 2)
{items[0]}|{items[1]}|{items[2]}|{LEN(items)}
~ ARRAY_INSERT(items, 0, 0)
~ ARRAY_INSERT(items, LEN(items), 4)
{items[0]}|{items[4]}|{LEN(items)}
~ ARRAY_PUSH(players, %Player{ hp: 7 })
~ ARRAY_INSERT(players, 0, %Player{ hp: 5 })
{players[0].hp}|{players[1].hp}|{LEN(players)}
~ ARRAY_PUSH(states, State.Busy)
~ ARRAY_INSERT(states, 0, State.Idle)
{states[0]}|{states[1]}|{LEN(states)}
~ ARRAY_PUSH(lookups, lookup)
{lookups[0]["ada"]}|{LEN(lookups)}
~ ARRAY_PUSH(nested, row)
~ ARRAY_INSERT(nested, LEN(nested), second_row)
{nested[0][1]}|{nested[1][0]}|{LEN(nested)}
~ ARRAY_PUSH(bag.items, 8)
~ ARRAY_INSERT(bag.items, 0, 7)
{bag.items[0]}|{bag.items[1]}|{LEN(bag.items)}
