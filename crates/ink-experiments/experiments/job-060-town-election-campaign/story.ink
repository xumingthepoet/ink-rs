=== module game ===

VAR bloc_harbors_size: int = 84
VAR bloc_guilds_size: int = 102
VAR bloc_scholars_size: int = 63
VAR bloc_canal_size: int = 71
VAR bloc_north_size: int = 58

VAR support_harbors_aria: int = 48
VAR support_harbors_bran: int = 37
VAR support_guilds_aria: int = 39
VAR support_guilds_bran: int = 45
VAR support_scholars_aria: int = 44
VAR support_scholars_bran: int = 41
VAR support_canal_aria: int = 50
VAR support_canal_bran: int = 34
VAR support_north_aria: int = 35
VAR support_north_bran: int = 56

VAR endorsement_harbors: string = "none"
VAR endorsement_guilds: string = "none"
VAR endorsement_scholars: string = "none"
VAR endorsement_canal: string = "none"
VAR endorsement_north: string = "none"

VAR current_aria_votes: int = 0
VAR current_bran_votes: int = 0
VAR scandal_aria: int = 0
VAR scandal_bran: int = 0

== main ==
The city debates tomorrow's election.
~ recalc_votes()
~ print_bloc_snapshot("Opening week")
~ run_phase("Dock Workers Rally", 0, 6, 2, 1, "Aria")
~ run_phase("Guild Ledger Hearing", 1, -3, 4, -1, "Bran")
~ grant_endorsement(2, "Bran")
~ run_phase("University Policy Forum", 2, 4, -2, 2, "Aria")
~ expose_scandal("Broken Contracts", "Aria", 7)
~ grant_endorsement(4, "Bran")
~ run_phase("Canal Repair Promise", 3, 5, -1, 0, "Aria")
~ expose_scandal("Harbor Ledger Leak", "Aria", 5)
~ grant_endorsement(0, "Bran")
~ run_phase("Late Debate", 4, 1, 3, 1, "Bran")
~ expose_scandal("Unverified Rumor", "Bran", 2)
~ print_bloc_snapshot("Final week")
~ declare_result()
-> DONE

== function run_phase(label: string, bloc: int, aria_delta: int, bran_delta: int, neutral_delta: int, event_lead: string) => void ==
~ temp bloc_name: string = bloc_label(bloc)
{ if event_lead == "Aria":
    Aria hosts {label}, targeting {bloc_name}.
- else:
    Bran leads {label}, targeting {bloc_name}.
}
~ update_bloc_support(bloc, aria_delta, bran_delta, neutral_delta)
~ print_bloc_snapshot(label)
~ announce_end_of_round(label)

== function announce_end_of_round(label: string) => void ==
-- End of {label} --
Current bloc readout:
~ print_bloc_snapshot("current")
~ adjust_supplies("campaign outreach", 2, 1)

== function update_bloc_support(bloc: int, aria_delta: int, bran_delta: int, neutral_delta: int) => void ==
~ temp old_harbors_aria: int = support_harbors_aria
~ temp old_harbors_bran: int = support_harbors_bran
~ temp old_guilds_aria: int = support_guilds_aria
~ temp old_guilds_bran: int = support_guilds_bran
~ temp old_scholars_aria: int = support_scholars_aria
~ temp old_scholars_bran: int = support_scholars_bran
~ temp old_canal_aria: int = support_canal_aria
~ temp old_canal_bran: int = support_canal_bran
~ temp old_north_aria: int = support_north_aria
~ temp old_north_bran: int = support_north_bran
~ temp next_undecided: int = 0
~ temp temp_aria: int = 0
~ temp temp_bran: int = 0

{ if bloc == 0:
    { if neutral_delta < 0:
        ~ next_undecided = 0 - neutral_delta
    - else:
        ~ next_undecided = neutral_delta
    }
    ~ temp_aria = clamp_percent(old_harbors_aria + aria_delta)
    ~ temp_bran = clamp_percent(old_harbors_bran + bran_delta)
    ~ temp next_aria: int = temp_aria
    ~ temp next_bran: int = temp_bran
    ~ temp total_undecided: int = 100 - temp_aria - temp_bran
    { if total_undecided < 0:
        { if temp_aria >= temp_bran:
            ~ temp_aria = clamp_percent(temp_aria - 0)
        - else:
            ~ temp_bran = clamp_percent(temp_bran - 0)
        }
    }
    ~ support_harbors_aria = temp_aria
    ~ support_harbors_bran = temp_bran
- else:
    { if bloc == 1:
        { if neutral_delta < 0:
            ~ next_undecided = 0 - neutral_delta
        - else:
            ~ next_undecided = neutral_delta
        }
        ~ temp_aria = clamp_percent(old_guilds_aria + aria_delta)
        ~ temp_bran = clamp_percent(old_guilds_bran + bran_delta)
        ~ temp next_aria: int = temp_aria
        ~ temp next_bran: int = temp_bran
        ~ temp total_undecided: int = 100 - temp_aria - temp_bran
        { if total_undecided < 0:
            { if temp_aria >= temp_bran:
                ~ temp_aria = clamp_percent(temp_aria - 0)
            - else:
                ~ temp_bran = clamp_percent(temp_bran - 0)
            }
        }
        ~ support_guilds_aria = temp_aria
        ~ support_guilds_bran = temp_bran
    - else:
        { if bloc == 2:
            { if neutral_delta < 0:
                ~ next_undecided = 0 - neutral_delta
            - else:
                ~ next_undecided = neutral_delta
            }
            ~ temp_aria = clamp_percent(old_scholars_aria + aria_delta)
            ~ temp_bran = clamp_percent(old_scholars_bran + bran_delta)
            ~ temp next_aria: int = temp_aria
            ~ temp next_bran: int = temp_bran
            ~ temp total_undecided: int = 100 - temp_aria - temp_bran
            { if total_undecided < 0:
                { if temp_aria >= temp_bran:
                    ~ temp_aria = clamp_percent(temp_aria - 0)
                - else:
                    ~ temp_bran = clamp_percent(temp_bran - 0)
                }
            }
            ~ support_scholars_aria = temp_aria
            ~ support_scholars_bran = temp_bran
        - else:
            { if bloc == 3:
                { if neutral_delta < 0:
                    ~ next_undecided = 0 - neutral_delta
                - else:
                    ~ next_undecided = neutral_delta
                }
                ~ temp_aria = clamp_percent(old_canal_aria + aria_delta)
                ~ temp_bran = clamp_percent(old_canal_bran + bran_delta)
                ~ temp next_aria: int = temp_aria
                ~ temp next_bran: int = temp_bran
                ~ temp total_undecided: int = 100 - temp_aria - temp_bran
                { if total_undecided < 0:
                    { if temp_aria >= temp_bran:
                        ~ temp_aria = clamp_percent(temp_aria - 0)
                    - else:
                        ~ temp_bran = clamp_percent(temp_bran - 0)
                    }
                }
                ~ support_canal_aria = temp_aria
                ~ support_canal_bran = temp_bran
            - else:
                ~ temp_aria = clamp_percent(old_north_aria + aria_delta)
                ~ temp_bran = clamp_percent(old_north_bran + bran_delta)
                ~ temp next_aria: int = temp_aria
                ~ temp next_bran: int = temp_bran
                ~ temp total_undecided: int = 100 - temp_aria - temp_bran
                { if total_undecided < 0:
                    { if temp_aria >= temp_bran:
                        ~ temp_aria = clamp_percent(temp_aria - 0)
                    - else:
                        ~ temp_bran = clamp_percent(temp_bran - 0)
                    }
                }
                ~ support_north_aria = temp_aria
                ~ support_north_bran = temp_bran
            }
        }
    }
}

~ recalc_votes()

== function bloc_label(bloc: int) => string ==
{ if bloc == 0:
    ~ return "Harbors"
- else:
    { if bloc == 1:
        ~ return "Guilds"
    - else:
        { if bloc == 2:
            ~ return "Scholars"
        - else:
            { if bloc == 3:
                ~ return "Canal Villages"
            - else:
                ~ return "North Gate"
            }
        }
    }
}

== function grant_endorsement(bloc: int, candidate: string) => void ==
~ temp bloc_name: string = bloc_label(bloc)
~ temp existing: string = "none"
~ temp next_aria: int = 0
~ temp next_bran: int = 0

{ if bloc == 0:
    ~ existing = endorsement_harbors
- else:
    { if bloc == 1:
        ~ existing = endorsement_guilds
    - else:
        { if bloc == 2:
            ~ existing = endorsement_scholars
        - else:
            { if bloc == 3:
                ~ existing = endorsement_canal
            - else:
                ~ existing = endorsement_north
            }
        }
    }
}

{ if existing == candidate:
    { if bloc_name == "Scholars":
        The Scholar Hall confirms the same endorsement stays active.
    - else:
        { if bloc_name == "North Gate":
            The gate watch keeps its existing trust.
        - else:
            { if bloc_name == "Harbors":
                Harbor captains keep their current pledge.
            - else:
                The bloc keeps its current alignment unchanged.
            }
        }
    }
    ~ return
- else:
    { if existing == "Aria":
        ~ next_aria = -1
    - else:
        { if existing == "Bran":
            ~ next_bran = -1
        }
    }
}

{ if candidate == "Aria":
    ~ endorsement_banner("Aria", bloc)
    ~ next_aria = next_aria + 4
    { if bloc == 0:
        ~ support_harbors_aria = support_harbors_aria + next_aria
    - else:
        { if bloc == 1:
            ~ support_guilds_aria = support_guilds_aria + next_aria
        - else:
            { if bloc == 2:
                ~ support_scholars_aria = support_scholars_aria + next_aria
            - else:
                { if bloc == 3:
                    ~ support_canal_aria = support_canal_aria + next_aria
                - else:
                    ~ support_north_aria = support_north_aria + next_aria
                }
            }
        }
    }
- else:
    ~ endorsement_banner("Bran", bloc)
    ~ next_bran = next_bran + 4
    { if bloc == 0:
        ~ support_harbors_bran = support_harbors_bran + next_bran
    - else:
        { if bloc == 1:
            ~ support_guilds_bran = support_guilds_bran + next_bran
        - else:
            { if bloc == 2:
                ~ support_scholars_bran = support_scholars_bran + next_bran
            - else:
                { if bloc == 3:
                    ~ support_canal_bran = support_canal_bran + next_bran
                - else:
                    ~ support_north_bran = support_north_bran + next_bran
                }
            }
        }
    }
}

{ if bloc == 0:
    ~ support_harbors_aria = clamp_percent(support_harbors_aria)
    ~ support_harbors_bran = clamp_percent(support_harbors_bran)
    ~ endorsement_harbors = candidate
    ~ print_support_pair(0)
- else:
    { if bloc == 1:
        ~ support_guilds_aria = clamp_percent(support_guilds_aria)
        ~ support_guilds_bran = clamp_percent(support_guilds_bran)
        ~ endorsement_guilds = candidate
        ~ print_support_pair(1)
    - else:
        { if bloc == 2:
            ~ support_scholars_aria = clamp_percent(support_scholars_aria)
            ~ support_scholars_bran = clamp_percent(support_scholars_bran)
            ~ endorsement_scholars = candidate
            ~ print_support_pair(2)
        - else:
            { if bloc == 3:
                ~ support_canal_aria = clamp_percent(support_canal_aria)
                ~ support_canal_bran = clamp_percent(support_canal_bran)
                ~ endorsement_canal = candidate
                ~ print_support_pair(3)
            - else:
                ~ support_north_aria = clamp_percent(support_north_aria)
                ~ support_north_bran = clamp_percent(support_north_bran)
                ~ endorsement_north = candidate
                ~ print_support_pair(4)
            }
        }
    }
}

~ recalc_votes()

== function endorsement_banner(candidate: string, bloc: int) => void ==
{ if candidate == "Aria":
    { if bloc == 2:
        The Astronomers endorse Aria's civic scholarship plan.
    - else:
        { if bloc == 0:
            A dock charter group endorses Aria.
        - else:
            { if bloc == 4:
                A frontier officer publicly supports Aria.
            - else:
                A civic guild endorses Aria.
            }
        }
    }
- else:
    { if bloc == 4:
        The North Gate patrol block endorses Bran.
    - else:
        { if bloc == 0:
            The harbor union chamber endorses Bran.
        - else:
            A major bloc leans publicly toward Bran.
        }
    }
}

== function expose_scandal(name: string, target: string, severity: int) => void ==
Public records release {name}, targeting {target}.
{ if target == "Aria":
    ~ scandal_aria = scandal_aria + 1
    ~ shift_due_to_scandal("Aria", severity, 0)
- else:
    ~ scandal_bran = scandal_bran + 1
    ~ shift_due_to_scandal("Bran", severity, 0)
}

== function shift_due_to_scandal(target: string, severity: int, bloc: int) => void ==
{ if bloc == 0:
    { if target == "Aria":
        ~ support_harbors_aria = clamp_percent(support_harbors_aria - severity)
        ~ support_harbors_bran = clamp_percent(support_harbors_bran + 1)
    - else:
        ~ support_harbors_bran = clamp_percent(support_harbors_bran - severity)
        ~ support_harbors_aria = clamp_percent(support_harbors_aria + 1)
    }
    ~ normalize_support_pair(0)
    ~ shift_due_to_scandal(target, severity, 1)
- else:
    { if bloc == 1:
        { if target == "Aria":
            ~ support_guilds_aria = clamp_percent(support_guilds_aria - severity)
            ~ support_guilds_bran = clamp_percent(support_guilds_bran + 1)
        - else:
            ~ support_guilds_bran = clamp_percent(support_guilds_bran - severity)
            ~ support_guilds_aria = clamp_percent(support_guilds_aria + 1)
        }
        ~ normalize_support_pair(1)
        ~ shift_due_to_scandal(target, severity, 2)
    - else:
        { if bloc == 2:
            { if target == "Aria":
                ~ support_scholars_aria = clamp_percent(support_scholars_aria - severity)
                ~ support_scholars_bran = clamp_percent(support_scholars_bran + 1)
            - else:
                ~ support_scholars_bran = clamp_percent(support_scholars_bran - severity)
                ~ support_scholars_aria = clamp_percent(support_scholars_aria + 1)
            }
            ~ normalize_support_pair(2)
            ~ shift_due_to_scandal(target, severity, 3)
        - else:
            { if bloc == 3:
                { if target == "Aria":
                    ~ support_canal_aria = clamp_percent(support_canal_aria - severity)
                    ~ support_canal_bran = clamp_percent(support_canal_bran + 1)
                - else:
                    ~ support_canal_bran = clamp_percent(support_canal_bran - severity)
                    ~ support_canal_aria = clamp_percent(support_canal_aria + 1)
                }
                ~ normalize_support_pair(3)
                ~ shift_due_to_scandal(target, severity, 4)
            - else:
                { if bloc == 4:
                    { if target == "Aria":
                        ~ support_north_aria = clamp_percent(support_north_aria - severity)
                        ~ support_north_bran = clamp_percent(support_north_bran + 1)
                    - else:
                        ~ support_north_bran = clamp_percent(support_north_bran - severity)
                        ~ support_north_aria = clamp_percent(support_north_aria + 1)
                    }
                    ~ normalize_support_pair(4)
                }
            }
        }
    }
}

~ recalc_votes()

== function print_bloc_snapshot(label: string) => void ==
Election bloc snapshot: {label}
~ print_support_pair(0)
~ print_support_pair(1)
~ print_support_pair(2)
~ print_support_pair(3)
~ print_support_pair(4)

== function print_support_pair(bloc: int) => void ==
{ if bloc == 0:
    ~ temp aria: int = support_harbors_aria
    ~ temp bran: int = support_harbors_bran
    ~ temp undecided: int = 100 - aria - bran
    ~ print_support_line("Harbors", aria, bran, undecided)
- else:
    { if bloc == 1:
        ~ temp aria: int = support_guilds_aria
        ~ temp bran: int = support_guilds_bran
        ~ temp undecided: int = 100 - aria - bran
        ~ print_support_line("Guilds", aria, bran, undecided)
    - else:
        { if bloc == 2:
            ~ temp aria: int = support_scholars_aria
            ~ temp bran: int = support_scholars_bran
            ~ temp undecided: int = 100 - aria - bran
            ~ print_support_line("Scholars", aria, bran, undecided)
        - else:
            { if bloc == 3:
                ~ temp aria: int = support_canal_aria
                ~ temp bran: int = support_canal_bran
                ~ temp undecided: int = 100 - aria - bran
                ~ print_support_line("Canal Villages", aria, bran, undecided)
            - else:
                ~ temp aria: int = support_north_aria
                ~ temp bran: int = support_north_bran
                ~ temp undecided: int = 100 - aria - bran
                ~ print_support_line("North Gate", aria, bran, undecided)
            }
        }
    }
}

== function print_support_line(label: string, aria: int, bran: int, undecided: int) => void ==
~ temp adjusted: int = clamp_percent(undecided)
Bloc {label}: Aria {aria} percent / Bran {bran} percent / Undecided {adjusted} percent

== function normalize_support_pair(bloc: int) => void ==
{ if bloc == 0:
    ~ temp aria: int = support_harbors_aria
    ~ temp bran: int = support_harbors_bran
    ~ temp undecided: int = 100 - aria - bran
    { if undecided < 0:
        { if aria >= bran:
            ~ support_harbors_aria = 100 - bran
        - else:
            ~ support_harbors_bran = 100 - aria
        }
    }
- else:
    { if bloc == 1:
        ~ temp aria: int = support_guilds_aria
        ~ temp bran: int = support_guilds_bran
        ~ temp undecided: int = 100 - aria - bran
        { if undecided < 0:
            { if aria >= bran:
                ~ support_guilds_aria = 100 - bran
            - else:
                ~ support_guilds_bran = 100 - aria
            }
        }
    - else:
        { if bloc == 2:
            ~ temp aria: int = support_scholars_aria
            ~ temp bran: int = support_scholars_bran
            ~ temp undecided: int = 100 - aria - bran
            { if undecided < 0:
                { if aria >= bran:
                    ~ support_scholars_aria = 100 - bran
                - else:
                    ~ support_scholars_bran = 100 - aria
                }
            }
        - else:
            { if bloc == 3:
                ~ temp aria: int = support_canal_aria
                ~ temp bran: int = support_canal_bran
                ~ temp undecided: int = 100 - aria - bran
                { if undecided < 0:
                    { if aria >= bran:
                        ~ support_canal_aria = 100 - bran
                    - else:
                        ~ support_canal_bran = 100 - aria
                    }
                }
            - else:
                ~ temp aria: int = support_north_aria
                ~ temp bran: int = support_north_bran
                ~ temp undecided: int = 100 - aria - bran
                { if undecided < 0:
                    { if aria >= bran:
                        ~ support_north_aria = 100 - bran
                    - else:
                        ~ support_north_bran = 100 - aria
                    }
                }
            }
        }
    }
}

== function clamp_percent(value: int) => int ==
{ if value < 0:
    ~ return 0
- else:
    { if value > 100:
        ~ return 100
    - else:
        ~ return value
    }
}

== function adjust_supplies(label: string, aria_delta: int, bran_delta: int) => void ==
Budget swings for {label}:
Aria campaign budget: {aria_delta}
Bran campaign budget: {bran_delta}

== function declare_result() => void ==
Election totals:
Aria votes: {current_aria_votes}
Bran votes: {current_bran_votes}

{ if current_aria_votes > current_bran_votes:
    Aria wins tonight's election.
    Margin: {current_aria_votes - current_bran_votes}
    Coalition path stays open with a mandate.
- else:
    { if current_bran_votes > current_aria_votes:
        Bran wins tonight's election.
        Margin: {current_bran_votes - current_aria_votes}
        Coalition math now favors a hard-line budget.
    - else:
        The result is tied.
        A runoff and one late-night convention are now required.
    }
}

~ report_turnout()
~ compare_scandals()

== function recalc_votes() => void ==
~ temp harbors_aria: int = (bloc_harbors_size * support_harbors_aria) / 100
~ temp harbors_bran: int = (bloc_harbors_size * support_harbors_bran) / 100
~ temp guilds_aria: int = (bloc_guilds_size * support_guilds_aria) / 100
~ temp guilds_bran: int = (bloc_guilds_size * support_guilds_bran) / 100
~ temp scholars_aria: int = (bloc_scholars_size * support_scholars_aria) / 100
~ temp scholars_bran: int = (bloc_scholars_size * support_scholars_bran) / 100
~ temp canal_aria: int = (bloc_canal_size * support_canal_aria) / 100
~ temp canal_bran: int = (bloc_canal_size * support_canal_bran) / 100
~ temp north_aria: int = (bloc_north_size * support_north_aria) / 100
~ temp north_bran: int = (bloc_north_size * support_north_bran) / 100

~ current_aria_votes = harbors_aria + guilds_aria + scholars_aria + canal_aria + north_aria
~ current_bran_votes = harbors_bran + guilds_bran + scholars_bran + canal_bran + north_bran

== function report_turnout() => void ==
Turnout capacity snapshot:
Bloc Harbors: {bloc_harbors_size} voters
Bloc Guilds: {bloc_guilds_size} voters
Bloc Scholars: {bloc_scholars_size} voters
Bloc Canal Villages: {bloc_canal_size} voters
Bloc North Gate: {bloc_north_size} voters
~ report_undecided_total()

== function report_undecided_total() => void ==
~ temp harbors_undecided: int = 100 - support_harbors_aria - support_harbors_bran
~ temp guilds_undecided: int = 100 - support_guilds_aria - support_guilds_bran
~ temp scholars_undecided: int = 100 - support_scholars_aria - support_scholars_bran
~ temp canal_undecided: int = 100 - support_canal_aria - support_canal_bran
~ temp north_undecided: int = 100 - support_north_aria - support_north_bran
~ temp total_undecided: int = (bloc_harbors_size * harbors_undecided) / 100 + (guilds_undecided * bloc_guilds_size) / 100 + (scholars_undecided * bloc_scholars_size) / 100 + (canal_undecided * bloc_canal_size) / 100 + (north_undecided * bloc_north_size) / 100
Undecided total: {clamp_percent(total_undecided)}

== function compare_scandals() => void ==
Scandal heat index:
Aria: {scandal_aria}
Bran: {scandal_bran}
