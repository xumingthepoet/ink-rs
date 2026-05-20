=== module game ===

STRUCT PriorFlag {
    key: string
    weight: int
    positive_if_true: bool
    consequence_true: string
    consequence_false: string
}

VAR prior_flags: PriorFlag[] = [
    %PriorFlag{
        key: "reactor_stable",
        weight: 8,
        positive_if_true: true,
        consequence_true: "Reactor remains under local control.",
        consequence_false: "Reactor remains unstable and under emergency lock."
    },
    %PriorFlag{
        key: "relay_network_secure",
        weight: 7,
        positive_if_true: true,
        consequence_true: "Long-range messaging remains trusted.",
        consequence_false: "Long-range messaging requires escort encryption."
    },
    %PriorFlag{
        key: "colony_fed",
        weight: 6,
        positive_if_true: true,
        consequence_true: "Food lines stabilize across all sectors.",
        consequence_false: "Food rationing remains severe in outer districts."
    },
    %PriorFlag{
        key: "treaty_signed",
        weight: 5,
        positive_if_true: true,
        consequence_true: "Frontier treaty remains enforceable.",
        consequence_false: "Border terms collapse into local standoffs."
    },
    %PriorFlag{
        key: "archive_recovered",
        weight: 4,
        positive_if_true: true,
        consequence_true: "Recovered archives improve planning forecasts.",
        consequence_false: "Forecasting remains incomplete without archives."
    },
    %PriorFlag{
        key: "captain_trusted",
        weight: 5,
        positive_if_true: true,
        consequence_true: "Command authority remains publicly accepted.",
        consequence_false: "Command authority fractures across officer groups."
    },
    %PriorFlag{
        key: "evacuation_prepared",
        weight: 3,
        positive_if_true: true,
        consequence_true: "Emergency corridors stay clear for civilians.",
        consequence_false: "Evacuation routes remain partially blocked."
    },
    %PriorFlag{
        key: "mutiny_prevented",
        weight: 6,
        positive_if_true: true,
        consequence_true: "Internal order holds after the final operation.",
        consequence_false: "Mutiny damage keeps command posts divided."
    }
]

VAR flag_state: Dict<string, bool> = %{
    "reactor_stable": true,
    "relay_network_secure": false,
    "colony_fed": true,
    "treaty_signed": true,
    "archive_recovered": false,
    "captain_trusted": true,
    "evacuation_prepared": true,
    "mutiny_prevented": true
}

VAR chapter_scores: Dict<string, int> = %{
    "reputation": 74,
    "resources": 61,
    "security": 48,
    "science": 55,
    "unity": 72,
    "debt": 18,
    "casualties": 12
}

VAR ending_points: int = 0
VAR positive_flags: int = 0
VAR negative_flags: int = 0
VAR ending_classification: string = ""
VAR ending_title: string = ""

== main ==
Endgame state recap aggregation.
~ evaluate_prior_flags(0)
~ apply_score_adjustments()
~ determine_ending()
~ print_final_recap()
-> DONE

== function evaluate_prior_flags(index: int) => void ==
{ if index >= LEN(prior_flags):
    Prior flag aggregation complete.
- else:
    ~ temp flag: PriorFlag = prior_flags[index]
    ~ temp state: bool = flag_state[flag.key]
    Flag {index + 1}: {flag.key} = {state}

    { if state == flag.positive_if_true:
        ~ ending_points = ending_points + flag.weight
        ~ positive_flags = positive_flags + 1
        {flag.consequence_true}
    - else:
        ~ ending_points = ending_points - flag.weight
        ~ negative_flags = negative_flags + 1
        {flag.consequence_false}
    }

    ~ evaluate_prior_flags(index + 1)
}

== function apply_score_adjustments() => void ==
Chapter score adjustments:

~ apply_reputation_adjustment()
~ apply_resources_adjustment()
~ apply_security_adjustment()
~ apply_science_adjustment()
~ apply_unity_adjustment()
~ apply_debt_adjustment()
~ apply_casualty_adjustment()

== function apply_reputation_adjustment() => void ==
~ temp value: int = chapter_scores["reputation"]
{ if value >= 70:
    ~ ending_points = ending_points + 12
    Reputation {value}: +12
- else:
    { if value >= 55:
        ~ ending_points = ending_points + 6
        Reputation {value}: +6
    - else:
        ~ ending_points = ending_points - 5
        Reputation {value}: -5
    }
}

== function apply_resources_adjustment() => void ==
~ temp value: int = chapter_scores["resources"]
{ if value >= 60:
    ~ ending_points = ending_points + 10
    Resources {value}: +10
- else:
    { if value >= 45:
        ~ ending_points = ending_points + 4
        Resources {value}: +4
    - else:
        ~ ending_points = ending_points - 6
        Resources {value}: -6
    }
}

== function apply_security_adjustment() => void ==
~ temp value: int = chapter_scores["security"]
{ if value >= 60:
    ~ ending_points = ending_points + 7
    Security {value}: +7
- else:
    { if value >= 50:
        ~ ending_points = ending_points + 2
        Security {value}: +2
    - else:
        ~ ending_points = ending_points - 8
        Security {value}: -8
    }
}

== function apply_science_adjustment() => void ==
~ temp value: int = chapter_scores["science"]
{ if value >= 50:
    ~ ending_points = ending_points + 6
    Science {value}: +6
- else:
    { if value >= 35:
        ~ ending_points = ending_points + 1
        Science {value}: +1
    - else:
        ~ ending_points = ending_points - 4
        Science {value}: -4
    }
}

== function apply_unity_adjustment() => void ==
~ temp value: int = chapter_scores["unity"]
{ if value >= 70:
    ~ ending_points = ending_points + 8
    Unity {value}: +8
- else:
    { if value >= 55:
        ~ ending_points = ending_points + 3
        Unity {value}: +3
    - else:
        ~ ending_points = ending_points - 7
        Unity {value}: -7
    }
}

== function apply_debt_adjustment() => void ==
~ temp value: int = chapter_scores["debt"]
{ if value <= 25:
    ~ ending_points = ending_points + 2
    Debt {value}: +2
- else:
    { if value <= 40:
        ~ ending_points = ending_points - 2
        Debt {value}: -2
    - else:
        ~ ending_points = ending_points - 6
        Debt {value}: -6
    }
}

== function apply_casualty_adjustment() => void ==
~ temp value: int = chapter_scores["casualties"]
{ if value <= 8:
    ~ ending_points = ending_points + 1
    Casualties {value}: +1
- else:
    { if value <= 15:
        ~ ending_points = ending_points - 5
        Casualties {value}: -5
    - else:
        ~ ending_points = ending_points - 10
        Casualties {value}: -10
    }
}

== function determine_ending() => void ==
{ if ending_points >= 45:
    { if negative_flags <= 2:
        ~ ending_classification = "Radiant Recovery"
        ~ ending_title = "The Long Dawn"
    - else:
        ~ ending_classification = "Hard-Won Continuance"
        ~ ending_title = "Embers That Hold"
    }
- else:
    { if ending_points >= 20:
        ~ ending_classification = "Hard-Won Continuance"
        ~ ending_title = "Embers That Hold"
    - else:
        ~ ending_classification = "Fractured Aftermath"
        ~ ending_title = "Ashes Under Watch"
    }
}

== function print_final_recap() => void ==
Final ending classification: {ending_classification}
Ending title: {ending_title}
Ending points: {ending_points}
Positive flags: {positive_flags}
Negative flags: {negative_flags}

Recap consequences:
{ if flag_state["relay_network_secure"]:
    Relay routes keep automatic trust channels.
- else:
    Relay routes require manual encryption escorts.
}

{ if flag_state["archive_recovered"]:
    Science board publishes full archive projections.
- else:
    Science board proceeds with incomplete projections.
}

{ if chapter_scores["security"] >= 50:
    Border posts return to normal staffing.
- else:
    Border posts keep expanded watch rotations.
}

{ if chapter_scores["casualties"] > 10:
    Two districts continue under recovery leave.
- else:
    Civil districts reopen full shift schedules.
}

{ if ending_classification == "Radiant Recovery":
    Coalition leadership confirms a stable reconstruction term.
- else:
    { if ending_classification == "Hard-Won Continuance":
        Coalition leadership authorizes emergency oversight for one more season.
    - else:
        Coalition leadership imposes crisis command restructuring.
    }
}
