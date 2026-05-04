=== module game ===
ENUM State { Idle Busy Done }
VAR state: State
VAR states: State[] = [State.Idle, State.Busy]
STRUCT Actor {
state: State
history: State[]
}
VAR actor: Actor = { state: State.Busy, history: [State.Idle] }
CONST DEFAULT_STATE: State = State.Done

== function echo(value: State) => State ==
~ return value

== main ==
{state}|{state == State.Idle}|{states[1]}|{actor.state}|{actor.history[0]}|{echo(DEFAULT_STATE)}
~ state = State.Done
|{state != State.Busy}
-> DONE
