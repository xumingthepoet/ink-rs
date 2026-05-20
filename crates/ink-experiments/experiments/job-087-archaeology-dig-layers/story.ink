=== module game ===

STRUCT Layer {
    name: string
    depth: int
    hardness: int
    fragility: int
    context_cost: int
}

STRUCT Tool {
    name: string
    precision: int
    durability: int
}

STRUCT Artifact {
    title: string
    layer_tag: int
    fragility: int
    cultural_value: int
    found: bool
    preserved: bool
}

VAR dig_layers: Layer[] = [
    %Layer{name: "Silted floor", depth: 0, hardness: 3, fragility: 2, context_cost: 6},
    %Layer{name: "Packed clay", depth: 1, hardness: 5, fragility: 3, context_cost: 8},
    %Layer{name: "Burn mark stratum", depth: 2, hardness: 7, fragility: 6, context_cost: 10},
    %Layer{name: "Crumble shelf", depth: 3, hardness: 9, fragility: 8, context_cost: 14},
    %Layer{name: "Foundational stone", depth: 4, hardness: 11, fragility: 9, context_cost: 16}
]

VAR excavation_tools: Tool[] = [
    %Tool{name: "Claw Pick", precision: 5, durability: 18},
    %Tool{name: "Fine Chisel", precision: 8, durability: 12},
    %Tool{name: "Artist Brush", precision: 10, durability: 10}
]

VAR site_artifacts: Artifact[] = [
    %Artifact{
        title: "Charcoal seal",
        layer_tag: 0,
        fragility: 4,
        cultural_value: 22,
        found: false,
        preserved: false
    },
    %Artifact{
        title: "Bronze coin cluster",
        layer_tag: 1,
        fragility: 6,
        cultural_value: 44,
        found: false,
        preserved: false
    },
    %Artifact{
        title: "Mosaic fragment",
        layer_tag: 2,
        fragility: 9,
        cultural_value: 66,
        found: false,
        preserved: false
    },
    %Artifact{
        title: "Bronze figurine wrist",
        layer_tag: 3,
        fragility: 10,
        cultural_value: 78,
        found: false,
        preserved: false
    },
    %Artifact{
        title: "Foundation tablet",
        layer_tag: 4,
        fragility: 10,
        cultural_value: 95,
        found: false,
        preserved: false
    }
]

VAR team_technique: int = 68
VAR context_integrity: int = 100
VAR wear_total: int = 0

== main ==
The excavation camp opens before dawn.
~ profile_site(0)
~ report_tools()
~ run_dig(0)
~ finalize_dig()
-> DONE

== function run_dig(index: int) => void ==
{ if index >= LEN(dig_layers):
    ~ return
- else:
    ~ temp layer: Layer = dig_layers[index]
    ~ temp tool_index: int = choose_tool(layer.hardness, layer.fragility)
    ~ temp tool: Tool = excavation_tools[tool_index]
    { layer.name } at depth {layer.depth} is being opened with {tool.name}.
    ~ apply_tool_wear(index, tool_index)
    ~ temp result: int = extract_artifact(index, tool_index)
    { if result == 2:
        ~ site_artifacts[index].found = true
        ~ site_artifacts[index].preserved = true
        A pristine artifact is recovered.
    - else:
        { if result == 1:
            ~ site_artifacts[index].found = true
            ~ site_artifacts[index].preserved = false
            The find is recoverable but context is damaged.
        - else:
            No recoverable remains appear this layer.
        }
    }
    ~ context_integrity = context_integrity - layer.context_cost
    { if context_integrity < 0:
        ~ context_integrity = 0
    }
    ~ run_dig(index + 1)
}

== function choose_tool(hardness: int, fragility: int) => int ==
{ if fragility >= 10:
    ~ return 2
- else:
    { if hardness >= 9:
        ~ return 0
    - else:
        { if hardness >= 6:
            ~ return 1
        - else:
            ~ return 2
        }
    }
}

== function apply_tool_wear(layer_index: int, tool_index: int) => void ==
~ temp layer: Layer = dig_layers[layer_index]
~ temp tool: Tool = excavation_tools[tool_index]
~ temp wear: int = layer.hardness - tool.precision + 1
{ if wear < 1:
    ~ wear = 1
}
~ temp new_condition: int = tool.durability - wear
{ if new_condition < 0:
    ~ new_condition = 0
}
~ excavation_tools[tool_index].durability = new_condition
~ wear_total = wear_total + wear
~ tool_status_log(tool_index, new_condition, wear)

== function extract_artifact(layer_index: int, tool_index: int) => int ==
~ temp layer: Layer = dig_layers[layer_index]
~ temp tool: Tool = excavation_tools[tool_index]
~ temp artifact: Artifact = site_artifacts[layer_index]
~ temp precision_buffer: int = tool.precision + tool.durability
~ temp required: int = layer.fragility + artifact.fragility + layer.hardness + layer.context_cost
~ temp bonus: int = team_technique / 10
{ if precision_buffer + bonus >= required + 10:
    ~ return 2
- else:
    { if precision_buffer + bonus >= required:
        ~ return 1
    - else:
        ~ return 0
    }
}

== function count_found(index: int) => int ==
{ if index >= LEN(site_artifacts):
    ~ return 0
- else:
    { if site_artifacts[index].found:
        ~ return 1 + count_found(index + 1)
    - else:
        ~ return count_found(index + 1)
    }
}

== function count_preserved(index: int) => int ==
{ if index >= LEN(site_artifacts):
    ~ return 0
- else:
    { if site_artifacts[index].preserved:
        ~ return 1 + count_preserved(index + 1)
    - else:
        ~ return count_preserved(index + 1)
    }
}

== function profile_site(index: int) => void ==
{ if index >= LEN(dig_layers):
    ~ return
- else:
    ~ temp layer: Layer = dig_layers[index]
    Layer {index + 1}: {layer.name} depth {layer.depth} with hardness {layer.hardness}.
    ~ profile_site(index + 1)
}

== function report_tools() => void ==
Current tools:
~ report_tool_status(0)

== function report_tool_status(index: int) => void ==
{ if index >= LEN(excavation_tools):
    ~ return
- else:
    ~ temp tool: Tool = excavation_tools[index]
    {tool.name} condition {tool.durability}.
    ~ report_tool_status(index + 1)
}

== function tool_status_log(tool_index: int, condition: int, wear: int) => void ==
~ temp tool: Tool = excavation_tools[tool_index]
Use {tool.name}: wear {wear}, condition now {condition}.

== function finalize_dig() => void ==
Excavation complete.
~ temp found_count: int = count_found(0)
~ temp preserved_count: int = count_preserved(0)
Artifacts recovered: {found_count}
Artifacts preserved: {preserved_count}
Site context integrity: {context_integrity}
Total tool wear: {wear_total}
Report quality score: {report_quality_score(found_count, preserved_count)}
{ if preserved_count == found_count and found_count > 0:
    Provenance chain remains intact.
- else:
    { if found_count == 0:
        Nothing of note was retained for the archive.
    - else:
        Context gaps are visible in the official report.
    }
}

== function report_quality_score(found: int, preserved: int) => int ==
~ temp score: int = context_integrity + preserved * 18 + found * 4
{ if score > 120:
    ~ return 120
- else:
    { if score < 0:
        ~ return 0
    - else:
        ~ return score
    }
}
