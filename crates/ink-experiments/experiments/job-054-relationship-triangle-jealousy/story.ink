=== module game ===

VAR lyra_affection: int = 6
VAR kest_affection: int = 4
VAR orin_affection: int = 7

VAR lyra_jealousy: int = 2
VAR kest_jealousy: int = 5
VAR orin_jealousy: int = 3

== main ==
Three-way relation tracking sample.
~ log_triangle("Opening baseline")
~ share_with_lyra()
~ log_triangle("After shared tea with Lyra")
~ spar_with_kest()
~ log_triangle("After sparring praise for Kest")
~ walk_with_orin()
~ log_triangle("After private walk with Orin")
~ public_reassurance()
~ log_triangle("After public reassurance")
~ final_consequence()
-> DONE

== function log_triangle(phase: string) => void ==
Triangle state: {phase}
Lyra: affection {lyra_affection} ({affection_tone(lyra_affection)}), jealousy {lyra_jealousy} ({jealousy_tone(lyra_jealousy)}).
Kest: affection {kest_affection} ({affection_tone(kest_affection)}), jealousy {kest_jealousy} ({jealousy_tone(kest_jealousy)}).
Orin: affection {orin_affection} ({affection_tone(orin_affection)}), jealousy {orin_jealousy} ({jealousy_tone(orin_jealousy)}).

== function share_with_lyra() => void ==
You spend quality time with Lyra and help repair her tools.
~ lyra_affection = lyra_affection + 3
{ if lyra_affection - kest_affection >= 4:
    ~ kest_jealousy = kest_jealousy + 3
- else:
    { if lyra_affection - kest_affection >= 2:
        ~ kest_jealousy = kest_jealousy + 1
    }
}

{ if lyra_affection - orin_affection >= 4:
    ~ orin_jealousy = orin_jealousy + 2
- else:
    { if lyra_affection - orin_affection >= 2:
        ~ orin_jealousy = orin_jealousy + 1
    }
}

~ clamp_states()

== function spar_with_kest() => void ==
You witness Kest execute a perfect training pass.
~ kest_affection = kest_affection + 4
{ if kest_affection - lyra_affection >= 2:
    ~ lyra_jealousy = lyra_jealousy + 2
- else:
    { if kest_affection - lyra_affection >= 1:
        ~ lyra_jealousy = lyra_jealousy + 1
    }
}

{ if kest_affection - orin_affection >= 3:
    ~ orin_jealousy = orin_jealousy + 1
- else:
    { if kest_affection - orin_affection >= 1:
        ~ orin_jealousy = orin_jealousy + 1
    }
}

~ clamp_states()

== function walk_with_orin() => void ==
Orin confides about a mission route and offers quiet advice.
~ orin_affection = orin_affection + 2
{ if orin_affection - lyra_affection >= 2:
    ~ lyra_jealousy = lyra_jealousy + 2
- else:
    { if orin_affection - lyra_affection >= 1:
        ~ lyra_jealousy = lyra_jealousy + 1
    }
}

{ if orin_affection - kest_affection >= 2:
    ~ kest_jealousy = kest_jealousy + 2
- else:
    { if orin_affection - kest_affection >= 1:
        ~ kest_jealousy = kest_jealousy + 1
    }
}

~ clamp_states()

== function public_reassurance() => void ==
You thank the group together at dinner, so jealousy decays for everyone.
{ if lyra_jealousy > 0:
    ~ lyra_jealousy = lyra_jealousy - 1
}
{ if kest_jealousy > 0:
    ~ kest_jealousy = kest_jealousy - 1
}
{ if orin_jealousy > 0:
    ~ orin_jealousy = orin_jealousy - 1
}

~ lyra_affection = lyra_affection + 1
~ kest_affection = kest_affection + 1
~ orin_affection = orin_affection + 1
~ clamp_states()

== function final_consequence() => void ==
Final scene consequence:
{ final_consequence_text() }
Triangle readiness:
~ print_priority("affection")
~ print_priority("jealousy")

== function print_priority(mode: string) => void ==
{ if mode == "affection":
    Most favored: {top_affection_person()} ({top_value_affection()}).
- else:
    Most uneasy: {top_jealousy_person()} ({top_value_jealousy()}).
}

== function final_consequence_text() => string ==
~ temp peak: int = top_value_jealousy()
~ temp jealous_person: string = top_jealousy_person()
~ temp adored_person: string = top_affection_person()
{ if peak >= 8:
    { if jealous_person == "Lyra":
        ~ return "Lyra corners you after dinner and demands private explanation. A confrontation scene opens."
    - else:
        { if jealous_person == "Kest":
            ~ return "Kest refuses another order and breaks a teammate promise for a tense showdown."
        - else:
            ~ return "Orin takes the night off-route and ends the social scene early."
        }
    }
- else:
    { if top_value_affection() - peak >= 3:
        You are pulled into a quiet, affectionate scene with {adored_person}.
        The others keep distance but remain present.
    - else:
        A balanced social scene unfolds. The three remain tense but cooperative.
    }
}

== function clamp_states() => void ==
{ if lyra_affection < 0:
    ~ lyra_affection = 0
- else:
    { if lyra_affection > 12:
        ~ lyra_affection = 12
    }
}
{ if kest_affection < 0:
    ~ kest_affection = 0
- else:
    { if kest_affection > 12:
        ~ kest_affection = 12
    }
}
{ if orin_affection < 0:
    ~ orin_affection = 0
- else:
    { if orin_affection > 12:
        ~ orin_affection = 12
    }
}
{ if lyra_jealousy < 0:
    ~ lyra_jealousy = 0
- else:
    { if lyra_jealousy > 12:
        ~ lyra_jealousy = 12
    }
}
{ if kest_jealousy < 0:
    ~ kest_jealousy = 0
- else:
    { if kest_jealousy > 12:
        ~ kest_jealousy = 12
    }
}
{ if orin_jealousy < 0:
    ~ orin_jealousy = 0
- else:
    { if orin_jealousy > 12:
        ~ orin_jealousy = 12
    }
}

== function affection_tone(value: int) => string ==
{ if value >= 10:
    ~ return "loved"
- else:
    { if value >= 7:
        ~ return "warm"
    - else:
        { if value >= 5:
            ~ return "fond"
        - else:
            { if value >= 3:
                ~ return "steady"
            - else:
                ~ return "fragile"
            }
        }
    }
}

== function jealousy_tone(value: int) => string ==
{ if value >= 8:
    ~ return "volatile"
- else:
    { if value >= 5:
        ~ return "watchful"
    - else:
        { if value >= 3:
            ~ return "sensitive"
        - else:
            ~ return "calm"
        }
    }
}

== function top_affection_person() => string ==
~ temp leader: string = "Lyra"
~ temp best: int = lyra_affection
{ if kest_affection > best:
    ~ best = kest_affection
    ~ leader = "Kest"
- else:
    { if orin_affection > best:
        ~ best = orin_affection
        ~ leader = "Orin"
    }
}
~ return leader

== function top_value_affection() => int ==
~ temp best: int = lyra_affection
{ if kest_affection > best:
    ~ best = kest_affection
- else:
    { if orin_affection > best:
        ~ best = orin_affection
    }
}
~ return best

== function top_jealousy_person() => string ==
~ temp trouble: string = "Lyra"
~ temp highest: int = lyra_jealousy
{ if kest_jealousy > highest:
    ~ highest = kest_jealousy
    ~ trouble = "Kest"
- else:
    { if orin_jealousy > highest:
        ~ highest = orin_jealousy
        ~ trouble = "Orin"
    }
}
~ return trouble

== function top_value_jealousy() => int ==
~ temp highest: int = lyra_jealousy
{ if kest_jealousy > highest:
    ~ highest = kest_jealousy
- else:
    { if orin_jealousy > highest:
        ~ highest = orin_jealousy
    }
}
~ return highest
