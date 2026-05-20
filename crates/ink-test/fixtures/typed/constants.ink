=== module game ===
STRUCT Stats {
hp: int
ready: bool
}
CONST default_stats: Stats = %Stats{ hp: 7 }
CONST party: Stats[] = [%Stats{ hp: 1 }, %Stats{}]
VAR copied_stats: Stats = default_stats
VAR copied_party: Stats[] = party

== main ==
{default_stats}|{party}|{copied_stats}|{copied_party}
-> DONE
