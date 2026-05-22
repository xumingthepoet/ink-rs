=== module game ===
VAR numbers: int[] = [1, 2, 3]
VAR names: string[] = ["Ada", "Grace"]
VAR scores: Dict<string, int> = %{"grace": 2, "ada": 1}
VAR labels: Dict<int, string> = %{2: "two", 1: "one"}
VAR rows: int[][] = [[1, 2], [3]]

== main ==
~ temp total: int = 0
{ for number in numbers:
{ if number > 1:
big {number}
- else:
small {number}
}
{ switch number:
- 2:
two
- else:
other
}
~ total += number
}
total {total}
{ for index, name in names:
{index}:{name}
}
{ for key, value in scores:
{key}={value}
}
{ for id, label in labels:
{id}={label}
}
{ for row in rows:
row
{ for value in row:
{value}
}
}
