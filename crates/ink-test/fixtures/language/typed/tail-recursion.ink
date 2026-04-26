{count_down(1500, 0)}|{sum_to(5, 0)}
-> DONE

== function count_down(n: int, acc: int) => int ==
{ n <= 0:
    ~ return acc
- else:
    ~ return count_down(n - 1, acc + 1)
}

== function sum_to(n: int, acc: int) => int ==
{ n <= 0:
    ~ return acc
- else:
    ~ return sum_to(n - 1, acc + n)
}
