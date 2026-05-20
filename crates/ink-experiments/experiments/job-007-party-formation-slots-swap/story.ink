=== module game ===
STRUCT PartySlot {
    name: string
    role: string
    ready: bool
}

VAR formation: PartySlot[] = [
    %PartySlot{name: "Rin", role: "Tank", ready: true},
    %PartySlot{name: "Mira", role: "Healer", ready: true},
    %PartySlot{name: "Niko", role: "Ranger", ready: true},
    %PartySlot{name: "Tao", role: "Mage", ready: false}
]

== main ==
Party formation:
-> show_formation(0)
Swap request: indices 0 and 2.
-> swap_slots(0, 2)
-> show_formation(0)
Swap request: indices 1 and 3.
-> swap_slots(1, 3)
-> show_formation(0)
-> DONE

== show_formation(index: int) ==
{ if index >= LEN(formation):
    ->->
- else:
    Slot {index + 1}: {formation[index].name} / {formation[index].role} / ready: {formation[index].ready}
    -> show_formation(index + 1)
}

== swap_slots(first: int, second: int) ==
~ temp first_name: string = formation[first].name
~ temp first_role: string = formation[first].role
~ temp first_ready: bool = formation[first].ready
~ formation[first].name = formation[second].name
~ formation[first].role = formation[second].role
~ formation[first].ready = formation[second].ready
~ formation[second].name = first_name
~ formation[second].role = first_role
~ formation[second].ready = first_ready
->->
