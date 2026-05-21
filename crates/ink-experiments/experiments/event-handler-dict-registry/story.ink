=== interface IEventHandler ===
== run(actor_id: int, amount: int) ==
== function label() => string ==
== function child_count(actor_id: int, amount: int) => int ==
== function child_at(actor_id: int, amount: int, index: int) => int ==

=== module game ===
FROM heal_event
FROM ambush_event
FROM treasure_event
FROM morale_event
FROM blessing_event

VAR handlers: Dict<int, interface<IEventHandler>> = %{}
VAR one_shot: Dict<int, bool> = %{}

== main ==
-> register(10, heal_event, false) ->
-> register(20, ambush_event, false) ->
-> register(30, treasure_event, true) ->
-> register(40, morale_event, false) ->
-> print_registry ->
-> trigger(20, 7, 2) ->
-> trigger(30, 7, 1) ->
-> print_registry ->
-> trigger(30, 7, 1) ->
-> unregister(10) ->
-> trigger(10, 7, 4) ->
-> register(10, blessing_event, false) ->
-> print_registry ->
-> trigger(10, 7, 4) ->
-> END

== register(event_id: int, handler: interface<IEventHandler>, once: bool) ==
~ handlers[event_id] = handler
~ one_shot[event_id] = once
{ if once:
    Registered {event_id}: {{handler}::label()} once.
- else:
    Registered {event_id}: {{handler}::label()} repeatable.
}
->->

== unregister(event_id: int) ==
{ if DICT_HAS(handlers, event_id):
    ~ DICT_REMOVE(handlers, event_id)
    ~ DICT_REMOVE(one_shot, event_id)
    Unregistered {event_id}; registry slot removed.
- else:
    Unregister ignored for {event_id}; no registry slot.
}
->->

== trigger(event_id: int, actor_id: int, amount: int) ==
{ if DICT_HAS(handlers, event_id):
    ~ temp handler: interface<IEventHandler> = handlers[event_id]
    ~ temp once: bool = one_shot[event_id]
    Trigger {event_id}: {{handler}::label()} for actor {actor_id}.
    -> {{handler}::run}(actor_id, amount) ->
    { if once:
        -> unregister(event_id) ->
    }
    -> trigger_children(event_id, handler, actor_id, amount, 0)
- else:
    Skipped {event_id}: not active.
}
->->

== trigger_children(event_id: int, handler: interface<IEventHandler>, actor_id: int, amount: int, index: int) ==
~ temp count: int = {handler}::child_count(actor_id, amount)
{ if index >= count:
    ->->
- else:
    ~ temp child_id: int = {handler}::child_at(actor_id, amount, index)
    Event {event_id} emits child {child_id}.
    -> trigger(child_id, actor_id, amount) ->
    -> trigger_children(event_id, handler, actor_id, amount, index + 1)
}

== print_registry ==
Registry has {DICT_SIZE(handlers)} active handlers.
{ for event_id, handler in handlers:
    Handler {event_id}: {{handler}::label()}
}
->->

=== module heal_event implements IEventHandler ===
== function label() => string ==
~ return "heal"

== run(actor_id: int, amount: int) ==
Healed actor {actor_id} for {amount}.
->->

== function child_count(actor_id: int, amount: int) => int ==
~ return 0

== function child_at(actor_id: int, amount: int, index: int) => int ==
~ return -1

=== module ambush_event implements IEventHandler ===
== function label() => string ==
~ return "ambush"

== run(actor_id: int, amount: int) ==
Actor {actor_id} is ambushed by {amount} hidden archers.
->->

== function child_count(actor_id: int, amount: int) => int ==
~ return 2

== function child_at(actor_id: int, amount: int, index: int) => int ==
{ if index == 0:
    ~ return 10
- else:
    ~ return 40
}

=== module treasure_event implements IEventHandler ===
== function label() => string ==
~ return "treasure"

== run(actor_id: int, amount: int) ==
Actor {actor_id} opens a sealed coffer.
->->

== function child_count(actor_id: int, amount: int) => int ==
~ return 1

== function child_at(actor_id: int, amount: int, index: int) => int ==
~ return 40

=== module morale_event implements IEventHandler ===
== function label() => string ==
~ return "morale"

== run(actor_id: int, amount: int) ==
Actor {actor_id} gains morale from the event chain.
->->

== function child_count(actor_id: int, amount: int) => int ==
~ return 0

== function child_at(actor_id: int, amount: int, index: int) => int ==
~ return -1

=== module blessing_event implements IEventHandler ===
== function label() => string ==
~ return "blessing"

== run(actor_id: int, amount: int) ==
Actor {actor_id} receives a replacement blessing for {amount}.
->->

== function child_count(actor_id: int, amount: int) => int ==
~ return 0

== function child_at(actor_id: int, amount: int, index: int) => int ==
~ return -1
