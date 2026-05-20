=== module game ===

STRUCT NPC {
    name: string
    fatigue: int
    voice: int
}

STRUCT Route {
    from: int
    to: int
    loss: int
    tone: int
}

STRUCT RumorEntry {
    person: string
    phrase: string
    confidence: int
    source: string
}

VAR tavern_npcs: NPC[] = [
    %NPC{name: "Innkeeper Vale", fatigue: 0, voice: 3},
    %NPC{name: "Captain Rook", fatigue: 2, voice: -1},
    %NPC{name: "Healer Nessa", fatigue: 5, voice: 2},
    %NPC{name: "Blacksmith Orin", fatigue: 3, voice: 1},
    %NPC{name: "Magistrate Tolan", fatigue: 1, voice: 0}
]

VAR rumor_routes: Route[] = [
    %Route{from: 0, to: 1, loss: 8, tone: 2},
    %Route{from: 0, to: 3, loss: 10, tone: 0},
    %Route{from: 1, to: 2, loss: 11, tone: -1},
    %Route{from: 1, to: 4, loss: 14, tone: 1},
    %Route{from: 3, to: 2, loss: 9, tone: -2},
    %Route{from: 2, to: 4, loss: 7, tone: 2}
]

VAR rumor_map: RumorEntry[] = [
    %RumorEntry{person: "Innkeeper Vale", phrase: "(unheard)", confidence: 0, source: ""},
    %RumorEntry{person: "Captain Rook", phrase: "(unheard)", confidence: 0, source: ""},
    %RumorEntry{person: "Healer Nessa", phrase: "(unheard)", confidence: 0, source: ""},
    %RumorEntry{person: "Blacksmith Orin", phrase: "(unheard)", confidence: 0, source: ""},
    %RumorEntry{person: "Magistrate Tolan", phrase: "(unheard)", confidence: 0, source: ""}
]

VAR rumor_seed: string = "A sealed wagon is bringing tax silver through the north road."
VAR rumor_floor: int = 22

== main ==
Evenings in the tavern begin with one whisper.
~ initialize_rumor()
~ propagate_from(0, rumor_seed, 95)
~ final_rumor_map()
-> DONE

== function initialize_rumor() => void ==
~ rumor_map[0].phrase = rumor_seed
~ rumor_map[0].confidence = 95
~ rumor_map[0].source = "seed"
Initial report from {rumor_map[0].person}: "{rumor_map[0].phrase}" with confidence {rumor_map[0].confidence}.

== function propagate_from(sender: int, topic: string, confidence: int) => void ==
{ if confidence < rumor_floor:
    ~ return
- else:
    ~ dispatch_routes(sender, 0, topic, confidence)
}

== function dispatch_routes(sender: int, route_index: int, topic: string, confidence: int) => void ==
{ if route_index >= LEN(rumor_routes):
    ~ return
- else:
    ~ temp route: Route = rumor_routes[route_index]
    { if route.from == sender:
        ~ attempt_route(route, route_index, topic, confidence)
    }
    ~ dispatch_routes(sender, route_index + 1, topic, confidence)
}

== function attempt_route(route: Route, route_index: int, topic: string, confidence: int) => void ==
~ temp to_npc: NPC = tavern_npcs[route.to]
~ temp speaker: NPC = tavern_npcs[route.from]
~ temp decayed: int = confidence - route.loss - to_npc.fatigue
~ temp next_conf: int = decayed
{ if next_conf < 0:
    ~ next_conf = 0
}
~ temp routed_topic: string = mutate_topic(topic, route.tone, speaker.name, to_npc.name)
{ if next_conf >= rumor_floor:
    ~ update_rumor(route.to, route.from, routed_topic, next_conf)
    ~ propagate_from(route.to, routed_topic, next_conf)
- else:
    The route from {speaker.name} to {to_npc.name} arrives too thin to keep.
    ~ update_if_possible(route.to, route.from, routed_topic, next_conf)
}

== function mutate_topic(topic: string, tone: int, speaker: string, listener: string) => string ==
{ if tone > 1:
    ~ return topic + " and " + speaker + " says it was " + listener + " sized up at the gate."
- else:
    { if tone < 0:
        ~ return "Rumor from " + speaker + ": maybe " + topic + ", though this feels uncertain."
    - else:
        ~ return topic + " (transmitted with little flourish)."
    }
}

== function update_rumor(index: int, sender_index: int, topic: string, confidence: int) => void ==
{ if confidence > rumor_map[index].confidence:
    ~ rumor_map[index].person = tavern_npcs[index].name
    ~ rumor_map[index].phrase = topic
    ~ rumor_map[index].confidence = confidence
    ~ rumor_map[index].source = tavern_npcs[sender_index].name
    { if confidence >= 60:
        ~ speaker_update(index, confidence, sender_index)
    - else:
        { if confidence >= rumor_floor:
            Rumor reaches {tavern_npcs[index].name} with soft certainty.
        }
    }
}

== function speaker_update(index: int, confidence: int, sender_index: int) => void ==
A confident account of {tavern_npcs[index].name}'s hearing comes from {tavern_npcs[sender_index].name}.

== function update_if_possible(index: int, sender_index: int, topic: string, confidence: int) => void ==
{ if confidence > rumor_map[index].confidence:
    ~ rumor_map[index].person = tavern_npcs[index].name
    ~ rumor_map[index].phrase = topic
    ~ rumor_map[index].confidence = confidence
    ~ rumor_map[index].source = tavern_npcs[sender_index].name + " (weak)"
}

== function final_rumor_map() => void ==
The tavern settles; what each person believes is logged.
~ report_entry(0)
~ most_trusted_index(0, 0)

== function report_entry(index: int) => void ==
{ if index >= LEN(rumor_map):
    ~ return
- else:
    { if rumor_map[index].confidence == 0:
        {rumor_map[index].person} never heard the claim.
    - else:
        {rumor_map[index].person}: "{rumor_map[index].phrase}"
        Confidence {rumor_map[index].confidence}, source {rumor_map[index].source}.
    }
    ~ report_entry(index + 1)
}

== function most_trusted_index(index: int, best: int) => void ==
{ if index >= LEN(rumor_map):
    ~ print_winner(best)
    ~ return
- else:
    { if rumor_map[index].confidence > rumor_map[best].confidence:
        ~ most_trusted_index(index + 1, index)
    - else:
        ~ most_trusted_index(index + 1, best)
    }
}

== function print_winner(best: int) => void ==
Most stable rumor currently comes from {rumor_map[best].person} with confidence {rumor_map[best].confidence}.
