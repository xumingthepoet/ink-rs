=== module game ===

STRUCT Species {
    name: string
    population: int
    growth_rate: int
    habitat_pressure: int
    role: string
    prey: string
    hunt_rate: int
}

VAR species: Species[] = [
    %Species{
        name: "Hare",
        population: 520,
        growth_rate: 30,
        habitat_pressure: 4,
        role: "prey",
        prey: "",
        hunt_rate: 0
    },
    %Species{
        name: "Mouse",
        population: 420,
        growth_rate: 34,
        habitat_pressure: 3,
        role: "prey",
        prey: "",
        hunt_rate: 0
    },
    %Species{
        name: "Wolf",
        population: 115,
        growth_rate: 10,
        habitat_pressure: 5,
        role: "predator",
        prey: "Hare",
        hunt_rate: 3
    },
    %Species{
        name: "Owl",
        population: 78,
        growth_rate: 12,
        habitat_pressure: 4,
        role: "predator",
        prey: "Mouse",
        hunt_rate: 2
    }
]

VAR carrying_capacity: int = 1800
VAR total_kill_events: int = 0
VAR cycles: int = 4
VAR season_growth_shift: Dict<int, int> = %{1: 6, 2: 2, 3: 0, 4: 4}

== main ==
Territory ecology simulation starts.
~ print_species("Initial state")
~ simulate_season(1)
~ print_species("Final state")
~ print_ecology_summary()
-> DONE

== function print_species(label: string) => void ==
Territory snapshot: {label}
~ print_species_rows(0)

== function print_species_rows(index: int) => void ==
{ if index >= LEN(species):
    ~ return
- else:
    ~ temp animal: Species = species[index]
    {animal.name} - pop {animal.population}, growth {animal.growth_rate}, role {animal.role}, territory {animal.habitat_pressure}
    ~ print_species_rows(index + 1)
}

== function simulate_season(season: int) => void ==
{ if season > cycles:
    ~ return
- else:
    ~ temp total: int = total_population(0)
    ~ temp pressure: int = pressure_index(total)
    Season {season} begins.
    Pressure index {pressure}.
    ~ grow_species(0, season, pressure)
    ~ process_predation(0, pressure)
    ~ enforce_territory_pressure(0, pressure)
    ~ print_season_state(season)
    ~ process_season_summary()
    ~ simulate_season(season + 1)
}

== function grow_species(index: int, season: int, pressure: int) => void ==
{ if index >= LEN(species):
    ~ return
- else:
    ~ temp creature: Species = species[index]
    ~ temp growth_shift: int = season_growth_shift[season]
    ~ temp pressure_drag: int = pressure / 35
    ~ temp seasonal_growth: int = creature.growth_rate + growth_shift - pressure_drag
    { if creature.role == "predator":
        ~ seasonal_growth = seasonal_growth - 2
    }

    { if seasonal_growth < -10:
        ~ seasonal_growth = -10
    }

    ~ temp delta: int = (creature.population * seasonal_growth) / 100
    ~ temp next_population: int = creature.population + delta
    { if seasonal_growth > 0:
        { if delta == 0:
            ~ delta = 1
            ~ next_population = next_population + 1
        }
    }

    { if next_population < 0:
        ~ next_population = 0
    }

    ~ species[index].population = next_population
    ~ grow_species(index + 1, season, pressure)
}

== function process_predation(index: int, pressure: int) => void ==
{ if index >= LEN(species):
    ~ return
- else:
    { if species[index].role == "predator":
        ~ temp predator_index: int = index
        ~ temp predator: Species = species[predator_index]
        ~ temp prey_index: int = locate_species(predator.prey, 0)

        { if prey_index != -1 and predator.population > 0:
            ~ temp prey: Species = species[prey_index]
            ~ temp base_kill: int = predator.population * predator.hunt_rate
    ~ temp pressure_reduction: int = 5 + pressure / 40
            ~ temp kills: int = base_kill / pressure_reduction
            { if kills > prey.population:
                ~ kills = prey.population
            }

            ~ species[prey_index].population = prey.population - kills
            ~ total_kill_events = total_kill_events + kills
            ~ temp predator_gain: int = kills / 6
            ~ species[predator_index].population = predator.population + predator_gain
            { if kills > 0:
                {predator.name} consumed {kills} of {prey.name}.
            - else:
                {predator.name} found little prey.
            }
        - else:
            {species[index].name} has no valid prey target now.
        }
    }
    ~ process_predation(index + 1, pressure)
}

== function enforce_territory_pressure(index: int, pressure: int) => void ==
{ if index >= LEN(species):
    ~ return
- else:
    ~ temp animal: Species = species[index]
    { if pressure > 145:
        ~ temp overfull: int = pressure - 145
        ~ temp loss: int = overfull / 20
        ~ total_kill_events = total_kill_events + (loss * animal.population) / 100
        { if loss > 0:
            { if animal.population < loss:
                ~ species[index].population = 0
            - else:
                ~ species[index].population = animal.population - loss
            }
        }
    }
    ~ enforce_territory_pressure(index + 1, pressure)
}

== function print_season_state(season: int) => void ==
After season {season} state.
~ print_species_rows(0)

== function locate_species(name: string, index: int) => int ==
{ if index >= LEN(species):
    ~ return -1
- else:
    { if species[index].name == name:
        ~ return index
    - else:
        ~ return locate_species(name, index + 1)
    }
}

== function total_population(index: int) => int ==
{ if index >= LEN(species):
    ~ return 0
- else:
    ~ return species[index].population + total_population(index + 1)
}

== function pressure_index(total: int) => int ==
~ temp index: int = (total * 100) / carrying_capacity
~ return index

== function process_season_summary() => void ==
~ temp total: int = total_population(0)
~ temp pressure: int = pressure_index(total)
~ temp preys: int = count_role("prey", 0)
~ temp predators: int = count_role("predator", 0)
~ temp hare_index: int = locate_species("Hare", 0)
~ temp mouse_index: int = locate_species("Mouse", 0)
~ temp hare: Species = species[hare_index]
~ temp mouse: Species = species[mouse_index]
Ecosystem status:
Total organisms: {total}
Pressure index: {pressure}
Prey groups: {preys}, predators: {predators}
Hare {hare.population} Mouse {mouse.population}
Kill events so far: {total_kill_events}
{ if pressure >= 170:
    Habitat is overrun and stability is falling.
- else:
    { if pressure >= 145:
        Habitat is crowded, but still bounded by predation cycles.
    - else:
        Habitat remains in balanced turnover.
    }
}

== function count_role(target: string, index: int) => int ==
{ if index >= LEN(species):
    ~ return 0
- else:
    { if species[index].role == target:
        ~ return 1 + count_role(target, index + 1)
    - else:
        ~ return count_role(target, index + 1)
    }
}

== function print_ecology_summary() => void ==
Final wildlife summary.
~ temp total: int = total_population(0)
~ temp pressure: int = pressure_index(total)
~ temp kills: int = total_kill_events
~ temp predators: int = count_role("predator", 0)
~ temp preys: int = count_role("prey", 0)
~ temp hare_index: int = locate_species("Hare", 0)
~ temp mouse_index: int = locate_species("Mouse", 0)
~ temp hare: Species = species[hare_index]
~ temp mouse: Species = species[mouse_index]
Total population: {total}
Pressure index: {pressure}
Predator groups: {predators}
Prey groups: {preys}
Key populations:
Hare {hare.population}
Mouse {mouse.population}
Total predation events logged: {kills}
{ if kills >= 250:
    Territorial pressure collapsed by intense feeding pressure.
- else:
    Territorial pressure stayed within long-tail dynamics.
}
