=== module data ===
ENUM State { Idle Busy Done }
ENUM Tone {
Calm
Sharp
}
VAR state: State
VAR states: State[] = [State.Idle, State.Busy]
STRUCT Actor {
state: State
history: State[]
mood: Tone
}
VAR actor: Actor = %Actor{ state: State.Busy, history: [State.Idle], mood: Tone.Sharp }
CONST DEFAULT_STATE: State = State.Done
CONST DEFAULT_TONE: Tone = Tone.Calm

== function echo(value: State) => State ==
~ return value

== function tone_name(value: Tone) => string ==
{ switch value:
- Tone.Calm:
    ~ return "calm"
- else:
    ~ return "sharp"
}

=== module game ===
FROM data IMPORT State, state, states, actor, DEFAULT_STATE, DEFAULT_TONE, echo, tone_name

== main ==
{data::state}|{data::state == data::State.Idle}|{data::states[1]}|{data::actor.state}|{data::actor.history[0]}|{data::echo(data::DEFAULT_STATE)}|{data::tone_name(data::DEFAULT_TONE)}|{data::actor.mood}
~ data::state = data::State.Done
|{data::state != data::State.Busy}
