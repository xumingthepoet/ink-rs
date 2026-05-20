=== module game ===

VAR hall_haunt: int = 12
VAR gallery_haunt: int = 9
VAR archive_haunt: int = 14
VAR cellar_haunt: int = 6

VAR room_targets: int[] = [1, 2, 0, 3, 1, 0, 2]
VAR action_plan: int[] = [1, 0, 1, 2, 1, 2, 1]

VAR total_spread: int = 0
VAR successful_cleanses: int = 0
VAR failed_cleanses: int = 0
VAR manifestation_state: string = ""

== main ==
Night watch begins across four rooms.
~ print_initial_state()
~ run_round(0)
~ finalize_watch()
-> DONE

== function print_initial_state() => void ==
Initial haunt levels:
Great Hall: {hall_haunt}
Gallery: {gallery_haunt}
Archive: {archive_haunt}
Cellar: {cellar_haunt}

== function run_round(step: int) => void ==
{ if step >= LEN(room_targets):
    Watch sequence complete.
- else:
    ~ temp room: int = room_targets[step]
    ~ temp action: int = action_plan[step]
    ~ print_round_header(step + 1, room, action)
    ~ apply_room_action(room, action)
    ~ spread_fear_between_rooms()
    ~ print_current_levels()
    ~ run_round(step + 1)
}

== function print_round_header(step: int, room: int, action: int) => void ==
Round {step}: {room_name(room)}
{ if action == 1:
    Planned cleansing action: Cleanse
- else:
    { if action == 2:
        Planned cleansing action: Contain surge
    - else:
        Planned cleansing action: Observe only
    }
}

== function apply_room_action(room: int, action: int) => void ==
~ temp before: int = room_haunt(room)
{ if action == 1:
    { if before <= 0:
        Cleanse attempt fails: spirit pressure already absent.
        ~ failed_cleanses = failed_cleanses + 1
    - else:
        ~ temp after: int = before - 4
        { if after < 0:
            ~ after = 0
        }
        ~ set_room_haunt(room, after)
        ~ successful_cleanses = successful_cleanses + 1
        { room_name(room) } cleansing reduces haunt by 4 to {after}.
    }
- else:
    { if action == 2:
        ~ temp after: int = before - 1
        { if after < 1:
            ~ after = 1
        }
        ~ set_room_haunt(room, after)
        ~ successful_cleanses = successful_cleanses + 1
        { room_name(room) } is stabilized for this round at {after}.
    - else:
        No intervention in {room_name(room)}.
    }
}

== function spread_fear_between_rooms() => void ==
~ temp hall_add: int = 0
~ temp gallery_add: int = 0
~ temp archive_add: int = 0
~ temp cellar_add: int = 0

{ if hall_haunt >= 8:
    ~ gallery_add = gallery_add + 1
}

{ if gallery_haunt >= 8:
    ~ hall_add = hall_add + 1
    ~ archive_add = archive_add + 1
}

{ if archive_haunt >= 8:
    ~ gallery_add = gallery_add + 1
    ~ cellar_add = cellar_add + 1
}

{ if cellar_haunt >= 8:
    ~ archive_add = archive_add + 1
}

~ hall_haunt = hall_haunt + hall_add
~ gallery_haunt = gallery_haunt + gallery_add
~ archive_haunt = archive_haunt + archive_add
~ cellar_haunt = cellar_haunt + cellar_add
~ total_spread = total_spread + hall_add + gallery_add + archive_add + cellar_add

== function print_current_levels() => void ==
Spread resolved. Current pressure:
Great Hall: {hall_haunt}
Gallery: {gallery_haunt}
Archive: {archive_haunt}
Cellar: {cellar_haunt}

== function finalize_watch() => void ==
~ evaluate_manifestation()
Watch complete.
Manifestation status: {manifestation_state}
Cleanse attempts: {successful_cleanses}
Failed cleanse attempts: {failed_cleanses}
Total spread events: {total_spread}
Final pressure:
Great Hall: {hall_haunt}
Gallery: {gallery_haunt}
Archive: {archive_haunt}
Cellar: {cellar_haunt}

== function evaluate_manifestation() => void ==
~ temp peak: int = hall_haunt
{ if gallery_haunt > peak:
    ~ peak = gallery_haunt
}
{ if archive_haunt > peak:
    ~ peak = archive_haunt
}
{ if cellar_haunt > peak:
    ~ peak = cellar_haunt
}
{ if peak >= 16:
    ~ manifestation_state = "Major manifestation breached the ward."
- else:
    { if peak >= 11:
        ~ manifestation_state = "Minor manifestation appears and fades."
    - else:
        { if peak >= 6:
            ~ manifestation_state = "No open manifestation, but tension remains."
        - else:
            ~ manifestation_state = "All haunting pressure remains contained."
        }
    }
}

== function room_haunt(room: int) => int ==
{ if room == 0:
    ~ return hall_haunt
- else:
    { if room == 1:
        ~ return gallery_haunt
    - else:
        { if room == 2:
            ~ return archive_haunt
        - else:
            ~ return cellar_haunt
        }
    }
}

== function set_room_haunt(room: int, value: int) => void ==
{ if room == 0:
    ~ hall_haunt = value
- else:
    { if room == 1:
        ~ gallery_haunt = value
    - else:
        { if room == 2:
            ~ archive_haunt = value
        - else:
            ~ cellar_haunt = value
        }
    }
}

== function room_name(room: int) => string ==
{ if room == 0:
    ~ return "Great Hall"
- else:
    { if room == 1:
        ~ return "Gallery"
    - else:
        { if room == 2:
            ~ return "Archive"
        - else:
            ~ return "Cellar"
        }
    }
}
