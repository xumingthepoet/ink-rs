=== module festival ===

STRUCT Booth {
    name: string
    stock: int
    capacity: int
    attraction: int
    reward_rate: int
}

VAR booths: Booth[] = [
    %Booth{name: "Lantern Silk", stock: 3, capacity: 5, attraction: 6, reward_rate: 8},
    %Booth{name: "Honeyed Nuts", stock: 5, capacity: 6, attraction: 5, reward_rate: 6},
    %Booth{name: "Ink Carvings", stock: 4, capacity: 4, attraction: 7, reward_rate: 7},
    %Booth{name: "Cider Forge", stock: 2, capacity: 4, attraction: 4, reward_rate: 10}
]

VAR replenishment_order: int[] = [2, 0, 1, 3, 0, 2]
VAR total_slots: int = 6
VAR booth_cursor: int = 0
VAR crowd_attention: int = 18
VAR slot_rewards: int = 0
VAR total_booths_served: int = 0
VAR empty_booths_count: int = 0

== main ==
Festival booths begin rotating through the square.
~ run_slot_cycle(0)
~ print_booth_summary(0)
Festival attendance index: {crowd_attention}
Total rewards this event: {slot_rewards}
Booths visited: {total_booths_served}
Booths that emptied: {empty_booths_count}
-> DONE

== function run_slot_cycle(slot: int) => void ==
{ if slot >= total_slots:
    Rotation complete.
- else:
    ~ temp index: int = booth_cursor
    ~ temp booth: Booth = booths[index]
    ~ temp demand: int = estimate_demand(booth.attraction, crowd_attention)
    ~ temp sold: int = sold_units(demand, booth.stock)
    ~ temp gain: int = sold * booth.reward_rate
    ~ temp missed: int = demand - sold

    Slot {slot + 1}: {booth.name} hosts the crowd.
    Demand estimate: {demand}, sold: {sold}, reward gain: {gain}.

    ~ booths[index].stock = booth.stock - sold
    ~ slot_rewards = slot_rewards + gain
    ~ total_booths_served = total_booths_served + 1
    ~ crowd_attention = crowd_attention + sold * 2
    ~ crowd_attention = crowd_attention + booth.attraction / 2
    { if missed > 0:
        ~ crowd_attention = crowd_attention - missed * 3
        Crowd drops from unmet demand.
    - else:
        Crowds stay focused.
    }

    { if booths[index].stock <= 0:
        ~ empty_booths_count = empty_booths_count + 1
        {booth.name} is now sold out.
    - else:
        {booth.name} still has {booths[index].stock} stock.
    }

    ~ booth_cursor = booth_cursor + 1
    { if booth_cursor >= LEN(booths):
        ~ booth_cursor = 0
    }
    ~ temp restock_target: int = replenishment_order[slot]
    ~ restock_booth(restock_target, slot + 1)
    { if crowd_attention < 0:
        ~ crowd_attention = 0
    }
    ~ run_slot_cycle(slot + 1)
}

== function estimate_demand(attraction: int, crowd: int) => int ==
~ return attraction + crowd / 10

== function sold_units(demand: int, stock: int) => int ==
{ if stock >= demand:
    ~ return demand
- else:
    ~ return stock
}

== function restock_booth(index: int, slot_label: int) => void ==
~ temp booth: Booth = booths[index]
{ if booth.stock < booth.capacity:
    ~ booths[index].stock = booth.stock + 1
    Slot {slot_label} restock: {booth.name} receives 1 unit.
    - else:
        Slot {slot_label} restock: {booth.name} already full.
}

== function print_booth_summary(index: int) => void ==
{ if index >= LEN(booths):
    ~ return
}

~ temp booth: Booth = booths[index]
~ temp state: string = stock_state(booth.stock)
Booth {booth.name} final stock: {state}
~ print_booth_summary(index + 1)

== function stock_state(stock: int) => string ==
{ if stock == 0:
    ~ return "empty"
- else:
    { if stock <= 2:
        ~ return "low"
    - else:
        ~ return "healthy"
    }
}
