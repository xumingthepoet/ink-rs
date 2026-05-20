=== module game ===

VAR current_node: string = "Hearth Terminal"
VAR destination_node: string = "Citadel Core"
VAR route_log: string = "Hearth Terminal"
VAR credits: int = 36
VAR power: int = 14
VAR spine_key: bool = false
VAR hops_taken: int = 0
VAR successful_traversals: int = 0
VAR blocked_links: int = 0

== main ==
Portal relay route simulation.
Current node: {current_node}
Credit reserve: {credits}
Power units: {power}
~ inspect_routes()
~ move_via_portal("Hearth Terminal", "Harbor Relay")
~ audit_locked_link("Harbor Relay", "North Spine")
~ move_via_portal("Harbor Relay", "Cipher Bay")
~ move_via_portal("Cipher Bay", "Harbor Relay")
~ move_via_portal("Harbor Relay", "North Spine")
~ move_via_portal("North Spine", "Citadel Gate")
~ move_via_portal("Citadel Gate", "Citadel Core")
~ destination_report()
-> DONE

== function inspect_routes() => void ==
Portal network map:
- From Hearth Terminal to Harbor Relay: {link_description("Hearth Terminal", "Harbor Relay")}
- From Harbor Relay to Cipher Bay: {link_description("Harbor Relay", "Cipher Bay")}
- From Harbor Relay to North Spine: {link_description("Harbor Relay", "North Spine")}
- From Cipher Bay to Harbor Relay: {link_description("Cipher Bay", "Harbor Relay")}
- From North Spine to Citadel Gate: {link_description("North Spine", "Citadel Gate")}
- From Citadel Gate to Citadel Core: {link_description("Citadel Gate", "Citadel Core")}

== function link_description(origin: string, destination: string) => string ==
~ temp cost: int = portal_cost(origin, destination)
~ temp toll: int = portal_toll(origin, destination)
~ temp locked: bool = portal_locked(origin, destination)
{ if cost <= 0:
    route unavailable.
- else:
    { if locked:
        Locked link ({cost} cost + {toll} fee), needs key.
    - else:
        { cost + toll } credits.
    }
}

== function portal_cost(origin: string, destination: string) => int ==
~ temp cost: int = 0
{ if origin == "Hearth Terminal":
    { if destination == "Harbor Relay":
        ~ return 3
    - else:
        ~ return 0
    }
- else:
    { if origin == "Harbor Relay":
        { if destination == "Cipher Bay":
            ~ return 2
        - else:
            { if destination == "North Spine":
                ~ return 3
            - else:
                ~ return 0
            }
        }
    - else:
        { if origin == "Cipher Bay":
            { if destination == "Harbor Relay":
                ~ return 1
            - else:
                ~ return 0
            }
        - else:
            { if origin == "North Spine":
                { if destination == "Citadel Gate":
                    ~ return 2
                - else:
                    ~ return 0
                }
            - else:
                { if origin == "Citadel Gate":
                    { if destination == "Citadel Core":
                        ~ return 1
                    - else:
                        ~ return 0
                    }
                - else:
                    ~ return 0
                }
            }
        }
    }
}

== function portal_toll(origin: string, destination: string) => int ==
~ temp toll: int = 0
{ if origin == "Hearth Terminal":
    { if destination == "Harbor Relay":
        ~ return 1
    - else:
        ~ return 0
    }
- else:
    { if origin == "North Spine" && destination == "Citadel Gate":
        ~ return 1
    - else:
        ~ return 0
    }
}

== function portal_locked(origin: string, destination: string) => bool ==
~ temp locked: bool = false
{ if origin == "Harbor Relay" && destination == "North Spine":
    ~ return !spine_key
- else:
    { if origin == "Hearth Terminal" || origin == "Harbor Relay" || origin == "Cipher Bay" || origin == "North Spine" || origin == "Citadel Gate":
        ~ return false
    - else:
        ~ return true
    }
}

== function move_via_portal(origin: string, destination: string) => void ==
~ temp cost: int = portal_cost(origin, destination)
~ temp toll: int = portal_toll(origin, destination)
~ temp locked: bool = portal_locked(origin, destination)
~ temp needed: int = cost + toll

{ if cost <= 0:
    Link map missing for {origin} to {destination}. Route aborted.
- else:
    ~ successful_traversals = successful_traversals + 1
    { if locked:
        ~ blocked_links = blocked_links + 1
        Link {origin} to {destination} is locked and cannot be crossed.
    - else:
            { if credits < needed:
            ~ blocked_links = blocked_links + 1
            Insufficient credits for {origin} to {destination}. Needed {needed}, have {credits}.
        - else:
            ~ credits = credits - needed
            ~ power = power - cost
            ~ hops_taken = hops_taken + 1
            ~ current_node = destination
            ~ route_log = route_log + " to " + destination
            Traversing {origin} to {destination}: {portal_label(origin, destination)}
            Cost {cost}, fee {toll}. Remaining credits {credits}, power {power}.
            { if destination == "Cipher Bay":
                ~ collect_spine_key()
            - else:
                { if destination == destination_node:
                    Destination node reached.
                - else:
                    Arrived at {destination}.
                }
            }
        }
    }
}

== function portal_label(origin: string, destination: string) => string ==
~ temp note: string = route_note(origin, destination)
~ return note

== function route_note(origin: string, destination: string) => string ==
~ temp note: string = ""
{ if origin == "Hearth Terminal":
    { if destination == "Harbor Relay":
        ~ return "stable ring portal"
    - else:
        ~ return "unregistered"
    }
- else:
    { if origin == "Harbor Relay":
        { if destination == "Cipher Bay":
            ~ return "covert side lane"
        - else:
            { if destination == "North Spine":
                ~ return "locked spine corridor"
            - else:
                ~ return "unregistered"
            }
        }
    - else:
        { if origin == "Cipher Bay":
            { if destination == "Harbor Relay":
                ~ return "return lane"
            - else:
                ~ return "unregistered"
            }
        - else:
            { if origin == "North Spine":
                { if destination == "Citadel Gate":
                    ~ return "fortification bypass"
                - else:
                    ~ return "unregistered"
                }
            - else:
                { if origin == "Citadel Gate":
                    { if destination == "Citadel Core":
                        ~ return "final anchor lock"
                    - else:
                        ~ return "unregistered"
                    }
                - else:
                    ~ return "unregistered"
                }
            }
        }
    }
}

== function audit_locked_link(origin: string, destination: string) => void ==
~ temp locked: bool = portal_locked(origin, destination)
{ if locked:
    ~ blocked_links = blocked_links + 1
    Audit: {origin} to {destination} is locked until key acquired.
- else:
    Audit: {origin} to {destination} is open.
}

== function collect_spine_key() => void ==
~ spine_key = true
Spine key recovered at Cipher Bay.
Spine relay unlocked.

== function destination_report() => void ==
Route log: {route_log}
Destination target: {destination_node}
Hops used: {hops_taken}
Successful traversal checks: {successful_traversals}
Blocked link checks: {blocked_links}
Credits remaining: {credits}
Power remaining: {power}
{ if current_node == destination_node:
    Final state: Arrived at destination and completed routing.
- else:
    Final state: Network routing failed before destination.
}
