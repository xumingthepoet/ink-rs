=== module game ===
STRUCT Stats {
hp: int
ready: bool
}
CONST default_stats: Stats = { hp: 7 }
CONST party: Stats[] = [{ hp: 1 }, {}]
VAR copied_stats: Stats = default_stats
VAR copied_party: Stats[] = party

== main ==
{default_stats}|{party}|{copied_stats}|{copied_party}
-> DONE
