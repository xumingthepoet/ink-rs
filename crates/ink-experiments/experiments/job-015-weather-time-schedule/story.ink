=== module game ===
STRUCT SceneModifier {
    visibility: int
    acoustics: int
    encounter: int
}

STRUCT WeatherProfile {
    name: string
    scene: SceneModifier
    note: string
}

VAR day_schedule: string[] = [
    "dawn",
    "day",
    "dusk",
    "night",
    "dawn",
    "day",
    "dusk",
    "night"
]

VAR weather_by_segment: Dict<string, WeatherProfile> = %{
    "dawn": %WeatherProfile{
        name: "Mist",
        scene: %SceneModifier{visibility: 2, acoustics: 1, encounter: 3},
        note: "Cold haze in the roads."
    },
    "day": %WeatherProfile{
        name: "Clear",
        scene: %SceneModifier{visibility: 7, acoustics: 2, encounter: 5},
        note: "High sunlight and long shadows."
    },
    "dusk": %WeatherProfile{
        name: "Cloud",
        scene: %SceneModifier{visibility: 4, acoustics: 4, encounter: 7},
        note: "The city glows amber and dim."
    },
    "night": %WeatherProfile{
        name: "Storm",
        scene: %SceneModifier{visibility: 1, acoustics: 8, encounter: 10},
        note: "Rain and distant sirens."
    }
}

== main ==
Weather timeline enters.
~ temp _schedule: int = run_schedule(0)
-> DONE

== function run_schedule(hour: int) => int ==
{ if hour >= LEN(day_schedule):
    Schedule complete.
    ~ return 0
- else:
    ~ temp segment: string = day_schedule[hour]
    {weather_by_segment[segment].name} at hour {hour + 1}. {weather_by_segment[segment].note}
    ~ temp _mods_printed: int = print_modifiers(segment)
    { if is_stormy(segment):
        The storm increases contact risk this hour.
    - else:
        Standard movement rules hold.
    }
    ~ return run_schedule(hour + 1)
}

== function print_modifiers(segment: string) => int ==
Visibility: {visibility_modifier(segment)}
Acoustics: {acoustics_modifier(segment)}
Encounter modifier: {encounter_modifier(segment)}
~ return 0

== function visibility_modifier(segment: string) => int ==
~ return weather_by_segment[segment].scene.visibility

== function acoustics_modifier(segment: string) => int ==
~ return weather_by_segment[segment].scene.acoustics

== function encounter_modifier(segment: string) => int ==
~ return weather_by_segment[segment].scene.encounter

== function is_stormy(segment: string) => bool ==
~ return encounter_modifier(segment) >= 8 && visibility_modifier(segment) <= 2
