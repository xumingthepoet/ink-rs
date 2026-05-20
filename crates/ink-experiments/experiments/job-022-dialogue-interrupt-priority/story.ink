=== module game ===
STRUCT SceneInterrupt {
    name: string
    priority: int
    once_only: bool
    active: bool
    valid: bool
    delivered: bool
}

VAR interrupt_queue: SceneInterrupt[] = [
    %SceneInterrupt{name: "Hero cut-in", priority: 95, once_only: true, active: false, valid: false, delivered: false},
    %SceneInterrupt{name: "Scout signal", priority: 70, once_only: true, active: false, valid: false, delivered: false},
    %SceneInterrupt{name: "Alarm chime", priority: 70, once_only: false, active: false, valid: false, delivered: false},
    %SceneInterrupt{name: "Emergency broadcast", priority: 120, once_only: false, active: false, valid: false, delivered: false}
]

== main ==
Scene interrupt priority queue.
~ run_phase("Opening tension", true, true, false)
~ run_phase("Hero muted", false, true, false)
~ run_phase("Scout silent", false, false, false)
~ run_phase("Emergency override", false, false, true)
-> DONE

== function run_phase(phase_label: string, hero: bool, scout: bool, emergency: bool) => void ==
{phase_label}
~ arm_interrupts(hero, scout, emergency)
~ print_interrupt_queue("Queued")
~ resolve_interrupt()

== function arm_interrupts(hero_available: bool, scout_available: bool, emergency_available: bool) => void ==
~ interrupt_queue[0].active = true
~ interrupt_queue[1].active = true
~ interrupt_queue[2].active = true
~ interrupt_queue[3].active = emergency_available
~ interrupt_queue[0].valid = hero_available
~ interrupt_queue[1].valid = scout_available
~ interrupt_queue[2].valid = true
~ interrupt_queue[3].valid = emergency_available

== function resolve_interrupt() => void ==
~ temp selected: int = select_interrupt(0)
{ if selected == -1:
    No interrupt was delivered.
- else:
    {interrupt_queue[selected].name} interrupts with priority {interrupt_queue[selected].priority}.
    {if interrupt_queue[selected].once_only: "(once-only consumed)" else: "(kept available)"}
    { if interrupt_queue[selected].once_only:
        ~ interrupt_queue[selected].delivered = true
    }
    ~ interrupt_queue[selected].active = false
}

== function select_interrupt(index: int) => int ==
{ if index >= LEN(interrupt_queue):
    ~ return -1
- else:
    { if is_interrupt_ready(index):
        ~ return find_best_interrupt(index, index + 1)
    - else:
        ~ return select_interrupt(index + 1)
    }
}

== function is_interrupt_ready(index: int) => bool ==
~ temp entry: SceneInterrupt = interrupt_queue[index]
~ return entry.active && entry.valid && (!entry.once_only || !entry.delivered)

== function find_best_interrupt(best: int, index: int) => int ==
{ if index >= LEN(interrupt_queue):
    ~ return best
- else:
    { if !is_interrupt_ready(index):
        ~ return find_best_interrupt(best, index + 1)
    - else:
        { if interrupt_queue[index].priority > interrupt_queue[best].priority:
            ~ return find_best_interrupt(index, index + 1)
        - else:
            ~ return find_best_interrupt(best, index + 1)
        }
    }
}

== function print_interrupt_queue(label: string) => void ==
Interrupts queued ({label}):
~ print_interrupt_rows(0)

== function print_interrupt_rows(index: int) => void ==
{ if index < LEN(interrupt_queue):
    ~ temp interrupt: SceneInterrupt = interrupt_queue[index]
    {interrupt.name} p={interrupt.priority} active={interrupt.active} valid={interrupt.valid} once={interrupt.once_only} delivered={interrupt.delivered}
    ~ print_interrupt_rows(index + 1)
}
