=== module game ===
STRUCT Player {
hp: int
name: string
}
VAR numbers: int[] = [1, 2, 3]
VAR matrix: int[][] = [[1, 2], []]
VAR party: Player[] = [{ hp: 10, name: "Ada" }]

== main ==
{numbers}|{matrix}|{party}
-> DONE