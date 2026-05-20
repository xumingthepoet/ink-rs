=== module game ===

STRUCT Contestant {
    name: string
    music: int
    dance: int
    craft: int
    humor: int
    charisma: int
}

VAR contestants: Contestant[] = [
    %Contestant{ name: "Lina Wren", music: 18, dance: 13, craft: 16, humor: 11, charisma: 14 },
    %Contestant{ name: "Davos Kettle", music: 15, dance: 16, craft: 17, humor: 9, charisma: 12 },
    %Contestant{ name: "Mira Quill", music: 16, dance: 18, craft: 13, humor: 15, charisma: 9 },
    %Contestant{ name: "Kori Bell", music: 14, dance: 12, craft: 19, humor: 16, charisma: 11 }
]

VAR audience_mood: int = 79

VAR final_scores: int[] = [0, 0, 0, 0]
VAR audience_bonus: int[] = [0, 0, 0, 0]
VAR places: int[] = [0, 0, 0, 0]
VAR awards: string[] = ["", "", "", ""]

VAR gold_awards: int = 0
VAR silver_awards: int = 0
VAR bronze_awards: int = 0
VAR honorable_mentions: int = 0

== main ==
Festival contest scoring run.
~ evaluate_all(0)
~ award_placements(0)
~ print_results()
-> DONE

== function evaluate_all(index: int) => void ==
{ if index >= LEN(contestants):
    Score cards prepared.
- else:
    ~ temp contestant: Contestant = contestants[index]
    ~ temp base: int = weighted_total(contestant)
    ~ temp bonus: int = audience_bonus_for(contestant)
    ~ final_scores[index] = base + bonus
    ~ audience_bonus[index] = bonus
    ~ evaluate_all(index + 1)
}

== function weighted_total(contestant: Contestant) => int ==
~ temp discipline_score: int = contestant.music * 3 + contestant.dance * 2 + contestant.craft * 2 + contestant.humor
~ temp style_bonus: int = 0
{ if contestant.craft > 16:
    ~ style_bonus = 3
- else:
    { if contestant.craft > 12:
        ~ style_bonus = 1
    }
}
~ return discipline_score + style_bonus

== function audience_bonus_for(contestant: Contestant) => int ==
~ temp bonus: int = audience_mood / 12
~ temp charisma_bonus: int = contestant.charisma / 4
~ bonus = bonus + charisma_bonus
{ if contestant.humor > 14:
    ~ bonus = bonus + 5
- else:
    { if contestant.humor > 10:
        ~ bonus = bonus + 3
    - else:
        { if contestant.humor > 7:
            ~ bonus = bonus + 1
        }
    }
}
{ if contestant.music > contestant.dance + 4:
    ~ bonus = bonus + 2
- else:
    { if contestant.dance > contestant.craft + 2:
        ~ bonus = bonus + 1
    }
}
~ return bonus

== function award_placements(index: int) => void ==
{ if index >= LEN(contestants):
    ~ count_awards(0)
- else:
    ~ temp rank: int = count_higher(index, 0) + 1
    ~ places[index] = rank
    ~ temp shared: bool = has_tie(index, 0)
    ~ awards[index] = award_for_rank(rank, shared)
    ~ award_placements(index + 1)
}

== function count_higher(index: int, rival: int) => int ==
{ if rival >= LEN(contestants):
    ~ return 0
- else:
    ~ temp higher_rest: int = count_higher(index, rival + 1)
    { if rival != index && final_scores[rival] > final_scores[index]:
        ~ higher_rest = higher_rest + 1
    }
    ~ return higher_rest
}

== function has_tie(index: int, rival: int) => bool ==
{ if rival >= LEN(contestants):
    ~ return false
- else:
    { if rival != index && final_scores[rival] == final_scores[index]:
        ~ return true
    - else:
        ~ return has_tie(index, rival + 1)
    }
}

== function award_for_rank(rank: int, shared: bool) => string ==
{ if rank == 1:
    { if shared:
        ~ return "Shared Gold"
    - else:
        ~ return "Gold"
    }
- else:
    { if rank == 2:
        { if shared:
            ~ return "Shared Silver"
        - else:
            ~ return "Silver"
        }
    - else:
        { if rank == 3:
            { if shared:
                ~ return "Shared Bronze"
            - else:
                ~ return "Bronze"
            }
        - else:
            ~ return "Honorable Mention"
        }
    }
}

== function count_awards(index: int) => void ==
{ if index >= LEN(contestants):
    ~ return
- else:
    ~ temp rank: int = places[index]
    { if rank == 1:
        ~ gold_awards = gold_awards + 1
    - else:
        { if rank == 2:
            ~ silver_awards = silver_awards + 1
        - else:
            { if rank == 3:
                ~ bronze_awards = bronze_awards + 1
            - else:
                ~ honorable_mentions = honorable_mentions + 1
            }
        }
    }
    ~ count_awards(index + 1)
}

== function print_results() => void ==
Contest score summaries with category totals.
~ print_scores(0)
~ print_tallies()

== function print_scores(index: int) => void ==
{ if index >= LEN(contestants):
    ~ return
- else:
    ~ print_one_score(index)
    ~ print_scores(index + 1)
}

== function print_one_score(index: int) => void ==
~ temp contestant: Contestant = contestants[index]
~ temp weighted: int = weighted_total(contestant)
~ temp bonus: int = audience_bonus[index]
~ temp total: int = final_scores[index]
~ temp rank: int = places[index]
{ if rank == 1:
    Contestant {contestant.name}: place {rank}
- else:
    { if rank == 2:
        Contestant {contestant.name}: place {rank}
    - else:
        { if rank == 3:
            Contestant {contestant.name}: place {rank}
        - else:
            Contestant {contestant.name}: place {rank}
        }
    }
}
Category score: {weighted}
Audience bonus: {bonus}
Final total: {total}
Award: {awards[index]}

== function print_tallies() => void ==
Judging summary:
Audience mood: {audience_mood}
Gold awards: {gold_awards}
Silver awards: {silver_awards}
Bronze awards: {bronze_awards}
Honorable mentions: {honorable_mentions}
{ if gold_awards > 1:
    A tie occurred at first place. All top-scoring contestants share Gold.
- else:
    { if silver_awards > 1:
        Silver tier shared for multiple contestants.
    - else:
        { if bronze_awards > 1:
            Bronze tier shared for multiple contestants.
        - else:
            Awards assigned to unique podium positions.
        }
    }
}
Final crowd reaction: {audience_label()}

== function audience_label() => string ==
{ if audience_mood >= 85:
    ~ return "Triumphant"
- else:
    { if audience_mood >= 70:
        ~ return "Supportive"
    - else:
        { if audience_mood >= 55:
            ~ return "Engaged"
        - else:
            ~ return "Reserved"
        }
    }
}
