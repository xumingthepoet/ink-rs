=== module game ===

STRUCT CampfireStory {
    title: string
    priority: int
    told: bool
}

VAR camp_stories: CampfireStory[] = [
    %CampfireStory{title: "The bridge at moonfall", priority: 7, told: false},
    %CampfireStory{title: "Emberbird oaths", priority: 4, told: false},
    %CampfireStory{title: "The debt coin", priority: 8, told: false},
    %CampfireStory{title: "Hollow watchman", priority: 5, told: false},
    %CampfireStory{title: "The last cartographer", priority: 6, told: false}
]

VAR companions: string[] = ["Bran", "Neris", "Tal", "Sera"]
VAR companion_turn: int = 0
VAR cycle_completions: int = 0
VAR stories_told_in_cycle: int = 0
VAR stories_told_all_time: int = 0

== main ==
The campfire clears the fog for the night watch.
~ print_story_board(0)
~ run_campfire(1)
Night rotation complete.
~ print_final_state()
-> DONE

== function run_campfire(night: int) => void ==
{ if night > 8:
    ~ return
}

~ temp storyteller: string = next_companion()
~ temp story_index: int = next_untold_priority(0, -1, -1)

{ if story_index < 0:
    ~ cycle_completions = cycle_completions + 1
    All remaining stories have been told.
    The circle resets the slate.
    ~ reset_told_flags(0)
    ~ stories_told_in_cycle = 0
    ~ story_index = next_untold_priority(0, -1, -1)
}

~ temp selected: CampfireStory = camp_stories[story_index]
Night {night}: storyteller {storyteller}.
~ camp_stories[story_index].told = true
~ stories_told_in_cycle = stories_told_in_cycle + 1
~ stories_told_all_time = stories_told_all_time + 1
{selected.title}
Tally: priority {selected.priority}, band {story_band(selected.priority)}.
~ run_campfire(night + 1)

== function next_companion() => string ==
~ temp next: string = companions[companion_turn]
~ companion_turn = companion_turn + 1
{ if companion_turn >= LEN(companions):
    ~ companion_turn = 0
}
~ return next

== function next_untold_priority(index: int, top_priority: int, top_index: int) => int ==
{ if index >= LEN(camp_stories):
    ~ return top_index
}

~ temp story: CampfireStory = camp_stories[index]
{ if story.told:
    ~ return next_untold_priority(index + 1, top_priority, top_index)
- else:
    { if top_priority < 0 or story.priority > top_priority:
        ~ return next_untold_priority(index + 1, story.priority, index)
    - else:
        ~ return next_untold_priority(index + 1, top_priority, top_index)
    }
}

== function reset_told_flags(index: int) => void ==
{ if index >= LEN(camp_stories):
    ~ return
}
~ camp_stories[index].told = false
~ reset_told_flags(index + 1)

== function story_band(priority: int) => string ==
{ if priority >= 8:
    ~ return "epic"
- else:
    { if priority >= 6:
        ~ return "rare"
    - else:
        { if priority >= 4:
            ~ return "common"
        - else:
            ~ return "everyday"
        }
    }
}

== function print_story_board(index: int) => void ==
{ if index >= LEN(camp_stories):
    ~ return
}

~ temp story: CampfireStory = camp_stories[index]
~ temp status: string = if_not_told(story.told)
{story.title} status: {status}
~ print_story_board(index + 1)

== function if_not_told(told: bool) => string ==
{ if told:
    ~ return "told"
- else:
    ~ return "untold"
}

== function print_final_state() => void ==
~ print_story_board(0)
Cycles completed: {cycle_completions}
Stories told this run: {stories_told_all_time}
Stories told in current cycle: {stories_told_in_cycle}
