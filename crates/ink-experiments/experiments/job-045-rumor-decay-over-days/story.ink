=== module game ===

STRUCT Rumor {
    id: int
    source: string
    topic: string
    claim: string
}

VAR rumor_by_id: Dict<int, int> = %{
    501: 0,
    502: 1,
    503: 2
}

VAR rumor_catalog: Rumor[] = [
    %Rumor{
        id: 501,
        source: "Market Bell",
        topic: "trade",
        claim: "The harbor tax is canceled for a week."
    },
    %Rumor{
        id: 502,
        source: "Captain Rook",
        topic: "guards",
        claim: "A second patrol now circles the old east gate."
    },
    %Rumor{
        id: 503,
        source: "Oracle",
        topic: "bridge",
        claim: "The river bridge will be repaired after the flood week."
    }
]

VAR rumor_strength: Dict<int, int> = %{
    501: 6,
    502: 3,
    503: -2
}

VAR source_reliability: Dict<string, int> = %{
    "Market Bell": 72,
    "Captain Rook": 66,
    "Oracle": 58
}

VAR source_mentions: Dict<string, int> = %{
    "Market Bell": 0,
    "Captain Rook": 0,
    "Oracle": 0
}

VAR public_belief: Dict<string, int> = %{
    "trade": 50,
    "guards": 47,
    "bridge": 51
}

== main ==
Rumor landscape over days.
~ print_snapshot("Initial")
~ simulate_day_one()
~ simulate_day_two()
~ simulate_day_three()
~ simulate_day_four()
~ simulate_day_five()
Final civic belief readout.
~ print_belief()
-> DONE

== function simulate_day_one() => void ==
-- Day 1 --
~ decay_rumor(501)
~ decay_rumor(502)
~ decay_rumor(503)
~ reinforce_rumor(501, 4, "trade")
~ apply_rumor_to_belief(501)
~ apply_rumor_to_belief(502)
~ apply_rumor_to_belief(503)
~ print_snapshot("After day 1")

== function simulate_day_two() => void ==
-- Day 2 --
~ decay_rumor(501)
~ decay_rumor(502)
~ decay_rumor(503)
~ reinforce_rumor(502, 3, "guards")
~ reinforce_rumor(503, -3, "bridge")
~ apply_rumor_to_belief(501)
~ apply_rumor_to_belief(502)
~ apply_rumor_to_belief(503)
~ print_snapshot("After day 2")

== function simulate_day_three() => void ==
-- Day 3 --
~ decay_rumor(501)
~ decay_rumor(502)
~ decay_rumor(503)
~ reinforce_rumor(501, -2, "trade")
~ reinforce_rumor(503, 5, "bridge")
~ apply_rumor_to_belief(501)
~ apply_rumor_to_belief(502)
~ apply_rumor_to_belief(503)
~ print_snapshot("After day 3")

== function simulate_day_four() => void ==
-- Day 4 --
~ decay_rumor(501)
~ decay_rumor(502)
~ decay_rumor(503)
~ reinforce_rumor(502, 1, "guards")
~ apply_rumor_to_belief(501)
~ apply_rumor_to_belief(502)
~ apply_rumor_to_belief(503)
~ print_snapshot("After day 4")

== function simulate_day_five() => void ==
-- Day 5 --
~ decay_rumor(501)
~ decay_rumor(502)
~ decay_rumor(503)
~ reinforce_rumor(501, -1, "trade")
~ reinforce_rumor(503, -2, "bridge")
~ apply_rumor_to_belief(501)
~ apply_rumor_to_belief(502)
~ apply_rumor_to_belief(503)
~ print_snapshot("After day 5")

== function decay_rumor(rumor_id: int) => void ==
~ temp old_strength: int = rumor_strength[rumor_id]
~ temp rumor_index: int = rumor_by_id[rumor_id]
~ temp rumor: Rumor = rumor_catalog[rumor_index]
{ if old_strength > 0:
    { if old_strength > 2:
        ~ rumor_strength[rumor_id] = old_strength - 2
    - else:
        ~ rumor_strength[rumor_id] = 0
    }
- else:
    { if old_strength < 0:
        { if old_strength < -2:
            ~ rumor_strength[rumor_id] = old_strength + 2
        - else:
            ~ rumor_strength[rumor_id] = 0
        }
    }
}
~ temp reduced: int = rumor_strength[rumor_id]
{ if reduced == 0:
    The crowd loses interest in "{rumor.claim}".
- else:
    "{rumor.source}" rumor "{rumor.claim}" drifts to strength {reduced}.
}

== function reinforce_rumor(rumor_id: int, delta: int, topic_hint: string) => void ==
~ temp rumor_index: int = rumor_by_id[rumor_id]
~ temp rumor: Rumor = rumor_catalog[rumor_index]
~ temp old: int = rumor_strength[rumor_id]
~ rumor_strength[rumor_id] = old + delta
~ source_mentions[rumor.source] = source_mentions[rumor.source] + 1
~ temp source_update: int = source_mentions[rumor.source]
Rumor "{rumor.claim}" is heard again in topic {topic_hint}. Delta {delta}. Mentions for {rumor.source}: {source_update}.

== function apply_rumor_to_belief(rumor_id: int) => void ==
~ temp rumor_index: int = rumor_by_id[rumor_id]
~ temp rumor: Rumor = rumor_catalog[rumor_index]
~ temp source_score: int = source_reliability[rumor.source]
~ temp strength: int = rumor_strength[rumor_id]
~ temp delta: int = 0

{ if strength >= 7:
    ~ delta = 4
- else:
    { if strength >= 4:
        ~ delta = 3
    - else:
        { if strength >= 1:
            ~ delta = 1
        - else:
            { if strength <= -6:
                ~ delta = -4
            - else:
                { if strength <= -3:
                    ~ delta = -2
                - else:
                    { if strength < 0:
                        ~ delta = -1
                    - else:
                        ~ delta = 0
                    }
                }
            }
        }
    }
}

{ if source_score >= 70:
    ~ delta = delta + 1
- else:
    { if source_score <= 55:
        ~ delta = delta - 1
    }
}

~ adjust_belief(rumor.topic, delta)

{ if strength >= 0:
    Public belief gains {delta} from "{rumor.claim}" on {rumor.topic}.
- else:
    ~ temp loss: int = 0 - delta
    Public belief loses {loss} from "{rumor.claim}" on {rumor.topic}.
}

{ if delta == 0:
    No measurable shift from "{rumor.claim}".
}

{ if strength > 4:
    ~ source_reliability[rumor.source] = source_reliability[rumor.source] + 1
- else:
    { if strength < -4:
        ~ source_reliability[rumor.source] = source_reliability[rumor.source] - 1
    }
}

== function adjust_belief(topic: string, delta: int) => void ==
~ temp old: int = public_belief[topic]
~ temp fresh: int = old + delta
{ if fresh < 0:
    ~ public_belief[topic] = 0
- else:
    { if fresh > 100:
        ~ public_belief[topic] = 100
    - else:
        ~ public_belief[topic] = fresh
    }
}

== function print_snapshot(label: string) => void ==
-- {label} --
~ print_rumor(501)
~ print_rumor(502)
~ print_rumor(503)
~ print_belief()

== function print_rumor(rumor_id: int) => void ==
~ temp rumor_index: int = rumor_by_id[rumor_id]
~ temp rumor: Rumor = rumor_catalog[rumor_index]
Rumor {rumor_id} by {rumor.source}
Claim: {rumor.claim}
Strength: {rumor_strength[rumor_id]} | Source reliability: {source_reliability[rumor.source]}
---

== function print_belief() => void ==
Public belief:
Trade: {public_belief["trade"]}
Guards: {public_belief["guards"]}
Bridge: {public_belief["bridge"]}
