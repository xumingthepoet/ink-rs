=== module game ===
VAR quest_ids: string[] = ["Rescue", "Herbs", "Signals"]
VAR quest_progress: Dict<string, int> = %{"Rescue": 0, "Herbs": 1, "Signals": 0}
VAR quest_goal: Dict<string, int> = %{"Rescue": 3, "Herbs": 3, "Signals": 1}
VAR events: string[] = ["Rescue", "Herbs", "Rescue", "Signals", "Rescue", "Herbs"]
VAR event_amounts: int[] = [1, 1, 2, 1, 1, 1]

== main ==
Quest objective table seeded.
-> show_quests(0)
-> apply_events(0)
-> DONE

== apply_events(index: int) ==
{ if index >= LEN(events):
    All quest updates applied.
    ->->
- else:
    ~ temp quest_id: string = events[index]
    ~ temp amount: int = event_amounts[index]
    {quest_id} advances by {amount}.
    ~ quest_progress[quest_id] = quest_progress[quest_id] + amount
    { if quest_progress[quest_id] >= quest_goal[quest_id]:
        {quest_id} complete.
    - else:
        {quest_id} incomplete.
    }
    -> show_quests(0)
    -> apply_events(index + 1)
}

== show_quests(index: int) ==
{ if index >= LEN(quest_ids):
    ->->
- else:
    ~ temp quest_id: string = quest_ids[index]
    {quest_id}: {quest_progress[quest_id]} / {quest_goal[quest_id]}.
    -> show_quests(index + 1)
}
