=== module game ===

STRUCT IllegalTrade {
    good: string
    buy_cost: int
    sell_yield: int
    quantity: int
    heat: int
    informant_pressure: int
}

VAR cash: int = 48
VAR heat_level: int = 6
VAR informant_risk: int = 3
VAR informant_loyalty: int = 7
VAR heat_alerts: int = 0
VAR heat_penalties: int = 0
VAR raid_events: int = 0
VAR escape_succeeded: bool = false
VAR total_goods: int = 0

VAR black_market_runs: IllegalTrade[] = [
    %IllegalTrade{ good: "Arc Salt", buy_cost: 8, sell_yield: 11, quantity: 2, heat: 2, informant_pressure: 2 },
    %IllegalTrade{ good: "Clock Oil", buy_cost: 14, sell_yield: 12, quantity: 2, heat: 3, informant_pressure: 3 },
    %IllegalTrade{ good: "Cipher Grain", buy_cost: 9, sell_yield: 10, quantity: 3, heat: 5, informant_pressure: 5 },
    %IllegalTrade{ good: "Ghost Iron", buy_cost: 12, sell_yield: 7, quantity: 3, heat: 7, informant_pressure: 6 }
]

== main ==
Black-market processing started.
Cash on hand: {cash}
Heat marker: {heat_level}
Informant risk: {informant_risk}
~ run_book(0)
~ final_investigation_state()
-> DONE

== function run_book(index: int) => void ==
{ if index >= LEN(black_market_runs):
    All scheduled trades processed.
- else:
    { if !escape_succeeded:
        ~ temp item: IllegalTrade = black_market_runs[index]
        Trade {index + 1}: {item.good} x {item.quantity}
        ~ execute_trade(index, item)
        ~ run_book(index + 1)
    }
}

== function execute_trade(index: int, item: IllegalTrade) => void ==
~ temp buy_total: int = item.buy_cost * item.quantity
~ temp sell_total: int = item.sell_yield * item.quantity
~ cash = cash - buy_total
Cash spent {buy_total} on {item.good}.
~ temp prior_cash: int = cash
~ cash = cash + sell_total
~ temp net_change: int = cash - prior_cash
Trade profit for {item.good}: {net_change}
~ total_goods = total_goods + item.quantity
~ update_heat(index, item)

== function update_heat(index: int, item: IllegalTrade) => void ==
~ heat_level = heat_level + item.heat
~ informant_risk = informant_risk + item.informant_pressure
Heat is now {heat_level}, informant risk {informant_risk}.

~ temp alert_cost: int = item.informant_pressure - 1
{ if item.informant_pressure > 3:
    ~ heat_alerts = heat_alerts + 1
    Heat spike detected: informant pressure {item.informant_pressure}.
    ~ adjust_informant(alert_cost)
- else:
    Trade stays quiet for now.
}

{ if heat_level >= 18:
    ~ heat_alerts = heat_alerts + 1
    Heat threshold crossed.
    ~ trigger_police_response()
}

== function adjust_informant(alert_cost: int) => void ==
~ informant_loyalty = informant_loyalty - 1
~ heat_level = heat_level + alert_cost
~ heat_level = heat_level + 1
Informant now doubts discretion. Extra heat added.

== function trigger_police_response() => void ==
~ raid_events = raid_events + 1
~ temp bribe_cost: int = 12 + (heat_level / 2)
Raid {raid_events} response: heat {heat_level}, bribe quote {bribe_cost}.

{ if cash >= bribe_cost:
    ~ cash = cash - bribe_cost
    ~ heat_level = heat_level - 4
    Bribe paid. Route continues.
- else:
    ~ attempt_escape()
}

== function attempt_escape() => void ==
~ temp total_exit_cost: int = 7 + (informant_risk / 2)
~ temp can_escape: bool = cash >= total_exit_cost && total_goods >= 3
{ if can_escape:
    ~ cash = cash - total_exit_cost
    ~ heat_level = heat_level - 6
    ~ total_goods = total_goods - 3
    ~ escape_succeeded = true
    Escape route executed under cover, goods moved.
- else:
    Informant points to safehouse. Unable to run; operation ends in seizure.
    ~ heat_level = heat_level + 6
    ~ total_goods = 0
    ~ escape_succeeded = true
}

== function final_investigation_state() => void ==
Heat final: {heat_level}
Informant risk final: {informant_risk}
Informant loyalty final: {informant_loyalty}
Goods on hand: {total_goods}
Raid events: {raid_events}
Heat alerts: {heat_alerts}
Cash end: {cash}
{ if escape_succeeded:
    { if heat_level <= 12:
        Escape outcome: route escaped clean.
    - else:
        Escape outcome: escape with elevated heat.
    }
- else:
    { if heat_level >= 18:
        Outcome: raid risk still active, patrols remain.
    - else:
        Outcome: routine closing, no pursuit.
    }
}
