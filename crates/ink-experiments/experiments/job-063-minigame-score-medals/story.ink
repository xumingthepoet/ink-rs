=== module game ===

STRUCT MinigameMove {
    name: string
    base: int
    accuracy: int
    timing: int
}

STRUCT Medal {
    name: string
    minimum_score: int
    reward: string
}

VAR moves: MinigameMove[] = [
    %MinigameMove{name: "Whirlwind Slash", base: 12, accuracy: 10, timing: 9},
    %MinigameMove{name: "Ridge Jump", base: 8, accuracy: 6, timing: 6},
    %MinigameMove{name: "Counter Beat", base: 10, accuracy: 8, timing: 7},
    %MinigameMove{name: "Aerial Feint", base: 14, accuracy: 9, timing: 10},
    %MinigameMove{name: "Chain Spin", base: 9, accuracy: 7, timing: 5},
    %MinigameMove{name: "Focus Burst", base: 11, accuracy: 10, timing: 10},
    %MinigameMove{name: "Final Lock", base: 13, accuracy: 8, timing: 8}
]

VAR score_medals: Medal[] = [
    %Medal{name: "Bronze", minimum_score: 35, reward: "Practice Arena Pass"},
    %Medal{name: "Silver", minimum_score: 65, reward: "Combat Focus Herb"},
    %Medal{name: "Gold", minimum_score: 90, reward: "Twin-Reflex Charm"},
    %Medal{name: "Platinum", minimum_score: 115, reward: "Mythic Trigger Ring"}
]

VAR quality_name: Dict<int, string> = %{
    0: "miss",
    1: "pass",
    2: "good",
    3: "great"
}

VAR total_base: int = 0
VAR total_combo_bonus: int = 0
VAR total_score: int = 0
VAR combo_streak: int = 0
VAR max_combo: int = 0
VAR perfect_moves: int = 0
VAR missed_moves: int = 0
VAR unlocked_rewards: int = 0

== main ==
Rhythm-board minigame score audit.
~ print_move_sheet(0)
~ run_moves(0)
~ print_final_scores()
-> DONE

== function print_move_sheet(index: int) => void ==
Planned move sequence:
{ if index >= LEN(moves):
    ~ return
- else:
    ~ temp move: MinigameMove = moves[index]
    {index + 1}. {move.name} (base {move.base}, accuracy {move.accuracy}, timing {move.timing})
    ~ print_move_sheet(index + 1)
}

== function run_moves(index: int) => void ==
{ if index >= LEN(moves):
    ~ return
- else:
    ~ temp move: MinigameMove = moves[index]
    ~ temp move_no: int = index + 1
    ~ temp grade: int = evaluate_grade(move.accuracy, move.timing)
    ~ temp grade_label: string = quality_name[grade]
    ~ temp combo_bonus_this: int = 0
    ~ temp base_points: int = move.base
    ~ temp extra_points: int = 0
    ~ temp strike_label: string = "recovered"
    { if grade > 0:
        ~ combo_streak = combo_streak + 1
        ~ combo_bonus_this = combo_streak * 2
        ~ extra_points = combo_bonus_this
        ~ total_combo_bonus = total_combo_bonus + combo_bonus_this
        { if combo_streak > max_combo:
            ~ max_combo = combo_streak
        }
    - else:
        ~ combo_streak = 0
        ~ missed_moves = missed_moves + 1
        ~ strike_label = "broken"
    }

    { if grade >= 3:
        ~ perfect_moves = perfect_moves + 1
        ~ base_points = move.base + 4
        ~ strike_label = "perfect"
    - else:
        { if grade >= 2:
            ~ base_points = move.base + 2
            ~ strike_label = "clean"
        - else:
            { if grade == 1:
                ~ base_points = move.base
                ~ strike_label = "borderline"
            - else:
                ~ base_points = 0
            }
        }
    }

    ~ total_base = total_base + move.base
    ~ total_score = total_score + base_points + extra_points

    Move {move_no}: {move.name}
    Grade: {grade_label}
    {strike_label} swing.
    Move gain {base_points} + combo bonus {combo_bonus_this}, total {base_points + extra_points}.
    Current total: {total_score}, current combo {combo_streak}.
    ~ run_moves(index + 1)
}

== function evaluate_grade(accuracy: int, timing: int) => int ==
{ if accuracy >= 9 and timing >= 9:
    ~ return 3
- else:
    { if accuracy >= 7 and timing >= 7:
        ~ return 2
    - else:
        { if accuracy >= 5 and timing >= 5:
            ~ return 1
        - else:
            ~ return 0
        }
    }
}

== function print_final_scores() => void ==
Minigame complete.
Raw move points: {total_base}
Total combo bonus: {total_combo_bonus}
Overall score: {total_score}
Category: {score_category(total_score)}
Perfect moves: {perfect_moves}
Misses: {missed_moves}
Longest combo: {max_combo}
~ unlocked_rewards = reward_summary(0)
Unlocked rewards: {unlocked_rewards}
Combo award:
~ temp combo_award: string = combo_reward(max_combo)
{combo_award}
{ if unlocked_rewards == 0:
    No score medals unlocked this run.
- else:
    { if unlocked_rewards == LEN(score_medals):
        Full medal set unlocked.
    - else:
        Medal rewards unlocked: {unlocked_rewards} out of {LEN(score_medals)}.
    }
}

== function score_category(score: int) => string ==
{ if score >= 115:
    ~ return "Platinum Run"
- else:
    { if score >= 90:
        ~ return "Gold Run"
    - else:
        { if score >= 65:
            ~ return "Silver Run"
        - else:
        { if score >= 35:
            ~ return "Bronze Run"
        - else:
            ~ return "Practice Run"
            }
        }
    }
}

== function reward_summary(index: int) => int ==
{ if index >= LEN(score_medals):
    ~ return 0
- else:
    ~ temp medal: Medal = score_medals[index]
    { if total_score >= medal.minimum_score:
        Medal unlocked: {medal.name}
        reward: {medal.reward}
        ~ return 1 + reward_summary(index + 1)
    - else:
        Medal locked: {medal.name}
        ~ return reward_summary(index + 1)
    }
}

== function combo_reward(combo: int) => string ==
{ if combo >= 4:
    ~ return "Combo medal unlocked: Momentum Crest (+8% reroll resistance)."
- else:
    { if combo >= 3:
        ~ return "Almost there: maintain a four-hit chain for Momentum Crest."
    - else:
        ~ return "No combo reward yet. Two-hit chains only reset after misses."
    }
}
