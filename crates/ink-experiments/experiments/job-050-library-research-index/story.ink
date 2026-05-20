=== module game ===

STRUCT ResearchTopic {
    topic_id: string
    branch: string
    required_skill: int
    required_hours: int
    unlocked: bool
    investigated: bool
    note: string
}

VAR topic_catalog: ResearchTopic[] = [
    %ResearchTopic{
        topic_id: "ARC-01",
        branch: "ARC",
        required_skill: 4,
        required_hours: 3,
        unlocked: false,
        investigated: false,
        note: "Cross-checked archive transfer stamps show late manifest swaps."
    },
    %ResearchTopic{
        topic_id: "LAW-31",
        branch: "LAW",
        required_skill: 8,
        required_hours: 5,
        unlocked: false,
        investigated: false,
        note: "Legal transcripts reveal a suppressed hearing motion."
    },
    %ResearchTopic{
        topic_id: "NAV-04",
        branch: "NAV",
        required_skill: 6,
        required_hours: 4,
        unlocked: false,
        investigated: false,
        note: "Dock logs tie the midnight cargo list to an unlogged departure."
    },
    %ResearchTopic{
        topic_id: "MEC-77",
        branch: "MEC",
        required_skill: 7,
        required_hours: 6,
        unlocked: false,
        investigated: false,
        note: "Machine-room signatures expose temporary bypass power routing."
    },
    %ResearchTopic{
        topic_id: "PHI-19",
        branch: "PHI",
        required_skill: 9,
        required_hours: 4,
        unlocked: false,
        investigated: false,
        note: "Encoded ledger key confirms a dead-drop identity chain."
    }
]

VAR branch_skill_bonus: Dict<string, int> = %{
    "ARC": 2,
    "LAW": 1,
    "NAV": 2,
    "MEC": 1,
    "PHI": -1
}

VAR search_plan: string[] = [
    "ARC-01",
    "LAW-31",
    "UNKNOWN-01",
    "MEC-77",
    "LAW-31",
    "PHI-19",
    "NAV-04",
    "ARC-01"
]

VAR investigator_skill: int = 6
VAR remaining_hours: int = 16
VAR unlocked_notes: int = 0
VAR investigated_topics: int = 0
VAR missing_queries: int = 0
VAR unresolved_topics: int = 0
VAR evidence_weight: int = 0

== main ==
Library desk opens with {remaining_hours} investigator-hours.
~ run_topic_queries(0)
~ assemble_research_conclusion()
-> DONE

== function run_topic_queries(index: int) => void ==
{ if index >= LEN(search_plan):
    Search queue complete.
- else:
    ~ process_query_index(index)
    ~ run_topic_queries(index + 1)
}

== function process_query_index(index: int) => void ==
~ temp topic_id: string = search_plan[index]
-- Query {index + 1}: {topic_id} --
~ process_query(topic_id)

== function process_query(topic_id: string) => void ==
~ temp found_index: int = find_topic(topic_id, 0)
{ if found_index == -1:
    ~ missing_queries = missing_queries + 1
    No indexed record for topic "{topic_id}".
- else:
    ~ try_unlock(found_index)
}

== function try_unlock(index: int) => void ==
{ if topic_catalog[index].investigated:
    Topic "{topic_catalog[index].topic_id}" already investigated. Duplicate query skipped.
- else:
    ~ investigate_topic(index)
}

== function find_topic(topic_id: string, cursor: int) => int ==
{ if cursor < LEN(topic_catalog):
    { if topic_catalog[cursor].topic_id == topic_id:
        ~ return cursor
    - else:
        ~ return find_topic(topic_id, cursor + 1)
    }
- else:
    ~ return -1
}

== function investigate_topic(index: int) => void ==
~ temp topic: ResearchTopic = topic_catalog[index]
~ topic_catalog[index].investigated = true
~ investigated_topics = investigated_topics + 1
-- Investigating {topic.topic_id}: {topic.branch} branch --
~ temp effective_skill: int = investigator_skill + branch_skill_bonus[topic.branch]
{ if effective_skill >= topic.required_skill:
    { if remaining_hours >= topic.required_hours:
        ~ remaining_hours = remaining_hours - topic.required_hours
        ~ topic_catalog[index].unlocked = true
        ~ unlocked_notes = unlocked_notes + 1
        ~ evidence_weight = evidence_weight + 18
        Topic unlocked.
        Note: {topic.note}
    - else:
        ~ unresolved_topics = unresolved_topics + 1
        Not enough time. {topic.required_hours} hours needed, {remaining_hours} left.
    }
- else:
    ~ unresolved_topics = unresolved_topics + 1
    Skill check failed. Need {topic.required_skill}, have {effective_skill}.
}

== function assemble_research_conclusion() => void ==
Research conclusion pass starts.
~ print_locked_topics(0)
Conclusion index: {evidence_weight + unlocked_notes * 7}
~ unresolved_count_check()

== function print_locked_topics(index: int) => void ==
{ if index >= LEN(topic_catalog):
    End conclusion archive.
- else:
    { if topic_catalog[index].unlocked:
        Unlocked note: {topic_catalog[index].note}
    - else:
        { if topic_catalog[index].investigated:
            {topic_catalog[index].topic_id} blocked during search.
        - else:
            {topic_catalog[index].topic_id} was not reached in this pass.
        }
    }
    ~ print_locked_topics(index + 1)
}

== function unresolved_count_check() => void ==
-- Search report --
Topics investigated: {investigated_topics}
Topics unlocked: {unlocked_notes}
Missing queries: {missing_queries}
Unresolved searches: {unresolved_topics}
Remaining hours: {remaining_hours}
{ if unlocked_notes >= 3:
    The research division can build a formal hypothesis.
- else:
    The research trail is incomplete for courtroom-grade evidence.
}
