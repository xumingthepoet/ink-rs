=== module game ===

STRUCT ControlPoint {
    name: string
    blue_pressure: int
    red_pressure: int
    owner: string
    contested: bool
    last_event: string
}

VAR control_points: ControlPoint[] = [
    %ControlPoint{
        name: "Ridgeline Beacon",
        blue_pressure: 0,
        red_pressure: 0,
        owner: "neutral",
        contested: false,
        last_event: "No activity"
    },
    %ControlPoint{
        name: "Black Forge",
        blue_pressure: 0,
        red_pressure: 0,
        owner: "neutral",
        contested: false,
        last_event: "No activity"
    },
    %ControlPoint{
        name: "River Bastion",
        blue_pressure: 0,
        red_pressure: 0,
        owner: "neutral",
        contested: false,
        last_event: "No activity"
    }
]

VAR blue_total: int = 0
VAR red_total: int = 0
VAR round_blue: int = 0
VAR round_red: int = 0

== main ==
Battlefield control simulation.
~ print_control_points("Initial state")
~ simulate_round(1)
~ print_match_tally()
-> DONE

== function simulate_round(round: int) => void ==
{ if round <= 4:
    ~ apply_round_events(round)
    ~ resolve_ownership(0)
    ~ clear_round_totals()
    ~ print_control_points("Round {round} after events")
    ~ round_scores(round)
    ~ print_match_tally()
    ~ simulate_round(round + 1)
- else:
    Combat window closes.
}

== function apply_round_events(round: int) => void ==
~ clear_pressure_comments()
{ if round == 1:
    ~ apply_pressure(0, 2, 0, "Blue scouts secure Ridgeline")
    ~ apply_pressure(1, 0, 1, "Red raider slips into Black Forge")
    ~ apply_pressure(2, 0, 0, "No changes at River Bastion")
- else:
    { if round == 2:
        ~ apply_pressure(0, 0, 2, "Red counter-scouts contest Ridgeline")
        ~ apply_pressure(1, 2, 0, "Blue patrols reach Black Forge")
        ~ apply_pressure(2, 0, 3, "Red artillery opens fire on River Bastion")
    - else:
        { if round == 3:
            ~ apply_pressure(0, 2, 0, "Blue reinforcement pushes into Ridgeline")
            ~ apply_pressure(1, 1, 1, "Equal skirmishes at Black Forge")
            ~ apply_pressure(2, 1, 0, "Blue engineers retake a foothold")
        - else:
            ~ apply_pressure(0, 0, 1, "Red night sweep at Ridgeline")
            ~ apply_pressure(1, 0, 2, "Red reserve fortifies Black Forge")
            ~ apply_pressure(2, 2, 0, "Blue reserve lifts River Bastion")
        }
    }
}

== function apply_pressure(index: int, blue_delta: int, red_delta: int, event: string) => void ==
~ temp point: ControlPoint = control_points[index]
~ control_points[index].blue_pressure = point.blue_pressure + blue_delta
~ control_points[index].red_pressure = point.red_pressure + red_delta
~ control_points[index].last_event = event
[{point.name}] {event} (+{blue_delta}/+{red_delta}).

== function clear_pressure_comments() => void ==
~ control_points[0].last_event = "No additional events"
~ control_points[1].last_event = "No additional events"
~ control_points[2].last_event = "No additional events"

== function resolve_ownership(index: int) => void ==
{ if index >= LEN(control_points):
    ~ return
- else:
    ~ temp point: ControlPoint = control_points[index]
    { if point.blue_pressure > point.red_pressure:
        ~ control_points[index].owner = "Blue"
        ~ control_points[index].contested = false
    - else:
        { if point.red_pressure > point.blue_pressure:
            ~ control_points[index].owner = "Red"
            ~ control_points[index].contested = false
        - else:
            { if point.blue_pressure == 0 && point.red_pressure == 0:
                ~ control_points[index].owner = "Neutral"
                ~ control_points[index].contested = false
            - else:
                ~ control_points[index].owner = "Contested"
                ~ control_points[index].contested = true
            }
        }
    }
    ~ resolve_ownership(index + 1)
}

== function clear_round_totals() => void ==
~ round_blue = 0
~ round_red = 0

== function round_scores(round: int) => void ==
~ accumulate_scores(0)
~ blue_total = blue_total + round_blue
~ red_total = red_total + round_red
~ print_round_summary(round)

== function accumulate_scores(index: int) => void ==
{ if index < LEN(control_points):
    ~ temp point: ControlPoint = control_points[index]
    { if point.owner == "Blue":
        ~ round_blue = round_blue + 1
    - else:
        { if point.owner == "Red":
            ~ round_red = round_red + 1
        }
    }
    ~ accumulate_scores(index + 1)
}

== function print_control_points(label: string) => void ==
-- {label} --
~ print_control_point(0)

== function print_control_point(index: int) => void ==
{ if index < LEN(control_points):
    ~ temp point: ControlPoint = control_points[index]
    { if point.contested:
        {point.name}: {point.blue_pressure} - {point.red_pressure}, {point.owner}.
    - else:
        { if point.owner == "Blue":
            {point.name}: {point.blue_pressure} - {point.red_pressure}, controlled by Blue.
        - else:
            { if point.owner == "Red":
                {point.name}: {point.blue_pressure} - {point.red_pressure}, controlled by Red.
            - else:
                {point.name}: {point.blue_pressure} - {point.red_pressure}, remains neutral.
            }
        }
    }
    Last action: {point.last_event}
    ~ print_control_point(index + 1)
}

== function print_round_summary(round: int) => void ==
Round {round} scoring:
Blue controlled {round_blue} points.
Red controlled {round_red} points.
Running total: Blue {blue_total}, Red {red_total}.
{if_leading(blue_total, red_total)}

== function if_leading(blue: int, red: int) => void ==
{ if blue > red:
    Blue leads by {blue - red}.
- else:
    { if red > blue:
        Red leads by {red - blue}.
    - else:
        Totals are tied.
    }
}

== function print_match_tally() => void ==
-- Match tally --
Blue: {blue_total}
Red: {red_total}
