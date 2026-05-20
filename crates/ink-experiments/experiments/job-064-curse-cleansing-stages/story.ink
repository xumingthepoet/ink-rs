=== module game ===

STRUCT CurseLayer {
    name: string
    taint: int
    resistance: int
    status: string
}

STRUCT CleansingRite {
    layer: string
    method: string
    focus: int
    vigor: int
    backlash_risk: int
}

VAR layers: CurseLayer[] = [
    %CurseLayer{name: "Stonebound Knot", taint: 11, resistance: 8, status: "violent"},
    %CurseLayer{name: "Salt-Wind Loop", taint: 9, resistance: 6, status: "bound"},
    %CurseLayer{name: "Mirror Throat", taint: 13, resistance: 10, status: "violent"},
    %CurseLayer{name: "Bloodline Sigil", taint: 7, resistance: 5, status: "bound"},
    %CurseLayer{name: "Night-Water Pool", taint: 10, resistance: 7, status: "bound"}
]

VAR rites: CleansingRite[] = [
    %CleansingRite{
        layer: "Stonebound Knot",
        method: "stabilize",
        focus: 9,
        vigor: 3,
        backlash_risk: 4
    },
    %CleansingRite{
        layer: "Salt-Wind Loop",
        method: "seal",
        focus: 7,
        vigor: 4,
        backlash_risk: 5
    },
    %CleansingRite{
        layer: "Mirror Throat",
        method: "drain",
        focus: 8,
        vigor: 2,
        backlash_risk: 6
    },
    %CleansingRite{
        layer: "Bloodline Sigil",
        method: "bind",
        focus: 6,
        vigor: 5,
        backlash_risk: 3
    },
    %CleansingRite{
        layer: "Salt-Wind Loop",
        method: "stabilize",
        focus: 10,
        vigor: 1,
        backlash_risk: 7
    },
    %CleansingRite{
        layer: "Night-Water Pool",
        method: "drain",
        focus: 7,
        vigor: 3,
        backlash_risk: 4
    },
    %CleansingRite{
        layer: "Mirror Throat",
        method: "bind",
        focus: 11,
        vigor: 2,
        backlash_risk: 5
    }
]

VAR backlash_meter: int = 0
VAR full_rites: int = 0
VAR partial_rites: int = 0
VAR failed_rites: int = 0

== main ==
Curse cleansing cycle starts.
~ print_layers("Initial state")
~ execute_rites(0)
~ print_layers("After rites")
~ print_final_status()
-> DONE

== function print_layers(label: string) => void ==
Curse layers: {label}
~ print_layer_rows(0)

== function print_layer_rows(index: int) => void ==
{ if index >= LEN(layers):
    ~ return
- else:
    ~ temp layer: CurseLayer = layers[index]
    {layer.name}: taint {layer.taint}, resistance {layer.resistance}, state {layer.status}
    ~ print_layer_rows(index + 1)
}

== function execute_rites(index: int) => void ==
{ if index >= LEN(rites):
    ~ return
- else:
    ~ temp rite: CleansingRite = rites[index]
    Rite {index + 1} on {rite.layer}
    {rite.method} with focus {rite.focus} and vigor {rite.vigor}.
    ~ temp layer_index: int = find_layer(rite.layer, 0)
    { if layer_index == -1:
        No active layer named {rite.layer} for this rite.
    - else:
        ~ apply_rite(layer_index, rite)
    }
    ~ execute_rites(index + 1)
}

== function find_layer(name: string, index: int) => int ==
{ if index >= LEN(layers):
    ~ return -1
- else:
    { if layers[index].name == name:
        ~ return index
    - else:
        ~ return find_layer(name, index + 1)
    }
}

== function apply_rite(layer_index: int, rite: CleansingRite) => void ==
~ temp layer: CurseLayer = layers[layer_index]
{ if layer.taint <= 0:
    {layer.name} is already sealed.
- else:
    ~ temp power: int = rite.focus + rite.vigor
    ~ temp full_threshold: int = layer.taint + layer.resistance
    ~ temp partial_threshold: int = full_threshold / 2
    ~ temp taint_delta: int = 0
    ~ temp next_taint: int = layer.taint

    { if power >= full_threshold:
        {layer.name} fully calms.
        ~ full_rites = full_rites + 1
        ~ next_taint = 0
    - else:
        { if power >= partial_threshold:
            {layer.name} loosens from the binding but does not clear.
            ~ partial_rites = partial_rites + 1
            ~ taint_delta = power / 2
            ~ next_taint = layer.taint - taint_delta
        - else:
            {layer.name} resists and rebounds.
            ~ failed_rites = failed_rites + 1
            ~ taint_delta = -1
            ~ next_taint = layer.taint + 1
            ~ temp backlash: int = power + rite.backlash_risk
            ~ backlash_meter = backlash_meter + backlash
            { if backlash_meter >= 8:
                ~ emit_backlash(layer_index)
                ~ backlash_meter = 0
            - else:
                Backlash pressure rises to {backlash_meter}.
            }
        }
    }

    { if next_taint < 0:
        ~ next_taint = 0
    }
    ~ layers[layer_index].taint = next_taint
    ~ layers[layer_index].status = layer_state(next_taint)
    { if layer_index == 2 and next_taint == 0:
        Final anchor at {layer.name} collapses.
        ~ print_layers("Anchor cleared")
    }
}
}

== function emit_backlash(source: int) => void ==
Backlash pulse. Adjacent bindings destabilize.
~ spread_backlash(source, 0)
}

== function spread_backlash(source: int, index: int) => void ==
{ if index >= LEN(layers):
    ~ return
- else:
    { if index != source:
        ~ temp taint: int = layers[index].taint + 2
        { if taint < 0:
            ~ taint = 0
        }
        ~ layers[index].taint = taint
        ~ layers[index].status = layer_state(taint)
    }
    ~ spread_backlash(source, index + 1)
}

== function layer_state(taint: int) => string ==
{ if taint <= 0:
    ~ return "sealed"
- else:
    { if taint <= 4:
        ~ return "thin"
    - else:
        { if taint <= 9:
            ~ return "bound"
        - else:
            ~ return "wild"
        }
    }
}

== function print_final_status() => void ==
Final cleansing review.
~ temp sealed: int = layer_count("sealed", 0)
~ temp thin: int = layer_count("thin", 0)
~ temp bound: int = layer_count("bound", 0)
~ temp wild: int = layer_count("wild", 0)
Cleanse outcomes:
Full rites: {full_rites}
Partial rites: {partial_rites}
Failed rites: {failed_rites}
Layer states:
Sealed {sealed}
Thin {thin}
Bound {bound}
Wild {wild}
Backlash meter: {backlash_meter}
~ temp final_status: string = final_status_text(sealed, LEN(layers))
Result: {final_status}
~ temp strongest: string = strongest_layer()
Most volatile layer: {strongest}
~ print_layers("Final state")
}

== function layer_count(target: string, index: int) => int ==
{ if index >= LEN(layers):
    ~ return 0
- else:
    { if layers[index].status == target:
        ~ return 1 + layer_count(target, index + 1)
    - else:
        ~ return layer_count(target, index + 1)
    }
}

== function final_status_text(sealed: int, total: int) => string ==
{ if sealed == total:
    ~ return "Cleansing complete. Curse removed."
- else:
    { if sealed >= total / 2:
        ~ return "Cleansing mostly done, with residual bindings."
    - else:
        { if sealed > 0:
            ~ return "Cleansing partial. Several bindings remain dangerous."
        - else:
            ~ return "Cleansing failed. Curse remains dominant."
        }
    }
}

== function strongest_layer() => string ==
~ temp highest_taint: int = layers[0].taint
~ temp strongest_name: string = layers[0].name
~ temp strongest: string = find_strongest_taint(highest_taint, strongest_name, 1)
~ return strongest
}

== function find_strongest_taint(highest: int, name: string, index: int) => string ==
{ if index >= LEN(layers):
    ~ return name
- else:
    ~ temp layer: CurseLayer = layers[index]
    { if layer.taint > highest:
        ~ return find_strongest_taint(layer.taint, layer.name, index + 1)
    - else:
        ~ return find_strongest_taint(highest, name, index + 1)
    }
}
