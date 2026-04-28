=== module game ===

== main ==
{count_down(1500, 0)}|{carry(3, 0)}|{fact(5)}
-> DONE

== function count_down(n: int, acc: int) => int ==
{ n <= 0:
    ~ return acc
- else:
    ~ return count_down(n - 1, acc + 1)
}

== function carry(n: int, seen: int) => int ==
{ n <= 0:
    ~ return seen
- else:
    ~ return carry(n - 1, n)
}

== function fact(n: int) => int ==
{ n <= 1:
    ~ return 1
- else:
    ~ return n * fact(n - 1)
}
