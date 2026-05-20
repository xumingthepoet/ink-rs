=== module rivalry ===

STRUCT RivalPhase {
    phase: string
    player_gain: int
    rival_gain: int
    player_resource_gain: int
    rival_resource_gain: int
    player_setback: int
    rival_setback: int
}

VAR phases: RivalPhase[] = [
    %RivalPhase{
        phase: "Opening March",
        player_gain: 3,
        rival_gain: 2,
        player_resource_gain: 5,
        rival_resource_gain: 4,
        player_setback: 1,
        rival_setback: 0
    },
    %RivalPhase{
        phase: "Border Survey",
        player_gain: 4,
        rival_gain: 5,
        player_resource_gain: 3,
        rival_resource_gain: 5,
        player_setback: 3,
        rival_setback: 2
    },
    %RivalPhase{
        phase: "Ritual Relay",
        player_gain: 6,
        rival_gain: 5,
        player_resource_gain: 2,
        rival_resource_gain: 4,
        player_setback: 2,
        rival_setback: 1
    },
    %RivalPhase{
        phase: "Diplomat Route",
        player_gain: 2,
        rival_gain: 4,
        player_resource_gain: 4,
        rival_resource_gain: 6,
        player_setback: 1,
        rival_setback: 3
    },
    %RivalPhase{
        phase: "Final Assembly",
        player_gain: 5,
        rival_gain: 6,
        player_resource_gain: 3,
        rival_resource_gain: 5,
        player_setback: 2,
        rival_setback: 2
    }
]

VAR player_progress: int = 0
VAR rival_progress: int = 0
VAR player_resources: int = 18
VAR rival_resources: int = 16
VAR player_resource_gain_total: int = 0
VAR rival_resource_gain_total: int = 0
VAR player_setbacks: int = 0
VAR rival_setbacks: int = 0
VAR player_momentum: int = 0
VAR rival_momentum: int = 0

== main ==
Competing parties prepare their rival quest logs.
~ execute_phases(0)
~ print_competition_result()
-> DONE

== function execute_phases(index: int) => void ==
{ if index >= LEN(phases):
    ~ return
- else:
    ~ temp phase: RivalPhase = phases[index]
    ~ temp player_gain: int = phase.player_gain
    ~ temp rival_gain: int = phase.rival_gain

    Phase {index + 1}: {phase.phase}
    Starting momentum: player {player_progress}, rival {rival_progress}

    ~ player_resource_gain_total = player_resource_gain_total + phase.player_resource_gain
    ~ rival_resource_gain_total = rival_resource_gain_total + phase.rival_resource_gain

    { if player_resources + phase.player_resource_gain < phase.player_setback:
        ~ player_gain = player_gain - 1
        { if player_gain < 0:
            ~ player_gain = 0
        }
        ~ player_setbacks = player_setbacks + 1
        { if player_gain < phase.player_gain:
            Player stalls from depleted logistics.
        }
    }
    { if rival_resources + phase.rival_resource_gain < phase.rival_setback:
        ~ rival_gain = rival_gain - 1
        { if rival_gain < 0:
            ~ rival_gain = 0
        }
        ~ rival_setbacks = rival_setbacks + 1
        { if rival_gain < phase.rival_gain:
            Rival stumbles from poor supplies.
        }
    }

    ~ player_resources = apply_loss(player_resources + phase.player_resource_gain, phase.player_setback)
    ~ rival_resources = apply_loss(rival_resources + phase.rival_resource_gain, phase.rival_setback)
    ~ player_progress = player_progress + player_gain
    ~ rival_progress = rival_progress + rival_gain
    ~ player_momentum = player_momentum + (player_gain + 1)
    ~ rival_momentum = rival_momentum + rival_gain
    ~ print_phase_score(phase, player_gain, rival_gain)

    ~ execute_phases(index + 1)
}

== function apply_loss(resource: int, loss: int) => int ==
{ if resource >= loss:
    ~ return resource - loss
- else:
    ~ return 0
}

== function print_phase_score(phase: RivalPhase, player_gain: int, rival_gain: int) => void ==
Phase result:
Player gains {player_gain} progress.
Rival gains {rival_gain} progress.
Net change:
Player resources now {player_resources}.
Rival resources now {rival_resources}.
Resources gained total:
Player {player_resource_gain_total}, Rival {rival_resource_gain_total}.

== function print_competition_result() => void ==
Final momentum count:
Player momentum: {player_momentum}
Rival momentum: {rival_momentum}
Player progress: {player_progress}
Rival progress: {rival_progress}
Total resources:
Player {player_resources}
Rival {rival_resources}
Setbacks faced:
Player {player_setbacks}, Rival {rival_setbacks}
Competition outcome:
~ print_outcome()

== function print_outcome() => void ==
~ temp gap: int = player_progress - rival_progress
{ if gap > 0:
    Player party leads by {gap} progress.
- else:
    { if gap < 0:
        Rival party leads by {0 - gap} progress.
    - else:
        Both parties remain tied.
    }
}
