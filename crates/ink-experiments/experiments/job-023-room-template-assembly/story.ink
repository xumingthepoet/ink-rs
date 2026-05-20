=== module game ===
STRUCT RoomTemplate {
    id: int
    region: string
    base: string
}

STRUCT Hazard {
    id: int
    detail: string
}

VAR room_template_ids: int[] = [101, 102, 103]
VAR room_hazard_ids: int[] = [1, 2, 0]

VAR room_templates: RoomTemplate[] = [
    %RoomTemplate{
        id: 101,
        region: "Rivergate",
        base: "A long corridor of black stone lined with rusted hooks."
    },
    %RoomTemplate{
        id: 102,
        region: "Cathedral Crypt",
        base: "An echoing hall where statues watch over broken steps."
    },
    %RoomTemplate{
        id: 103,
        region: "Clockwork Loft",
        base: "A cramped chamber full of windless cogs and loose chains."
    }
]

VAR hazard_catalog: Hazard[] = [
    %Hazard{id: 1, detail: "A cracked rigging cable sways above the center path."},
    %Hazard{id: 2, detail: "A row of pressure plates is still warm from the last alarm."}
]

VAR template_by_id: Dict<int, int> = %{
    101: 0,
    102: 1,
    103: 2
}
VAR hazard_by_id: Dict<int, int> = %{
    1: 0,
    2: 1
}

VAR room_has_fog: Dict<int, bool> = %{
    0: true,
    1: false,
    2: true
}
VAR room_has_wet_floor: Dict<int, bool> = %{
    0: true,
    1: true,
    2: false
}
VAR room_has_traps: Dict<int, bool> = %{
    0: false,
    1: true,
    2: true
}

== main ==
Room template assembly.
-> print_room_0

== print_room_0 ==
~ temp template_id: int = room_template_ids[0]
~ temp template_index: int = template_by_id[template_id]
~ temp template: RoomTemplate = room_templates[template_index]
~ temp hazard_id: int = room_hazard_ids[0]
-- Room 1 --
Region: {template.region}
{template.base}
{resolve_hazard(hazard_id)}
{ if room_has_fog[0]:
    Mist crawls through the upper vents.
- else:
    The room is clear.
}
{ if room_has_wet_floor[0]:
    The floor is wet.
- else:
    The floor is dry.
}
{ if room_has_traps[0]:
    Alert beacons indicate active trap pressure.
- else:
    No trap pressure detected.
}

-> print_room_1

== print_room_1 ==
~ temp template_id: int = room_template_ids[1]
~ temp template_index: int = template_by_id[template_id]
~ temp template: RoomTemplate = room_templates[template_index]
~ temp hazard_id: int = room_hazard_ids[1]
-- Room 2 --
Region: {template.region}
{template.base}
{resolve_hazard(hazard_id)}
{ if room_has_fog[1]:
    Mist crawls through the upper vents.
- else:
    The room is clear.
}
{ if room_has_wet_floor[1]:
    The floor is wet.
- else:
    The floor is dry.
}
{ if room_has_traps[1]:
    Alert beacons indicate active trap pressure.
- else:
    No trap pressure detected.
}

-> print_room_2

== print_room_2 ==
~ temp template_id: int = room_template_ids[2]
~ temp template_index: int = template_by_id[template_id]
~ temp template: RoomTemplate = room_templates[template_index]
~ temp hazard_id: int = room_hazard_ids[2]
-- Room 3 --
Region: {template.region}
{template.base}
{resolve_hazard(hazard_id)}
{ if room_has_fog[2]:
    Mist crawls through the upper vents.
- else:
    The room is clear.
}
{ if room_has_wet_floor[2]:
    The floor is wet.
- else:
    The floor is dry.
}
{ if room_has_traps[2]:
    Alert beacons indicate active trap pressure.
- else:
    No trap pressure detected.
}

-> DONE

== function resolve_hazard(hazard_id: int) => string ==
{ if hazard_id == 0:
    ~ return "No immediate hazard for this room."
- else:
    ~ temp hazard_index: int = hazard_by_id[hazard_id]
    ~ temp hazard: Hazard = hazard_catalog[hazard_index]
    ~ return "Hazard notice: " + hazard.detail
}
