=== module game ===
STRUCT Rumor {
    id: int
    source: string
    topic: string
    claim: string
    reliability: int
}

VAR rumor_queue: int[] = [301, 302, 303, 304]

VAR rumor_catalog: Rumor[] = [
    %Rumor{
        id: 301,
        source: "Captain Rook",
        topic: "guards",
        claim: "The patrol shifted to the western wall.",
        reliability: 8
    },
    %Rumor{
        id: 302,
        source: "Street Singer",
        topic: "tax",
        claim: "The tax was canceled at dawn.",
        reliability: -20
    },
    %Rumor{
        id: 303,
        source: "Archivist Venn",
        topic: "tunnels",
        claim: "There is a maintenance hatch behind the shrine.",
        reliability: 10
    },
    %Rumor{
        id: 304,
        source: "Gossip",
        topic: "rumors",
        claim: "A market collapse means war is imminent.",
        reliability: -12
    }
]

VAR rumor_by_id: Dict<int, int> = %{
    301: 0,
    302: 1,
    303: 2,
    304: 3
}

VAR source_credibility: Dict<string, int> = %{
    "Captain Rook": 72,
    "Street Singer": 32,
    "Archivist Venn": 84,
    "Gossip": 24
}

VAR source_last_used: Dict<string, int> = %{
    "Captain Rook": 0,
    "Street Singer": 0,
    "Archivist Venn": 0,
    "Gossip": 0
}

VAR npc_names: string[] = ["Captain", "Trader", "Scholar"]
VAR npc_threshold: Dict<string, int> = %{
    "Captain": 65,
    "Trader": 42,
    "Scholar": 70
}

VAR rumor_accept_count: Dict<int, int> = %{
    301: 0,
    302: 0,
    303: 0,
    304: 0
}
VAR rumor_reject_count: Dict<int, int> = %{
    301: 0,
    302: 0,
    303: 0,
    304: 0
}

== main ==
Rumor credibility check.
~ evaluate_rumors(0)
~ summarize_sources()
-> DONE

== function evaluate_rumors(index: int) => void ==
{ if index >= LEN(rumor_queue):
    ~ return
- else:
    ~ temp rumor_id: int = rumor_queue[index]
    ~ temp rumor_index: int = rumor_by_id[rumor_id]
    ~ temp rumor: Rumor = rumor_catalog[rumor_index]
    Rumor {INT(index + 1)}: {rumor.claim}
    Source: {rumor.source} ({INT(rumor.reliability)} bonus)
    ~ audit_rumor(rumor_id, 0)
    ~ log_rumor_result(rumor_id)
    ~ evaluate_rumors(index + 1)
}

== function audit_rumor(rumor_id: int, audience_index: int) => void ==
{ if audience_index >= LEN(npc_names):
    ~ return
- else:
    ~ temp rumor_index: int = rumor_by_id[rumor_id]
    ~ temp rumor: Rumor = rumor_catalog[rumor_index]
    ~ temp listener: string = npc_names[audience_index]
    ~ temp listener_threshold: int = npc_threshold[listener]
    ~ temp source_score: int = source_credibility[rumor.source] + rumor.reliability
    { if source_score >= listener_threshold:
        {listener} believes this rumor.
        ~ rumor_accept_count[rumor_id] = rumor_accept_count[rumor_id] + 1
    - else:
        {listener} ignores this rumor.
        ~ rumor_reject_count[rumor_id] = rumor_reject_count[rumor_id] + 1
    }
    ~ audit_rumor(rumor_id, audience_index + 1)
}

== function log_rumor_result(rumor_id: int) => void ==
~ temp rumor_index: int = rumor_by_id[rumor_id]
~ temp rumor: Rumor = rumor_catalog[rumor_index]
Believers: {rumor_accept_count[rumor_id]}, Rejectors: {rumor_reject_count[rumor_id]}.

{ if rumor_accept_count[rumor_id] >= rumor_reject_count[rumor_id]:
    ~ source_credibility[rumor.source] = source_credibility[rumor.source] + 1
    Source trust in {rumor.source} increased.
- else:
    ~ source_credibility[rumor.source] = source_credibility[rumor.source] - 1
    Source trust in {rumor.source} decreased.
}

~ source_last_used[rumor.source] = source_last_used[rumor.source] + 1

== function summarize_sources() => void ==
Source credibility update:
Captain Rook: {source_credibility["Captain Rook"]} ({source_last_used["Captain Rook"]} citations)
Street Singer: {source_credibility["Street Singer"]} ({source_last_used["Street Singer"]} citations)
Archivist Venn: {source_credibility["Archivist Venn"]} ({source_last_used["Archivist Venn"]} citations)
Gossip: {source_credibility["Gossip"]} ({source_last_used["Gossip"]} citations)
