=== module game ===

STRUCT Room {
    name: string
    room_class: string
    rate: int
    clean: bool
    assigned_guest: string
}

STRUCT Guest {
    name: string
    preferred_class: string
    budget: int
    max_overpay: int
    cancels: bool
}

STRUCT Reservation {
    guest_name: string
    room_name: string
    paid: int
    refund: int
    status: string
    canceled: bool
}

VAR rooms: Room[] = [
    %Room{name: "Morningside Loft", room_class: "deluxe", rate: 36, clean: true, assigned_guest: "vacant"},
    %Room{name: "Fjord Bunk", room_class: "compact", rate: 18, clean: true, assigned_guest: "vacant"},
    %Room{name: "North Hall", room_class: "compact", rate: 16, clean: true, assigned_guest: "vacant"},
    %Room{name: "Courtyard Suite", room_class: "suite", rate: 40, clean: false, assigned_guest: "vacant"}
]

VAR guests: Guest[] = [
    %Guest{name: "Master Venn", preferred_class: "suite", budget: 42, max_overpay: 6, cancels: true},
    %Guest{name: "Lio the Cartographer", preferred_class: "deluxe", budget: 55, max_overpay: 10, cancels: false},
    %Guest{name: "Ariya", preferred_class: "compact", budget: 24, max_overpay: 4, cancels: false},
    %Guest{name: "Jorah", preferred_class: "compact", budget: 19, max_overpay: 2, cancels: false},
    %Guest{name: "Kessa", preferred_class: "deluxe", budget: 28, max_overpay: 8, cancels: true},
    %Guest{name: "Bram", preferred_class: "suite", budget: 66, max_overpay: 18, cancels: false}
]

VAR reservations: Reservation[] = [
    %Reservation{guest_name: "", room_name: "", paid: 0, refund: 0, status: "empty", canceled: false},
    %Reservation{guest_name: "", room_name: "", paid: 0, refund: 0, status: "empty", canceled: false},
    %Reservation{guest_name: "", room_name: "", paid: 0, refund: 0, status: "empty", canceled: false},
    %Reservation{guest_name: "", room_name: "", paid: 0, refund: 0, status: "empty", canceled: false},
    %Reservation{guest_name: "", room_name: "", paid: 0, refund: 0, status: "empty", canceled: false},
    %Reservation{guest_name: "", room_name: "", paid: 0, refund: 0, status: "empty", canceled: false}
]

VAR reservation_count: int = 0
VAR treasury: int = 0
VAR overbook_limit: int = 1
VAR overbook_price: int = 28
VAR overbook_refund_bonus: int = 6
VAR refunded_total: int = 0
VAR vacancy_tokens: int = 0

== main ==
The inn clerk opens tonight's ledger at moonrise.
~ print_status_intro()
~ process_bookings(0)
~ apply_cancellations()
~ apply_overbook_resolution()
~ final_report()
-> DONE

== function print_status_intro() => void ==
Room inventory:
~ print_rooms(0)

== function print_rooms(index: int) => void ==
{ if index >= LEN(rooms):
    ~ return
- else:
    ~ temp room: Room = rooms[index]
    {room.name}: class {room.room_class}, rate {room.rate}, clean {room.clean}
    ~ print_rooms(index + 1)
}

== function process_bookings(index: int) => void ==
{ if index >= LEN(guests):
    ~ return
- else:
    ~ temp guest: Guest = guests[index]
    { if rooms_available():
        ~ take_reservation(guest)
    - else:
        ~ take_overbook_or_decline(guest)
    }
    ~ process_bookings(index + 1)
}

== function rooms_available() => bool ==
{ if count_free_rooms(0) > 0:
    ~ return true
- else:
    ~ return false
}

== function count_free_rooms(index: int) => int ==
{ if index >= LEN(rooms):
    ~ return 0
- else:
    ~ temp room: Room = rooms[index]
    { if room.assigned_guest == "vacant" and room.clean:
        ~ return 1 + count_free_rooms(index + 1)
    - else:
        ~ return count_free_rooms(index + 1)
    }
}

== function take_reservation(guest: Guest) => void ==
~ temp preferred: int = find_preferred_room(guest.preferred_class, 0, guest.budget)
~ temp fallback: int = find_any_clean_room(0, guest.budget)
{ if preferred == -1:
    { if fallback != -1:
        ~ reserve_confirmed_room(guest, fallback, true)
    - else:
        ~ take_overbook_or_decline(guest)
    }
- else:
    ~ reserve_confirmed_room(guest, preferred, false)
}

== function find_preferred_room(preference: string, room_index: int, budget: int) => int ==
{ if room_index >= LEN(rooms):
    ~ return -1
- else:
    ~ temp room: Room = rooms[room_index]
    { if room.clean and room.assigned_guest == "vacant" and room.room_class == preference and room.rate <= budget:
        ~ return room_index
    - else:
        ~ return find_preferred_room(preference, room_index + 1, budget)
    }
}

== function find_any_clean_room(room_index: int, budget: int) => int ==
{ if room_index >= LEN(rooms):
    ~ return -1
- else:
    ~ temp room: Room = rooms[room_index]
    { if room.clean and room.assigned_guest == "vacant" and room.rate <= budget:
        ~ return room_index
    - else:
        ~ return find_any_clean_room(room_index + 1, budget)
    }
}

== function reserve_confirmed_room(guest: Guest, room_index: int, was_fallback: bool) => void ==
~ temp room: Room = rooms[room_index]
~ temp room_cost: int = room.rate
{ if was_fallback:
    ~ room_cost = room_cost + 2
}
~ temp record_index: int = reservation_count
The clerk accepts {guest.name} for {room.name}.
~ rooms[room_index].assigned_guest = guest.name
~ reservations[record_index].guest_name = guest.name
~ reservations[record_index].room_name = room.name
~ reservations[record_index].paid = room_cost
~ reservations[record_index].refund = 0
~ reservations[record_index].status = "confirmed"
~ reservations[record_index].canceled = guest.cancels
~ reservation_count = reservation_count + 1
~ treasury = treasury + room_cost
{ if guest.cancels:
    The reservation includes a refundable hold and may reduce at night.
- else:
    Full payment expected at dawn.
}

== function take_overbook_or_decline(guest: Guest) => void ==
~ temp overbook_count: int = count_overbookings(0)
~ temp willing_overpay: int = guest.budget - overbook_price
{ if reservation_count >= LEN(reservations):
    The ledger is full and {guest.name} cannot be recorded.
- else:
    { if overbook_count < overbook_limit and willing_overpay >= guest.max_overpay:
        ~ temp record_index: int = reservation_count
        The house opens a temporary cot for {guest.name} due full occupancy.
        ~ reservations[record_index].guest_name = guest.name
        ~ reservations[record_index].room_name = "temporary cot"
        ~ reservations[record_index].paid = overbook_price
        ~ reservations[record_index].refund = 0
        ~ reservations[record_index].status = "overbooked"
        ~ reservations[record_index].canceled = guest.cancels
        ~ reservation_count = reservation_count + 1
        ~ treasury = treasury + overbook_price
    - else:
        The inn is closed for {guest.name}; deposit not accepted.
        ~ temp record_index: int = reservation_count
        ~ reservations[record_index].guest_name = guest.name
        ~ reservations[record_index].room_name = "declined"
        ~ reservations[record_index].paid = 0
        ~ reservations[record_index].refund = 0
        ~ reservations[record_index].status = "declined"
        ~ reservations[record_index].canceled = false
        ~ reservation_count = reservation_count + 1
    }
}

== function count_overbookings(index: int) => int ==
{ if index >= reservation_count:
    ~ return 0
- else:
    { if reservations[index].status == "overbooked":
        ~ return 1 + count_overbookings(index + 1)
    - else:
        ~ return count_overbookings(index + 1)
    }
}

== function apply_cancellations() => void ==
Guest updates arrive before sunrise.
~ apply_cancel(0)

== function apply_cancel(index: int) => void ==
{ if index >= reservation_count:
    ~ return
- else:
    { if reservations[index].status == "confirmed" and reservations[index].canceled:
        The clerk marks {reservations[index].guest_name} as canceled.
        ~ temp half_refund: int = reservations[index].paid / 2
        ~ reservations[index].refund = half_refund
        ~ reservations[index].status = "canceled"
        ~ refunded_total = refunded_total + half_refund
        ~ treasury = treasury - half_refund
        ~ vacancy_tokens = vacancy_tokens + 1
        ~ free_room_for(reservations[index].guest_name, 0)
    }
    ~ apply_cancel(index + 1)
}

== function free_room_for(guest_name: string, room_index: int) => void ==
{ if room_index >= LEN(rooms):
    ~ return
- else:
    ~ temp room: Room = rooms[room_index]
    { if room.assigned_guest == guest_name:
        ~ rooms[room_index].assigned_guest = "vacant"
        ~ room_clean_up(room_index)
    - else:
        ~ free_room_for(guest_name, room_index + 1)
    }
}

== function room_clean_up(room_index: int) => void ==
~ rooms[room_index].clean = true
The room is cleared and can be reassigned.

== function apply_overbook_resolution() => void ==
After cancellations, the overbooked guests are reviewed.
~ upgrade_overbooked(0)

== function upgrade_overbooked(index: int) => void ==
{ if index >= reservation_count:
    ~ return
- else:
    { if reservations[index].status == "overbooked" and vacancy_tokens > 0:
        ~ temp spare_room: int = pick_vacant_room(0)
        { if spare_room != -1:
            ~ temp room: Room = rooms[spare_room]
            The cot guest {reservations[index].guest_name} is now assigned to {room.name}.
            ~ reservations[index].room_name = room.name
            ~ reservations[index].status = "upgraded"
            ~ reservations[index].refund = reservations[index].refund + overbook_refund_bonus
            ~ refunded_total = refunded_total + overbook_refund_bonus
            ~ treasury = treasury - overbook_refund_bonus
            ~ rooms[spare_room].assigned_guest = reservations[index].guest_name
            ~ vacancy_tokens = vacancy_tokens - 1
        - else:
            No vacancy can be found for {reservations[index].guest_name} yet.
            ~ reservations[index].status = "pending-overbooked"
        }
    }
    ~ upgrade_overbooked(index + 1)
}

== function pick_vacant_room(room_index: int) => int ==
{ if room_index >= LEN(rooms):
    ~ return -1
- else:
    ~ temp room: Room = rooms[room_index]
    { if room.assigned_guest == "vacant" and room.clean:
        ~ return room_index
    - else:
        ~ return pick_vacant_room(room_index + 1)
    }
}

== function final_report() => void ==
Tonight's outcomes are final.
~ print_reservations(0)
~ print_money()

== function print_reservations(index: int) => void ==
{ if index >= reservation_count:
    ~ return
- else:
    { if reservations[index].status == "confirmed":
        {reservations[index].guest_name} / {reservations[index].room_name}: confirmed, paid {reservations[index].paid}, refunded {reservations[index].refund}
    - else:
        { if reservations[index].status == "overbooked":
            {reservations[index].guest_name} / {reservations[index].room_name}: overbooked, paid {reservations[index].paid}, refund pending {reservations[index].refund}
        - else:
            { if reservations[index].status == "upgraded":
                {reservations[index].guest_name} / {reservations[index].room_name}: overbook resolution, paid {reservations[index].paid}, extra refund {reservations[index].refund}
            - else:
                { if reservations[index].status == "pending-overbooked":
                    {reservations[index].guest_name} / {reservations[index].room_name}: pending cot, paid {reservations[index].paid}, refund pending {reservations[index].refund}
                - else:
                    { if reservations[index].status == "canceled":
                        {reservations[index].guest_name} / {reservations[index].room_name}: canceled, paid {reservations[index].paid}, refund {reservations[index].refund}
                    - else:
                        { if reservations[index].status == "declined":
                            {reservations[index].guest_name}: declined (insufficient room rights and no overbook room).
                        - else:
                            {reservations[index].guest_name} / {reservations[index].room_name}: {reservations[index].status}
                        }
                    }
                }
            }
        }
    }
    ~ print_reservations(index + 1)
}

== function print_money() => void ==
The clerk closes books.
~ temp gross: int = treasury + refunded_total
Gross intake: {gross}
Net on-site: {treasury}
Refunds issued: {refunded_total}
