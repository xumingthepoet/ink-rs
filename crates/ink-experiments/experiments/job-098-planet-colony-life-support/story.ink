=== module game ===

STRUCT SupportModule {
    name: string
    oxygen_delta: int
    water_delta: int
    power_delta: int
}

STRUCT FailureEvent {
    cycle: int
    module: string
    oxygen_loss: int
    water_loss: int
    power_loss: int
    repair_oxygen: int
    repair_water: int
    repair_power: int
    repair_hours: int
}

VAR modules: SupportModule[] = [
    %SupportModule{ name: "Oxygen Scrubber", oxygen_delta: 13, water_delta: -1, power_delta: -4 },
    %SupportModule{ name: "Water Recycler", oxygen_delta: 0, water_delta: 11, power_delta: -3 },
    %SupportModule{ name: "Power Hub", oxygen_delta: 0, water_delta: 0, power_delta: 16 }
]

VAR failure_plan: FailureEvent[] = [
    %FailureEvent{
        cycle: 2,
        module: "Water Recycler",
        oxygen_loss: 0,
        water_loss: 12,
        power_loss: 4,
        repair_oxygen: 0,
        repair_water: 7,
        repair_power: 1,
        repair_hours: 3
    },
    %FailureEvent{
        cycle: 4,
        module: "Oxygen Scrubber",
        oxygen_loss: 15,
        water_loss: 0,
        power_loss: 3,
        repair_oxygen: 8,
        repair_water: 1,
        repair_power: 0,
        repair_hours: 4
    },
    %FailureEvent{
        cycle: 5,
        module: "Power Hub",
        oxygen_loss: 0,
        water_loss: 0,
        power_loss: 18,
        repair_oxygen: 0,
        repair_water: 0,
        repair_power: 10,
        repair_hours: 5
    }
]

VAR module_online: Dict<string, bool> = %{
    "Oxygen Scrubber": true,
    "Water Recycler": true,
    "Power Hub": true
}

VAR module_failures: Dict<string, int> = %{
    "Oxygen Scrubber": 0,
    "Water Recycler": 0,
    "Power Hub": 0
}

VAR module_repairs: Dict<string, int> = %{
    "Oxygen Scrubber": 0,
    "Water Recycler": 0,
    "Power Hub": 0
}

VAR oxygen_units: int = 58
VAR water_units: int = 54
VAR power_units: int = 43

VAR total_failures: int = 0
VAR total_repairs: int = 0
VAR total_repair_hours: int = 0
VAR critical_cycles: int = 0
VAR failure_seen_this_cycle: bool = false

== main ==
Planet colony life support simulation.
Initial reserves:
Oxygen {oxygen_units}, Water {water_units}, Power {power_units}
~ run_cycles(1)
~ print_summary()
-> DONE

== function run_cycles(cycle: int) => void ==
{ if cycle > 6:
    Life support cycle log complete.
- else:
    Cycle {cycle} start.
    ~ apply_baseline_demand()
    ~ apply_module_support(0)
    ~ failure_seen_this_cycle = false
    ~ process_failures_for_cycle(cycle, 0)
    ~ check_critical_state()
    End cycle {cycle} reserves:
    Oxygen {oxygen_units}, Water {water_units}, Power {power_units}
    ~ run_cycles(cycle + 1)
}

== function apply_baseline_demand() => void ==
~ oxygen_units = clamp_non_negative(oxygen_units - 10)
~ water_units = clamp_non_negative(water_units - 8)
~ power_units = clamp_non_negative(power_units - 9)
Baseline life support demand applied.

== function apply_module_support(index: int) => void ==
{ if index >= LEN(modules):
    ~ return
- else:
    ~ temp module: SupportModule = modules[index]
    { if module_online[module.name]:
        ~ oxygen_units = clamp_non_negative(oxygen_units + module.oxygen_delta)
        ~ water_units = clamp_non_negative(water_units + module.water_delta)
        ~ power_units = clamp_non_negative(power_units + module.power_delta)
        {module.name} online contribution applied.
    - else:
        {module.name} offline. No contribution this cycle.
    }
    ~ apply_module_support(index + 1)
}

== function process_failures_for_cycle(cycle: int, index: int) => void ==
{ if index >= LEN(failure_plan):
    { if failure_seen_this_cycle == false:
        No failures this cycle.
    }
- else:
    ~ temp event: FailureEvent = failure_plan[index]
    { if event.cycle == cycle:
        ~ failure_seen_this_cycle = true
        ~ module_online[event.module] = false
        ~ module_failures[event.module] = module_failures[event.module] + 1
        ~ total_failures = total_failures + 1

        Failure: {event.module}
        ~ oxygen_units = clamp_non_negative(oxygen_units - event.oxygen_loss)
        ~ water_units = clamp_non_negative(water_units - event.water_loss)
        ~ power_units = clamp_non_negative(power_units - event.power_loss)
        Emergency loss O:{event.oxygen_loss} W:{event.water_loss} P:{event.power_loss}

        ~ repair_module(event)
    }
    ~ process_failures_for_cycle(cycle, index + 1)
}

== function repair_module(event: FailureEvent) => void ==
~ oxygen_units = clamp_non_negative(oxygen_units + event.repair_oxygen)
~ water_units = clamp_non_negative(water_units + event.repair_water)
~ power_units = clamp_non_negative(power_units + event.repair_power)

~ module_online[event.module] = true
~ module_repairs[event.module] = module_repairs[event.module] + 1
~ total_repairs = total_repairs + 1
~ total_repair_hours = total_repair_hours + event.repair_hours
Repair completed for {event.module} in {event.repair_hours} hours.

== function check_critical_state() => void ==
{ if oxygen_units < 35:
    ~ critical_cycles = critical_cycles + 1
    Critical threshold crossed.
- else:
    { if water_units < 35:
        ~ critical_cycles = critical_cycles + 1
        Critical threshold crossed.
    - else:
        { if power_units < 25:
            ~ critical_cycles = critical_cycles + 1
            Critical threshold crossed.
        - else:
            All resources above critical thresholds.
        }
    }
}

== function print_summary() => void ==
Life support summary:
Total failures: {total_failures}
Total repairs: {total_repairs}
Repair hours: {total_repair_hours}
Critical cycles: {critical_cycles}
Final reserves:
Oxygen {oxygen_units}, Water {water_units}, Power {power_units}
~ print_module_stats(0)
~ print_outcome()

== function print_module_stats(index: int) => void ==
{ if index >= LEN(modules):
    ~ return
- else:
    ~ temp module: SupportModule = modules[index]
    ~ temp failure_count: int = module_failures[module.name]
    ~ temp repair_count: int = module_repairs[module.name]
    {module.name}: failures {failure_count}, repairs {repair_count}
    ~ print_module_stats(index + 1)
}

== function print_outcome() => void ==
{ if oxygen_units >= 65 && water_units >= 60 && power_units >= 30 && critical_cycles == 0:
    Survival outcome: colony survives with stable reserves.
- else:
    { if oxygen_units >= 45 && water_units >= 40 && power_units >= 20:
        Survival outcome: colony survives but remains fragile.
    - else:
        Survival outcome: colony life support collapses.
    }
}

== function clamp_non_negative(value: int) => int ==
{ if value < 0:
    ~ return 0
- else:
    ~ return value
}
