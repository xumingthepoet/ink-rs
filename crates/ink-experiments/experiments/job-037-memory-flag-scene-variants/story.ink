=== module game ===

VAR ally_met: bool = false
VAR forged_pass: bool = false
VAR alarm_ringing: bool = false
VAR reputation: int = 0
VAR suspicion: int = 0
VAR supplies: int = 12

== main ==
Memory flags steering scene variants.
~ present_gate_scene("First attempt at the city gate")
~ issue_forged_pass()
~ present_gate_scene("Second attempt after scouting")
~ meet_ally()
~ present_gate_scene("Third attempt after the ally arrives")
~ trigger_alarm()
~ present_gate_scene("Fourth attempt after alarms")
~ print_casualty_rollup()
-> DONE

== function present_gate_scene(label: string) => void ==
-- {label} --
~ temp code: int = scene_code()
{ scene_dialogue(code) }
~ apply_consequence(code)
-- Current flags --
Ally met: {ally_met}.
Forged pass: {forged_pass}.
Alarm raised: {alarm_ringing}.
Reputation: {reputation}. Suspicion: {suspicion}. Supplies: {supplies}.

== function scene_code() => int ==
~ temp code: int = 0
{ if ally_met:
    ~ code = code + 4
}
{ if forged_pass:
    ~ code = code + 2
}
{ if alarm_ringing:
    ~ code = code + 1
}
~ return code

== function scene_dialogue(code: int) => void ==
{ if code == 0:
    The guards keep their halberds close. No name, no seal, no witness.
- else:
    { if code == 1:
        Only bells ring at the gate. The watch is alert and no one will be believed.
    - else:
        { if code == 2:
            A forged pass helps, but the ward has already marked the seal as suspicious.
        - else:
            { if code == 3:
                The false pass is useful, yet the alarm forces a second inspection.
            - else:
                { if code == 4:
                    A familiar face appears, but the gate captain still asks for proof.
                - else:
                    { if code == 5:
                        An ally and a loud alarm collide, so you answer under scrutiny and tension.
                    - else:
                        { if code == 6:
                            Your ally vouches for you and the forged pass slides through a narrow watch.
                        - else:
                            Your ally, forged pass, and raised alarm create a chaotic but usable escape.
                        }
                    }
                }
            }
        }
    }
}

== function apply_consequence(code: int) => void ==
{ if code == 0:
    ~ reputation = reputation - 1
    ~ suspicion = suspicion + 2
    ~ supplies = supplies - 2
- else:
    { if code == 1:
        ~ reputation = reputation - 2
        ~ suspicion = suspicion + 3
        ~ supplies = supplies - 2
    - else:
        { if code == 2:
            ~ reputation = reputation + 1
            ~ suspicion = suspicion - 1
            ~ supplies = supplies - 1
        - else:
            { if code == 3:
                ~ reputation = reputation + 0
                ~ suspicion = suspicion + 1
                ~ supplies = supplies - 3
            - else:
                { if code == 4:
                    ~ reputation = reputation + 1
                    ~ suspicion = suspicion + 1
                    ~ supplies = supplies - 1
                - else:
                    { if code == 5:
                        ~ reputation = reputation + 1
                        ~ suspicion = suspicion + 2
                        ~ supplies = supplies - 2
                    - else:
                        { if code == 6:
                            ~ reputation = reputation + 3
                            ~ suspicion = suspicion - 1
                            ~ supplies = supplies - 1
                        - else:
                            ~ reputation = reputation + 2
                            ~ suspicion = suspicion + 2
                            ~ supplies = supplies - 3
                        }
                    }
                }
            }
        }
    }
}
Reputation shift and consequence applied.

== function issue_forged_pass() => void ==
~ forged_pass = true
You find a forged civic seal.

== function meet_ally() => void ==
~ ally_met = true
An old quartermaster recognizes you through the smoke.

== function trigger_alarm() => void ==
~ alarm_ringing = true
The ward bell begins ringing across the outer lane.

== function print_casualty_rollup() => void ==
-- Final state --
Reputation: {reputation}
Suspicion: {suspicion}
Supplies: {supplies}
Result interpretation:
{if reputation > suspicion:
    You pass through with manageable scrutiny.
- else:
    If repetition continues, the city watch will detain you.
}
