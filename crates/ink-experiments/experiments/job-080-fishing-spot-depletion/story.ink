=== module game ===

STRUCT FishingSpot {
    name: string
    stock: int
    capacity: int
    bait: int
    catch_quality: int
    depletion: int
}

VAR fishing_spots: FishingSpot[] = [
    %FishingSpot{name: "Morning Shoals", stock: 5, capacity: 5, bait: 4, catch_quality: 6, depletion: 0},
    %FishingSpot{name: "Pebble Mouth", stock: 4, capacity: 4, bait: 3, catch_quality: 5, depletion: 0},
    %FishingSpot{name: "Mooring Line", stock: 3, capacity: 3, bait: 5, catch_quality: 7, depletion: 0}
]

VAR current_day: int = 1
VAR total_fish_caught: int = 0
VAR total_catch_quality: int = 0

== main ==
The fisheries master begins a measured harvest calendar.
~ announce_day()
~ run_fishing_days(3)
~ final_harvest_summary()
-> DONE

== function run_fishing_days(total_days: int) => void ==
{ if current_day > total_days:
    ~ return
- else:
    ~ announce_day()
    ~ fish_all_spots(0)
    ~ log_day_end()
    ~ recover_spots(0)
    ~ current_day = current_day + 1
    ~ run_fishing_days(total_days)
}

== function announce_day() => void ==
Day {current_day}

== function fish_all_spots(index: int) => void ==
{ if index >= LEN(fishing_spots):
    ~ return
- else:
    ~ temp spot: FishingSpot = fishing_spots[index]
    {spot.name} starts with stock {spot.stock}, bait {spot.bait}, depletion {spot.depletion}.
    ~ cast_at_spot(index, 2)
    ~ fish_all_spots(index + 1)
}

== function cast_at_spot(index: int, casts_left: int) => void ==
{ if casts_left <= 0:
    ~ return
- else:
    ~ temp result: int = attempt_cast(index)
    { if result > 0:
        A catch is secured with quality {result}.
        ~ total_fish_caught = total_fish_caught + 1
        ~ total_catch_quality = total_catch_quality + result
    - else:
        The cast lands bare at this spot.
    }
    ~ cast_at_spot(index, casts_left - 1)
}

== function attempt_cast(index: int) => int ==
{ if fishing_spots[index].stock <= 0:
    ~ return 0
- else:
    { if fishing_spots[index].bait <= 0:
        ~ return 0
    - else:
        ~ fishing_spots[index].stock = fishing_spots[index].stock - 1
        ~ fishing_spots[index].bait = fishing_spots[index].bait - 1
        ~ fishing_spots[index].depletion = fishing_spots[index].depletion + 2
        ~ temp raw_quality: int = fishing_spots[index].catch_quality - fishing_spots[index].depletion
        ~ return clamp_quality(raw_quality)
    }
}

== function clamp_quality(value: int) => int ==
{ if value < 1:
    ~ return 1
- else:
    { if value > 8:
        ~ return 8
    - else:
        ~ return value
    }
}

== function log_day_end() => void ==
~ report_spots("End of day")
~ report_fish_totals()

== function recover_spots(index: int) => void ==
{ if index >= LEN(fishing_spots):
    ~ return
- else:
    ~ temp restored_stock: int = fishing_spots[index].stock + 2
    ~ temp restored_bait: int = fishing_spots[index].bait + 1
    ~ temp recovered_depletion: int = fishing_spots[index].depletion - 1

    { if restored_stock > fishing_spots[index].capacity:
        ~ restored_stock = fishing_spots[index].capacity
    }
    ~ fishing_spots[index].stock = restored_stock

    { if restored_bait > 6:
        ~ restored_bait = 6
    }
    ~ fishing_spots[index].bait = restored_bait

    { if recovered_depletion < 0:
        ~ recovered_depletion = 0
    }
    ~ fishing_spots[index].depletion = recovered_depletion

    ~ recover_spots(index + 1)
}

== function report_spots(label: string) => void ==
{ label } after-day status:
~ report_spot_rows(0)

== function report_spot_rows(index: int) => void ==
{ if index >= LEN(fishing_spots):
    ~ return
- else:
    ~ temp spot: FishingSpot = fishing_spots[index]
    {spot.name}: stock {spot.stock}, bait {spot.bait}, depletion {spot.depletion}, catch quality {spot.catch_quality}
    ~ report_spot_rows(index + 1)
}

== function report_fish_totals() => void ==
Total caught this season: {total_fish_caught}
Total quality score: {total_catch_quality}
~ temp avg_quality: int = safe_average(total_catch_quality, total_fish_caught)
Average quality: {avg_quality}

== function safe_average(total: int, count: int) => int ==
{ if count <= 0:
    ~ return 0
- else:
    ~ return total / count
}

== function final_harvest_summary() => void ==
Harvest cycle ends.
~ report_spots("Final")
~ report_fish_totals()
