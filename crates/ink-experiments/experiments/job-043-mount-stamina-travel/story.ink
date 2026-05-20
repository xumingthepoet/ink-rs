=== module game ===

STRUCT Mount {
    name: string
    stamina: int
    max_stamina: int
    max_load: int
    speed: int
    recovery: int
}

STRUCT RouteLeg {
    segment: string
    distance: int
    terrain: string
    load: int
}

VAR mounts: Mount[] = [
    %Mount{
        name: "Larkspur",
        stamina: 20,
        max_stamina: 20,
        max_load: 6,
        speed: 8,
        recovery: 4
    },
    %Mount{
        name: "Ironhoof",
        stamina: 28,
        max_stamina: 28,
        max_load: 12,
        speed: 6,
        recovery: 5
    }
]

VAR route: RouteLeg[] = [
    %RouteLeg{ segment: "North Gate to Bramble Road", distance: 6, terrain: "plains", load: 4 },
    %RouteLeg{ segment: "Bramble Road to Cliff Pass", distance: 4, terrain: "hill", load: 5 },
    %RouteLeg{ segment: "Cliff Pass to Marsh Line", distance: 5, terrain: "swamp", load: 9 },
    %RouteLeg{ segment: "Marsh Line to River Ford", distance: 3, terrain: "river", load: 8 },
    %RouteLeg{ segment: "River Ford to Outpost", distance: 7, terrain: "forest", load: 11 }
]

VAR terrain_cost: Dict<string, int> = %{
    "plains": 1,
    "forest": 2,
    "hill": 3,
    "swamp": 4,
    "river": 3
}

VAR active_mount: int = 0
VAR travel_time_hours: int = 0
VAR rest_time_hours: int = 0
VAR aborted: bool = false

== main ==
Post becomes active for a loaded route.
~ report_mounts("Starting mounts")
~ attempt_leg(0)
~ report_trip("Trip complete")
-> DONE

== function attempt_leg(index: int) => void ==
{ if index >= LEN(route):
    ~ return
}
~ temp leg: RouteLeg = route[index]
~ active_mount = select_mount(leg)
~ temp mounted: Mount = mounts[active_mount]
~ temp stamina_needed: int = estimate_stamina_cost(active_mount, leg)
Leg {index + 1}: {leg.segment} ({leg.distance}km, load {leg.load}, terrain {leg.terrain})
~ print_mount_status(active_mount)
{ if mounted.stamina < stamina_needed:
    {mounted.name} needs rest before crossing.
    ~ recover_for_leg(active_mount, stamina_needed)
    { if mounts[active_mount].stamina < stamina_needed:
        Route abort: {leg.segment} cannot be crossed with current mounts.
        ~ aborted = true
    - else:
        ~ cross_leg(index)
        ~ attempt_leg(index + 1)
    }
- else:
    ~ cross_leg(index)
    ~ attempt_leg(index + 1)
}

== function select_mount(leg: RouteLeg) => int ==
~ temp horse_cost: int = estimate_stamina_cost(0, leg)
~ temp mule_cost: int = estimate_stamina_cost(1, leg)
{ if leg.load >= 7:
    { if mounts[1].stamina >= mule_cost:
        ~ return 1
    - else:
        { if mounts[0].stamina >= horse_cost:
            ~ return 0
        - else:
            ~ return 1
        }
    }
    - else:
        { if mounts[0].stamina >= horse_cost:
            ~ return 0
        - else:
            { if mounts[1].stamina >= mule_cost:
                ~ return 1
            - else:
                ~ return 0
            }
        }

}

== function cross_leg(leg_index: int) => void ==
~ temp leg: RouteLeg = route[leg_index]
~ temp stamina_cost: int = estimate_stamina_cost(active_mount, leg)
~ temp travel_hours: int = estimate_travel_hours(active_mount, leg)
~ mounts[active_mount].stamina = mounts[active_mount].stamina - stamina_cost
~ travel_time_hours = travel_time_hours + travel_hours
Riding {route[leg_index].segment} on {mounts[active_mount].name}.
Used {stamina_cost} stamina, remaining {mounts[active_mount].stamina}.
Travel time: {travel_hours} hours.

== function recover_for_leg(mount_index: int, required_stamina: int) => void ==
{ if mounts[mount_index].stamina >= required_stamina:
    ~ return
- else:
    { if mounts[mount_index].stamina >= mounts[mount_index].max_stamina:
        { if required_stamina > mounts[mount_index].max_stamina:
            Route abort check for {mounts[mount_index].name}; stamina cap too low for this leg.
        - else:
            Rest is pointless but this mount is full, aborting.
        }
        ~ aborted = true
    - else:
        ~ temp next_stamina: int = mounts[mount_index].stamina + mounts[mount_index].recovery
        { if next_stamina > mounts[mount_index].max_stamina:
            ~ next_stamina = mounts[mount_index].max_stamina
        }
        ~ mounts[mount_index].stamina = next_stamina
        ~ travel_time_hours = travel_time_hours + 1
        ~ rest_time_hours = rest_time_hours + 1
        Rested with {mounts[mount_index].name}. Stamina now {next_stamina}.
        ~ recover_for_leg(mount_index, required_stamina)
    }

}

== function estimate_stamina_cost(mount_index: int, leg: RouteLeg) => int ==
~ temp terrain: int = terrain_cost[leg.terrain]
~ temp base_cost: int = leg.distance * terrain
~ temp overload_penalty: int = 0
{ if leg.load > (mounts[mount_index].max_load / 2):
    ~ overload_penalty = 2
- else:
    { if leg.load > 3:
        ~ overload_penalty = 1
    }
}
~ return base_cost + overload_penalty

== function estimate_travel_hours(mount_index: int, leg: RouteLeg) => int ==
~ temp terrain: int = terrain_cost[leg.terrain]
~ temp terrain_penalty: int = terrain
~ temp load_penalty: int = 0
{ if leg.load > (mounts[mount_index].max_load - 2):
    ~ load_penalty = 1
}
~ temp speed: int = mounts[mount_index].speed - terrain_penalty - load_penalty
{ if speed < 2:
    ~ speed = 2
}
~ temp total_distance: int = leg.distance * 10
~ temp rounded_hours: int = (total_distance + speed - 1) / speed
~ return rounded_hours

== function report_mounts(label: string) => void ==
-- {label} --
~ print_mount_status(0)
~ print_mount_status(1)

== function print_mount_status(index: int) => void ==
~ temp mount: Mount = mounts[index]
{mount.name}: Stamina {mount.stamina}/{mount.max_stamina}, speed {mount.speed}, load {mount.max_load}

== function report_trip(label: string) => void ==
~ report_mounts(label)
{ if aborted:
    Journey terminated during the route.
- else:
    All legs completed.
    Total time: {travel_time_hours} travel hours, {rest_time_hours} rest hours.
}
