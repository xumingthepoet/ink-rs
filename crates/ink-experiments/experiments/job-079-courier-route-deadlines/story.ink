=== module game ===

STRUCT Delivery {
    id: int
    cargo: string
    travel_time: int
    deadline: int
    reward: int
    penalty_per_minute: int
}

VAR deliveries: Delivery[] = [
    %Delivery{id: 501, cargo: "Clockwork lens", travel_time: 8, deadline: 18, reward: 120, penalty_per_minute: 9},
    %Delivery{id: 502, cargo: "Medical herbs", travel_time: 5, deadline: 11, reward: 90, penalty_per_minute: 12},
    %Delivery{id: 503, cargo: "Ceremonial seals", travel_time: 7, deadline: 16, reward: 110, penalty_per_minute: 10},
    %Delivery{id: 504, cargo: "Encrypted letters", travel_time: 10, deadline: 30, reward: 150, penalty_per_minute: 7},
    %Delivery{id: 505, cargo: "Refined ore", travel_time: 9, deadline: 24, reward: 130, penalty_per_minute: 8}
]

VAR delay_plan: int[] = [4, 6, 2, 5, 1]
VAR delay_reason: string[] = [
    "storm at western gate",
    "bridge inspection",
    "street festival traffic",
    "river ferry delay",
    "night watch checkpoint"
]

VAR current_time: int = 0
VAR earned: int = 0
VAR penalties: int = 0
VAR gross_reward: int = 0
VAR completed_on_time: int = 0
VAR late_deliveries: int = 0
VAR failed_deliveries: int = 0
VAR reliability: int = 100

== main ==
Courier route cycle starts at dawn.
~ process_dispatches(0)
~ print_final_ledger()
-> DONE

== function process_dispatches(index: int) => void ==
{ if index >= LEN(deliveries):
    Dispatch log complete.
- else:
    ~ temp package: Delivery = deliveries[index]
    ~ temp delay: int = delay_plan[index]
    ~ temp reason: string = delay_reason[index]
    Delivering package {package.id}: {package.cargo}.
    Delay source: {reason} ({delay} minutes).
    ~ current_time = current_time + delay
    ~ temp arrival: int = current_time + package.travel_time
    ~ complete_delivery(package, arrival, index)
    ~ current_time = arrival + 2
    ~ process_dispatches(index + 1)
}

== function complete_delivery(package: Delivery, arrival: int, index: int) => void ==
~ temp late: int = arrival - package.deadline
~ temp payment: int = package.reward
~ temp penalty_rate: int = package.penalty_per_minute

Delivery deadline was {package.deadline}.
Predicted arrival: {arrival}.

{ if late <= 0:
    On time. No penalty applied.
    ~ completed_on_time = completed_on_time + 1
    ~ update_reliability_for_score(1)
- else:
    Shipment is late by {late} minute(s).
    ~ temp raw_penalty: int = late * penalty_rate
    ~ temp final_penalty: int = raw_penalty
    { if final_penalty < 0:
        ~ final_penalty = 0
    }
    ~ late_deliveries = late_deliveries + 1
    ~ payment = payment - final_penalty
    { if payment < 0:
        ~ payment = 0
    }
    ~ penalties = penalties + final_penalty
    ~ update_reliability_for_late(late, package.penalty_per_minute)
}

~ gross_reward = gross_reward + package.reward
~ earned = earned + payment

{ if payment > 0:
    Delivery accepted. Payout this stop: {payment}.
- else:
    Courier receives no payout for package {package.id}.
    ~ failed_deliveries = failed_deliveries + 1
}

== function update_reliability_for_score(bonus: int) => void ==
~ reliability = reliability + bonus + 1
{ if reliability > 100:
    ~ reliability = 100
}

== function update_reliability_for_late(minutes_late: int, risk: int) => void ==
~ temp loss: int = risk + minutes_late
~ reliability = reliability - loss
{ if reliability < 0:
    ~ reliability = 0
}

== function print_final_ledger() => void ==
Courier end-of-day ledger:
Total elapsed dispatch time: {current_time}
Gross scheduled reward: {gross_reward}
Late penalties: {penalties}
Net payout: {earned}
On-time deliveries: {completed_on_time}
Late deliveries: {late_deliveries}
Failed payout deliveries: {failed_deliveries}
Reliability points: {reliability}
Courier performance: {reliability_grade()}

== function reliability_grade() => string ==
{ if reliability >= 95:
    ~ return "Excellent"
- else:
    { if reliability >= 80:
        ~ return "Good"
    - else:
        { if reliability >= 60:
            ~ return "Recovering"
        - else:
            ~ return "Unreliable"
        }
    }
}
