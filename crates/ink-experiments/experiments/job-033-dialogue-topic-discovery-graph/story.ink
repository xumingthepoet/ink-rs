=== module game ===
VAR reputation: int = 0
VAR topic_discovered: Dict<string, bool> = %{
    "greeting": true,
    "rumor": false,
    "supplies": false,
    "council": false,
    "ally": false
}
VAR topic_spoken: Dict<string, bool> = %{
    "greeting": false,
    "rumor": false,
    "supplies": false,
    "council": false,
    "ally": false
}
VAR topics_visited: int = 0

== main ==
Dialogue topic discovery graph.
Initial reputation: {reputation}
~ visit_topic("greeting")
~ traverse_discovery_graph()
Visited {topics_visited} topics. Final reputation {reputation}.
-> DONE

== function traverse_discovery_graph() => void ==
~ update_discovered_topics()
~ temp next_topic: string = pick_next_topic()
{ if next_topic == "":
    No additional topics are unlocked yet.
- else:
    ~ visit_topic(next_topic)
    ~ traverse_discovery_graph()
}

== function pick_next_topic() => string ==
{ if topic_discovered["rumor"] && !topic_spoken["rumor"]:
    ~ return "rumor"
- else:
    { if topic_discovered["supplies"] && !topic_spoken["supplies"]:
        ~ return "supplies"
    - else:
        { if topic_discovered["council"] && !topic_spoken["council"]:
            ~ return "council"
        - else:
            { if topic_discovered["ally"] && !topic_spoken["ally"]:
                ~ return "ally"
            - else:
                ~ return ""
            }
        }
    }
}

== function update_discovered_topics() => void ==
{ if !topic_discovered["rumor"] && topic_spoken["greeting"] && reputation >= 1:
    ~ topic_discovered["rumor"] = true
    A new topic unlocks: rumor.
- else:
    { if !topic_discovered["supplies"] && topic_spoken["greeting"] && reputation >= 2:
        ~ topic_discovered["supplies"] = true
        A new topic unlocks: supplies.
    - else:
        { if !topic_discovered["council"] && topic_spoken["rumor"] && topic_spoken["supplies"] && reputation >= 3:
            ~ topic_discovered["council"] = true
            A new topic unlocks: council.
        - else:
            { if !topic_discovered["ally"] && topic_spoken["council"] && reputation >= 4:
                ~ topic_discovered["ally"] = true
                A new topic unlocks: ally.
            }
        }
    }
}

== function visit_topic(topic: string) => void ==
{ if !topic_discovered[topic]:
    {topic} cannot be visited yet.
- else:
    { if !topic_spoken[topic]:
        { if topic == "greeting":
            You greet the messenger.
            ~ reputation = reputation + 1
            The messenger trusts you enough to stay.
        - else:
            { if topic == "rumor":
                You ask about rumors in the city.
                ~ reputation = reputation + 1
                A lead surfaces about the north gate.
            - else:
                { if topic == "supplies":
                    You ask about supplies and rations.
                    ~ reputation = reputation + 2
                    The messenger opens access to logistics channels.
                - else:
                    { if topic == "council":
                        You request a council introduction.
                        ~ reputation = reputation + 2
                        The messenger sends your name to senior members.
                    - else:
                        { if topic == "ally":
                            You request an ally contract.
                            ~ reputation = reputation + 3
                            A ready ally accepts to stand with you.
                        - else:
                            Unknown topic {topic}.
                        }
                    }
                }
            }
        }
        ~ topic_spoken[topic] = true
        ~ topics_visited = topics_visited + 1
    - else:
        {topic} already visited.
    }
}

