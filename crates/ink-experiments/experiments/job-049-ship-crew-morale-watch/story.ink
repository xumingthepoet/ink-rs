=== module watch ===

STRUCT CrewMember {
    name: string
    fatigue: int
    morale: int
    watch_count: int
}

STRUCT WatchShift {
    day: int
    first_watch: int
    second_watch: int
    third_watch: int
    weather_pressure: int
    storm: string
}

VAR crew: CrewMember[] = [
    %CrewMember{
        name: "Nera",
        fatigue: 2,
        morale: 22,
        watch_count: 0
    },
    %CrewMember{
        name: "Quill",
        fatigue: 1,
        morale: 19,
        watch_count: 0
    },
    %CrewMember{
        name: "Rook",
        fatigue: 1,
        morale: 21,
        watch_count: 0
    }
]

VAR watch_plan: WatchShift[] = [
    %WatchShift{
        day: 1,
        first_watch: 0,
        second_watch: 1,
        third_watch: 2,
        weather_pressure: 2,
        storm: "night_mist"
    },
    %WatchShift{
        day: 2,
        first_watch: 0,
        second_watch: 0,
        third_watch: 2,
        weather_pressure: 5,
        storm: "crosswind"
    },
    %WatchShift{
        day: 3,
        first_watch: 1,
        second_watch: 1,
        third_watch: 2,
        weather_pressure: 8,
        storm: "heavy_storm"
    },
    %WatchShift{
        day: 4,
        first_watch: 0,
        second_watch: 2,
        third_watch: 2,
        weather_pressure: 6,
        storm: "choppy_swell"
    },
    %WatchShift{
        day: 5,
        first_watch: 0,
        second_watch: 1,
        third_watch: 2,
        weather_pressure: 3,
        storm: "clear_breeze"
    }
]

== main ==
Watch cycle begins for the long passage.
~ run_watch_cycles(0)
~ print_final_status()
-> DONE

== function run_watch_cycles(index: int) => void ==
{ if index < LEN(watch_plan):
    ~ temp shift: WatchShift = watch_plan[index]
    Day {shift.day} weather condition: {shift.storm}, pressure {shift.weather_pressure}.
    ~ apply_watch_shift(shift.first_watch, shift.second_watch, shift.third_watch, shift.weather_pressure, 0)
    Current crew status.
    ~ print_crew(0)
    ~ print_watch_risk(shift.weather_pressure, shift.day)
    ~ run_watch_cycles(index + 1)
}

== function apply_watch_shift(first: int, second: int, third: int, pressure: int, index: int) => void ==
{ if index < LEN(crew):
    { if index == first || index == second || index == third:
        ~ apply_watch_stress(index, pressure)
    - else:
        ~ apply_rest(index, pressure)
    }
    ~ apply_watch_shift(first, second, third, pressure, index + 1)
}

== function apply_watch_stress(member_index: int, pressure: int) => void ==
~ temp member: CrewMember = crew[member_index]
~ temp fatigue_next: int = member.fatigue + 2
~ temp morale_loss: int = 1 + pressure / 3
~ temp morale_next: int = member.morale - morale_loss
~ temp pressure_penalty: int = 0
{ if pressure >= 8:
    ~ pressure_penalty = 1
}
~ morale_next = morale_next - pressure_penalty
{ if fatigue_next >= 8:
    ~ morale_next = morale_next - 1
}
{ if morale_next < 0:
    ~ morale_next = 0
}
~ crew[member_index].fatigue = fatigue_next
~ crew[member_index].morale = morale_next
~ crew[member_index].watch_count = member.watch_count + 1

== function apply_rest(member_index: int, pressure: int) => void ==
~ temp member: CrewMember = crew[member_index]
~ temp fatigue_next: int = member.fatigue - 1
{ if fatigue_next < 0:
    ~ fatigue_next = 0
}
~ temp morale_gain: int = 1
{ if pressure >= 7:
    ~ morale_gain = 0
}
~ temp morale_next: int = member.morale + morale_gain
~ crew[member_index].fatigue = fatigue_next
~ crew[member_index].morale = morale_next

== function print_crew(index: int) => void ==
{ if index < LEN(crew):
    ~ temp member: CrewMember = crew[index]
    {member.name} | watch_count {member.watch_count} | fatigue {member.fatigue} | morale {member.morale}
    ~ print_crew(index + 1)
}

== function print_watch_risk(pressure: int, day: int) => void ==
~ temp score: int = mutiny_risk_score(pressure)
Day {day} mutiny risk score: {score}
{ if score >= 30:
    Warning: mutiny risk is critical. Watches can no longer continue as scheduled.
- else:
    { if score >= 22:
        Mutiny risk is elevated; senior command should rotate decks.
    - else:
        Crew remains stable under watch pressure.
    }
}

== function mutiny_risk_score(pressure: int) => int ==
~ temp total_fatigue: int = sum_fatigue(0)
~ temp total_morale: int = sum_morale(0)
~ temp morale_deficit: int = 66 - total_morale
{ if morale_deficit < 0:
    ~ morale_deficit = 0
}
~ temp score: int = total_fatigue + morale_deficit + pressure
~ return score

== function sum_fatigue(index: int) => int ==
{ if index >= LEN(crew):
    ~ return 0
- else:
    ~ temp member: CrewMember = crew[index]
    ~ return sum_fatigue(index + 1) + member.fatigue
}

== function sum_morale(index: int) => int ==
{ if index >= LEN(crew):
    ~ return 0
- else:
    ~ temp member: CrewMember = crew[index]
    ~ return sum_morale(index + 1) + member.morale
}

== function print_final_status() => void ==
Final watch completion summary.
Current crew status.
~ print_crew(0)
~ temp score: int = mutiny_risk_score(0)
~ print_final_watch_risk(score)
~ print_watch_distribution()

== function print_final_watch_risk(score: int) => void ==
Final mutiny risk score: {score}
{ if score >= 30:
    Warning: mutiny risk is critical at voyage close.
- else:
    { if score >= 22:
        Mutiny risk is elevated near the end of this cycle.
    - else:
        Closing risk remains manageable.
    }
}

== function print_watch_distribution() => void ==
Watch burden:
~ print_watch_distribution_entry(0)

== function print_watch_distribution_entry(index: int) => void ==
{ if index < LEN(crew):
    ~ temp member: CrewMember = crew[index]
    {member.name} watched {member.watch_count} shifts.
    ~ print_watch_distribution_entry(index + 1)
}
