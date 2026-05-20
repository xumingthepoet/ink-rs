=== module farm ===

STRUCT Crop {
    name: string
    growth: int
    water: int
    fertilizer: int
    pests: int
    harvested: bool
    yield_total: int
}

VAR crop_plots: Crop[] = [
    %Crop{
        name: "Tomato",
        growth: 0,
        water: 0,
        fertilizer: 0,
        pests: 0,
        harvested: false,
        yield_total: 0
    },
    %Crop{
        name: "Barley",
        growth: 0,
        water: 0,
        fertilizer: 0,
        pests: 0,
        harvested: false,
        yield_total: 0
    },
    %Crop{
        name: "Herb",
        growth: 0,
        water: 0,
        fertilizer: 0,
        pests: 0,
        harvested: false,
        yield_total: 0
    }
]

== main ==
Seasonal crop monitoring begins.
~ simulate_day(1)
~ print_total_yield()
-> DONE

== function simulate_day(day: int) => void ==
{ if day <= 6:
    ~ run_farm_day(day)
- else:
    Season watch is over.
}

== function run_farm_day(day: int) => void ==
Day {day} report.
~ apply_plan_for_day(day)
~ grow_all_crops(day, 0)
~ apply_pest_events(day)
~ maybe_harvest_all(day, 0)
-- Crop states day {day} --
~ print_crop_state_entries(0)
~ simulate_day(day + 1)

== function apply_plan_for_day(day: int) => void ==
{ if day == 1:
    Ground is warm. Morning irrigation starts with heavy water.
    ~ set_water(0, 2)
    ~ set_water(1, 1)
    ~ set_water(2, 1)
    ~ set_fertilizer(0, 2)
- else:
    { if day == 2:
        Rainfall is light. Selective tending continues.
        ~ set_water(0, 1)
        ~ set_water(1, 2)
        ~ set_water(2, 0)
        ~ set_fertilizer(1, 1)
    - else:
        { if day == 3:
            A storm rolls in. Saturating moisture with no care delay.
            ~ set_water(0, 2)
            ~ set_water(1, 2)
            ~ set_water(2, 2)
            ~ set_pest(0, 1)
            ~ set_pest(2, 1)
        - else:
            { if day == 4:
                A dry heat spike. No irrigation and weak guards.
                ~ set_water(0, 0)
                ~ set_water(1, 0)
                ~ set_water(2, 0)
                ~ spread_pests_if_present(0, 2)
            - else:
                { if day == 5:
                    Targeted spray suppresses pests. Evening hydration is uneven.
                    ~ set_water(0, 1)
                    ~ set_water(1, 1)
                    ~ set_water(2, 1)
                    ~ set_pest(0, 0)
                    ~ set_pest(1, 0)
                    ~ set_pest(2, 0)
                - else:
                    Day six finish-up irrigation before harvest.
                    ~ set_water(0, 2)
                    ~ set_water(1, 2)
                    ~ set_water(2, 1)
                }
            }
        }
    }
}

== function set_water(index: int, amount: int) => void ==
~ crop_plots[index].water = amount

== function set_fertilizer(index: int, turns: int) => void ==
~ crop_plots[index].fertilizer = turns

== function set_pest(index: int, level: int) => void ==
~ crop_plots[index].pests = level

== function spread_pests_if_present(source: int, target: int) => void ==
{ if crop_plots[source].pests > 0:
    ~ crop_plots[target].pests = 1
    {crop_plots[target].name} receives secondary pests from nearby beds.
- else:
    Pest pressure does not spread today.
}

== function grow_all_crops(day: int, index: int) => void ==
{ if index < LEN(crop_plots):
    ~ temp crop: Crop = crop_plots[index]
    { if crop.harvested:
        {crop.name} is already harvested.
    - else:
        ~ temp growth_gain: int = 1
        { if crop.water == 0:
            ~ growth_gain = growth_gain - 1
        - else:
            { if crop.water == 1:
                ~ growth_gain = growth_gain + 1
            - else:
                ~ growth_gain = growth_gain + 2
            }
        }
        { if crop.fertilizer > 0:
            ~ growth_gain = growth_gain + 2
            ~ crop_plots[index].fertilizer = crop.fertilizer - 1
        }
        { if crop.pests > 0:
            ~ growth_gain = growth_gain - 2
        }
        ~ temp fresh_growth: int = crop.growth + growth_gain
        { if fresh_growth < 0:
            ~ fresh_growth = 0
        - else:
            { if fresh_growth > 12:
                ~ fresh_growth = 12
            }
        }
        ~ crop_plots[index].growth = fresh_growth
        ~ crop_plots[index].water = 0
    }
    ~ grow_all_crops(day, index + 1)
}

== function apply_pest_events(day: int) => void ==
~ if_pests_over_limit(0)

== function if_pests_over_limit(index: int) => void ==
{ if index < LEN(crop_plots):
    ~ temp crop: Crop = crop_plots[index]
    { if crop.pests > 1:
        ~ crop_plots[index].pests = 1
    }
    ~ if_pests_over_limit(index + 1)
}

== function maybe_harvest_all(day: int, index: int) => void ==
{ if index < LEN(crop_plots):
    ~ temp crop: Crop = crop_plots[index]
    { if !crop.harvested && crop.growth >= 7:
        ~ temp base_yield: int = crop.growth / 2
        ~ temp pest_loss: int = crop.pests * 2
        ~ temp total_gain: int = base_yield - pest_loss
        { if total_gain < 1:
            ~ total_gain = 1
        }
        { if total_gain > 6:
            ~ total_gain = 6
        }
        ~ crop_plots[index].yield_total = crop.yield_total + total_gain
        ~ crop_plots[index].harvested = true
        {crop.name} is harvested on day {day} for {total_gain} units.
    - else:
        {crop.name} is not mature enough for harvest on day {day}.
    }
    ~ maybe_harvest_all(day, index + 1)
}

== function print_crop_state_entries(index: int) => void ==
{ if index < LEN(crop_plots):
    ~ temp crop: Crop = crop_plots[index]
    ~ temp stage: string = growth_stage(crop.growth)
    {crop.name} | {stage} | growth {crop.growth}
    Water {crop.water}
    Fertilizer {crop.fertilizer}
    Pests {crop.pests}
    Yield banked {crop.yield_total}
    ~ print_crop_state_entries(index + 1)
}

== function growth_stage(growth: int) => string ==
{ if growth >= 10:
    ~ return "ripe"
- else:
    { if growth >= 7:
        ~ return "near_ripe"
    - else:
        { if growth >= 4:
            ~ return "growing"
        - else:
            ~ return "young"
        }
    }
}

== function print_total_yield() => void ==
Final yield ledger.
~ print_yield_entry(0)

== function print_yield_entry(index: int) => void ==
{ if index < LEN(crop_plots):
    ~ temp crop: Crop = crop_plots[index]
    {crop.name}: {crop.yield_total}
    ~ print_yield_entry(index + 1)
}
