=== module game ===
STRUCT Player {
    hp: int
}

VAR grid: int[][] = [[1, 2], []]
VAR party: Player[] = [%Player{ hp: 4 }, %Player{ hp: 8 }]
VAR party_copy: Player[] = party

== main ==
~ party_copy[0].hp = 1
{grid[0][1]}|{LEN(grid[1])}|{party[0].hp}|{party_copy[0].hp}|{party == party_copy}
