=== module game ===

STRUCT Artifact {
    tag: string
    true_name: string
    curse_vector: string
    base_value: int
}

STRUCT Probe {
    method: string
    clue: string
    risk_gain: int
    confidence_gain: int
}

STRUCT Clue {
    method: string
    note: string
    confidence: int
    risk_after: int
    found: bool
}

VAR relic: Artifact = %Artifact{
    tag: "copper reliquary shard",
    true_name: "mirror-bound seal of Marrowmire",
    curse_vector: "latent possessive bind",
    base_value: 110
}

VAR probes: Probe[] = [
    %Probe{
        method: "lamp-lens scan",
        clue: "etched constellations align with oath-breaker marks",
        risk_gain: 8,
        confidence_gain: 12
    },
    %Probe{
        method: "resin scrape sample",
        clue: "a bitter resin bloom forms where heat is applied",
        risk_gain: 18,
        confidence_gain: 16
    },
    %Probe{
        method: "chime resonance",
        clue: "tones bend as if a hidden chamber shifts in phase",
        risk_gain: 27,
        confidence_gain: 24
    },
    %Probe{
        method: "mirror divination",
        clue: "reflections show names without mouths asking for release",
        risk_gain: 34,
        confidence_gain: 26
    },
    %Probe{
        method: "burning salt circle",
        clue: "tiny sparks form along the boundary rune",
        risk_gain: 40,
        confidence_gain: 20
    }
]

VAR clues: Clue[] = [
    %Clue{method: "", note: "", confidence: 0, risk_after: 0, found: false},
    %Clue{method: "", note: "", confidence: 0, risk_after: 0, found: false},
    %Clue{method: "", note: "", confidence: 0, risk_after: 0, found: false},
    %Clue{method: "", note: "", confidence: 0, risk_after: 0, found: false},
    %Clue{method: "", note: "", confidence: 0, risk_after: 0, found: false}
]

VAR total_confidence: int = 0
VAR curse_risk: int = 0
VAR stopped: bool = false

== main ==
The reliquary shard arrives with no provenance note.
~ print_setup()
~ inspect_all(0)
~ reveal_outcome()
-> DONE

== function print_setup() => void ==
Target: "{relic.tag}".
Baseline value estimate: {relic.base_value} silver.

== function inspect_all(index: int) => void ==
{ if index >= LEN(probes):
    ~ return
- else:
    { if stopped:
        Investigations pause while the relic hums on its own.
    - else:
        ~ run_probe(index)
        ~ inspect_all(index + 1)
    }
}

== function run_probe(index: int) => void ==
~ temp probe: Probe = probes[index]
Investigator uses {probe.method}.
~ temp risk: int = curse_risk + probe.risk_gain
~ temp confidence: int = probe.confidence_gain
{ if risk > 90:
    ~ risk = 90
}
{ if curse_risk >= 70:
    ~ confidence = confidence / 3
}
~ total_confidence = total_confidence + confidence
~ curse_risk = curse_risk + probe.risk_gain
~ clues[index].method = probe.method
~ clues[index].risk_after = curse_risk
~ clues[index].found = true
{ if curse_risk > 95:
    This run triggers an unsafe surge.
    ~ clues[index].note = "dangerous surge, corrupted read"
    ~ clues[index].confidence = 0
    ~ stopped = true
- else:
    { if confidence < 8:
        ~ clues[index].note = "fuzzy signal, low confidence"
        ~ clues[index].confidence = confidence
        ~ print_clue_fragment(probe, confidence)
    - else:
        ~ clues[index].note = "clear signal: " + probe.clue
        ~ clues[index].confidence = confidence
        ~ print_clue_fragment(probe, confidence)
    }
}

== function print_clue_fragment(probe: Probe, confidence: int) => void ==
~ temp status: string = confidence_state(confidence)
Clue ({status}): {probe.clue}

== function confidence_state(value: int) => string ==
{ if value >= 20:
    ~ return "strong"
- else:
    { if value >= 10:
        ~ return "moderate"
    - else:
        ~ return "weak"
    }
}

== function reveal_outcome() => void ==
The identification ledger closes.
~ print_clues(0)
~ temp outcome: string = reveal_label()
~ temp final_note: string = outcome_risk_note()
The final reading is: {outcome}
Curse status: {final_note}

== function reveal_label() => string ==
{ if stopped:
    ~ return "Unresolved: unstable trace, manual binding required before movement"
- else:
    { if total_confidence >= 70 and curse_risk <= 60:
        ~ return relic.true_name
    - else:
        { if total_confidence >= 70 and curse_risk > 60:
            ~ return relic.true_name + " (curse still active)"
        - else:
            { if total_confidence >= 45:
                ~ return "likely relic, but identity incomplete"
            - else:
                ~ return "identity unreadable"
            }
        }
    }
}

== function outcome_risk_note() => string ==
{ if curse_risk >= 95:
    ~ return "Immediate binding required; do not approach without a ward"
- else:
    { if curse_risk >= 70:
        ~ return "High curse pressure, inspect via safe shell next"
    - else:
        { if curse_risk >= 40:
            ~ return "Some curse pressure, monitor for escalation"
        - else:
            ~ return "Cursed signals low; normal handling safe"
        }
    }
}

== function print_clues(index: int) => void ==
{ if index >= LEN(clues):
    ~ return
- else:
    { if clues[index].found:
        Method {clues[index].method}: confidence {clues[index].confidence}, risk now {clues[index].risk_after}.
        Note: {clues[index].note}
    }
    ~ print_clues(index + 1)
}
