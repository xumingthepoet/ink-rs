=== module game ===
STRUCT Player {
hp: int
}
STRUCT Bag {
items: int[]
}
VAR items: int[] = [1, 2, 3, 4]
VAR players: Player[] = [%Player{ hp: 1 }, %Player{ hp: 2 }]
VAR nested: int[][] = [[1], [2, 3]]
VAR bag: Bag = %Bag{ items: [8, 9] }

== main ==
before{ARRAY_REMOVE(items, 0)}after|{items[0]}|{LEN(items)}
~ ARRAY_REMOVE(items, 1)
{items[0]}|{items[1]}|{LEN(items)}
~ ARRAY_REMOVE(items, 1)
{items[0]}|{LEN(items)}
~ ARRAY_REMOVE(players, 0)
{players[0].hp}|{LEN(players)}
~ ARRAY_REMOVE(nested[1], 0)
{nested[1][0]}|{LEN(nested[1])}
~ ARRAY_REMOVE(bag.items, 0)
{bag.items[0]}|{LEN(bag.items)}
-> DONE
