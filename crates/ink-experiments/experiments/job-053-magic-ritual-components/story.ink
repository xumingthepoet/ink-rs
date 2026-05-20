=== module game ===

VAR required_sequence: string[] = ["Aether", "Salt", "Quartz", "Heartfire"]
VAR required_power: int = 14
VAR required_stability: int = 7
VAR misaligned_sequence: string[] = ["Salt", "Aether", "Ash", "Heartfire"]
VAR aligned_sequence: string[] = ["Aether", "Salt", "Quartz", "Heartfire"]

== main ==
Ritual circle calibration report.
~ print_trial("Trial A", 0)
~ print_trial("Trial B", 1)
-> DONE

== function print_trial(label: string, trial: int) => void ==
Trial {label}
~ print_component_order(trial, 0)
~ temp mismatches: int = sequence_mismatches(trial, 0)
~ temp power_total: int = total_power(trial, 0)
~ temp power_delta: int = power_total - required_power
~ temp stability_total: int = total_stability(trial, 0)
~ temp stability_delta: int = stability_total - required_stability
~ temp risk: int = ritual_backlash(trial)
Sequence check: {mismatches} mismatch(s).
Target power: {required_power}, actual {power_total}.
{ if power_delta >= 0:
    Power offset: +{power_delta}
- else:
    Power offset: {power_delta}
}
Target stability: {required_stability}, actual {stability_total}.
{ if stability_delta >= 0:
    Stability offset: +{stability_delta}
- else:
    Stability offset: {stability_delta}
}
Backlash risk score: {risk}.
Ritual result:
{ ritual_result(trial)}

== function print_component_order(trial: int, index: int) => void ==
{ if index >= LEN(required_sequence):
    ~ return
- else:
    ~ temp position: int = index + 1
    ~ temp name: string = trial_component(trial, index)
    ~ temp role: string = component_role(name)
    Slot {position}: {name} ({role})
    ~ print_component_order(trial, index + 1)
}

== function sequence_mismatches(trial: int, index: int) => int ==
{ if index >= LEN(required_sequence):
    ~ return 0
- else:
    ~ temp expected: string = required_sequence[index]
    ~ temp actual: string = trial_component(trial, index)
    { if expected == actual:
        ~ return sequence_mismatches(trial, index + 1)
    - else:
        ~ return 1 + sequence_mismatches(trial, index + 1)
    }
}

== function total_power(trial: int, index: int) => int ==
{ if index >= LEN(required_sequence):
    ~ return 0
- else:
    ~ temp name: string = trial_component(trial, index)
    ~ temp power: int = component_power(name)
    ~ return power + total_power(trial, index + 1)
}

== function total_stability(trial: int, index: int) => int ==
{ if index >= LEN(required_sequence):
    ~ return 0
- else:
    ~ temp name: string = trial_component(trial, index)
    ~ temp stability: int = component_stability(name)
    ~ return stability + total_stability(trial, index + 1)
}

== function ritual_backlash(trial: int) => int ==
~ temp mismatch_penalty: int = sequence_mismatches(trial, 0) * 3
~ temp power_penalty: int = abs_int(total_power(trial, 0) - required_power)
~ temp stability_penalty: int = abs_int(total_stability(trial, 0) - required_stability)
~ return mismatch_penalty + power_penalty + stability_penalty

== function ritual_result(trial: int) => string ==
~ temp mismatches: int = sequence_mismatches(trial, 0)
~ temp risk: int = ritual_backlash(trial)
{ if mismatches > 0:
    { if risk >= 10:
        ~ return "Critical breach. Components fire out of phase and the circle folds in on itself."
    - else:
        ~ return "Unstable circle. The rite sputters but can be repeated after reset."
    }
- else:
    { if risk <= 2:
        ~ return "Clean binding. Focus holds and the ritual anchors properly."
    - else:
        ~ return "Balanced but volatile. The effect works, but residue lingers."
    }
}

== function trial_component(trial: int, index: int) => string ==
{ if trial == 0:
    ~ return misaligned_sequence[index]
- else:
    ~ return aligned_sequence[index]
}

== function component_power(name: string) => int ==
{ if name == "Aether":
    ~ return 5
- else:
    { if name == "Salt":
        ~ return 3
    - else:
        { if name == "Quartz":
            ~ return 2
        - else:
            { if name == "Heartfire":
                ~ return 4
            - else:
                ~ return 5
            }
        }
    }
}

== function component_stability(name: string) => int ==
{ if name == "Aether":
    ~ return 3
- else:
    { if name == "Salt":
        ~ return 1
    - else:
        { if name == "Quartz":
            ~ return 2
        - else:
            { if name == "Heartfire":
                ~ return 1
            - else:
                ~ return -5
            }
        }
    }
}

== function component_role(name: string) => string ==
{ if name == "Aether":
    ~ return "opener"
- else:
    { if name == "Salt":
        ~ return "balancer"
    - else:
        { if name == "Quartz":
            ~ return "focuser"
        - else:
            { if name == "Heartfire":
                ~ return "seal"
            - else:
                ~ return "wild catalyst"
            }
        }
    }
}

== function abs_int(value: int) => int ==
{ if value < 0:
    ~ return -value
- else:
    ~ return value
}
