=== module game ===
STRUCT Mail {
    id: int
    sender: string
    title: string
    hook: int
    body: string
}

VAR incoming_mail_ids: int[] = [2001, 2002, 2003]

VAR mail_templates: Mail[] = [
    %Mail{
        id: 2001,
        sender: "Captain Rook",
        title: "Escort confirmed",
        hook: 1,
        body: "Take the envoy through the east gate before sunset."
    },
    %Mail{
        id: 2002,
        sender: "Spyglass Guild",
        title: "Night shipment",
        hook: 2,
        body: "A caravan leaves with route maps. Record every marker."
    },
    %Mail{
        id: 2003,
        sender: "Marshal Hessa",
        title: "Call for garrison",
        hook: 3,
        body: "Summon two squads for the outer wall immediately."
    }
]

VAR mail_by_id: Dict<int, int> = %{
    2001: 0,
    2002: 1,
    2003: 2
}

VAR mail_read: Dict<int, bool> = %{
    2001: false,
    2002: false,
    2003: false
}

VAR quest_state: Dict<string, int> = %{
    "escort": 0,
    "intel": 0,
    "garrison": 0
}

== main ==
Mail inbox and quest hooks.
~ show_inbox()
~ show_quests("Initial")
~ process_inbox(0)
~ show_inbox()
~ show_quests("After first sweep")
~ mail_read[2002] = false
Marked mail 2002 as unread.
~ process_inbox(0)
~ show_inbox()
~ show_quests("After reread")
-> DONE

== function process_inbox(index: int) => void ==
{ if index >= LEN(incoming_mail_ids):
    ~ return
- else:
    ~ temp mail_id: int = incoming_mail_ids[index]
    { if mail_read[mail_id]:
        Mail {mail_id} is already read.
    - else:
        ~ temp mail_index: int = mail_by_id[mail_id]
        ~ temp letter: Mail = mail_templates[mail_index]
        Incoming mail from {letter.sender}.
        Subject: {letter.title}
        {letter.body}
        ~ mail_read[mail_id] = true
        { if letter.hook == 1:
            { if quest_state["escort"] == 0:
                ~ quest_state["escort"] = 1
                Quest "Escort" started.
            - else:
                Escort already underway.
            }
        - else:
            { if letter.hook == 2:
                ~ quest_state["intel"] = quest_state["intel"] + 1
                Quest "Intel" tally changed to {quest_state["intel"]}.
            - else:
                ~ quest_state["garrison"] = 1
                Quest "Garrison" called.
            }
        }
    }
    ~ process_inbox(index + 1)
}

== function show_inbox() => void ==
Inbox status:
Mail 2001 read? {mail_read[2001]}
Mail 2002 read? {mail_read[2002]}
Mail 2003 read? {mail_read[2003]}

== function show_quests(label: string) => void ==
-- {label} --
Quest Escort: {quest_state["escort"]}
Quest Intel: {quest_state["intel"]}
Quest Garrison: {quest_state["garrison"]}
