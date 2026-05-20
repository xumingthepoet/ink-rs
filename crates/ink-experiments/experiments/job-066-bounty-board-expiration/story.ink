=== module game ===

STRUCT Bounty {
    id: int
    title: string
    target: string
    reward: int
}

STRUCT ClaimAttempt {
    day: int
    bounty_id: int
    claimant: string
}

VAR board_ids: int[] = [101, 102, 103, 104]

VAR bounties: Bounty[] = [
    %Bounty{id: 101, title: "Escort the ferryman", target: "Night Courier", reward: 120},
    %Bounty{id: 102, title: "Recover missing ledgers", target: "Clerk Rowan", reward: 180},
    %Bounty{id: 103, title: "Patrol the west road", target: "Iron Convoy", reward: 140},
    %Bounty{id: 104, title: "Capture false alchemist", target: "Fraud Alchemist", reward: 220}
]

VAR bounty_index: Dict<int, int> = %{
    101: 0,
    102: 1,
    103: 2,
    104: 3
}

VAR days_left: Dict<int, int> = %{
    101: 3,
    102: 2,
    103: 4,
    104: 1
}

VAR bounty_status: Dict<int, string> = %{
    101: "open",
    102: "open",
    103: "open",
    104: "open"
}

VAR bounty_claimant: Dict<int, string> = %{
    101: "",
    102: "",
    103: "",
    104: ""
}

VAR day_log: int[] = [1, 2, 3, 4, 5]

VAR attempts: ClaimAttempt[] = [
    %ClaimAttempt{day: 1, bounty_id: 101, claimant: "Player"},
    %ClaimAttempt{day: 1, bounty_id: 102, claimant: "Guild Clerk"},
    %ClaimAttempt{day: 2, bounty_id: 102, claimant: "Player"},
    %ClaimAttempt{day: 3, bounty_id: 103, claimant: "Mercenary Captain"},
    %ClaimAttempt{day: 4, bounty_id: 104, claimant: "Player"},
    %ClaimAttempt{day: 5, bounty_id: 101, claimant: "Guild Clerk"}
]

VAR player_gold: int = 320
VAR guild_gold: int = 240
VAR active_cleanup: int = 0
VAR successful_claims: int = 0
VAR denied_claims: int = 0
VAR payouts_to_player: int = 0
VAR payouts_to_npcs: int = 0

== main ==
Bounty board runs a full week.
~ print_board_status("Opening board")
~ run_board(0)
~ print_board_status("Board closes")
~ print_bounty_summary()
-> DONE

== function run_board(day_index: int) => void ==
{ if day_index >= LEN(day_log):
    Daybook complete.
- else:
    ~ temp day: int = day_log[day_index]
    === City Day {day} ===
    ~ tick_expiration()
    ~ process_claims_for_day(day, 0)
    ~ cleanup_board()
    ~ print_board_status_day(day)
    ~ run_board(day_index + 1)
}

== function tick_expiration() => void ==
~ decrement_day_counter(0)

== function decrement_day_counter(index: int) => void ==
{ if index >= LEN(board_ids):
    ~ return
- else:
    ~ temp bounty_id: int = board_ids[index]
    ~ temp status: string = bounty_status[bounty_id]
    { if status == "open":
        ~ temp remaining: int = days_left[bounty_id]
        { if remaining <= 0:
            Bounty {bounty_id} has already expired.
            ~ bounty_status[bounty_id] = "expired"
        - else:
            ~ days_left[bounty_id] = remaining - 1
            ~ temp after: int = days_left[bounty_id]
            { if after == 0:
                Expiry warning for bounty {bounty_id} now reaches zero by next dawn.
            - else:
                Bounty {bounty_id} now has {after} day(s) left.
            }
        }
    }
    ~ decrement_day_counter(index + 1)
}

== function process_claims_for_day(day: int, attempt_index: int) => void ==
{ if attempt_index >= LEN(attempts):
    ~ return
- else:
    ~ temp attempt: ClaimAttempt = attempts[attempt_index]
    { if attempt.day == day:
        ~ handle_claim(attempt)
    }
    ~ process_claims_for_day(day, attempt_index + 1)
}

== function handle_claim(attempt: ClaimAttempt) => void ==
~ temp id: int = attempt.bounty_id
~ temp status: string = bounty_status[id]
{ if status != "open":
    Attempt by {attempt.claimant} on bounty {id} fails.
    ~ denied_claims = denied_claims + 1
- else:
    ~ temp remaining: int = days_left[id]
    { if remaining <= 0:
        Bounty {id} is no longer available.
        ~ bounty_status[id] = "expired"
        ~ denied_claims = denied_claims + 1
    - else:
        ~ bounty_status[id] = "claimed"
        ~ bounty_claimant[id] = attempt.claimant
        ~ temp reward: int = bounty_reward(id)
        ~ temp tax: int = reward / 10
        ~ temp payout: int = reward - tax
        { if attempt.claimant == "Player":
            ~ player_gold = player_gold + payout
            ~ payouts_to_player = payouts_to_player + payout
            ~ guild_gold = guild_gold + tax
            Bounty {id} claimed by Player and paid out {payout}.
        - else:
            ~ guild_gold = guild_gold + reward
            ~ payouts_to_npcs = payouts_to_npcs + reward
            Bounty {id} claimed by {attempt.claimant} and payout goes to guild network.
        }
        ~ successful_claims = successful_claims + 1
    }
}

== function bounty_reward(id: int) => int ==
~ temp index: int = bounty_index[id]
~ temp bounty: Bounty = bounties[index]
~ return bounty.reward

== function cleanup_board() => void ==
~ clean_claimed(0)
~ clean_expired(0)

== function clean_claimed(index: int) => void ==
{ if index >= LEN(board_ids):
    ~ return
- else:
    ~ temp bounty_id: int = board_ids[index]
    ~ temp status: string = bounty_status[bounty_id]
    { if status == "claimed":
        ~ bounty_status[bounty_id] = "archived"
        ~ active_cleanup = active_cleanup + 1
        Archive removes completed bounty {bounty_id}.
    }
    ~ clean_claimed(index + 1)
}

== function clean_expired(index: int) => void ==
{ if index >= LEN(board_ids):
    ~ return
- else:
    ~ temp bounty_id: int = board_ids[index]
    ~ temp status: string = bounty_status[bounty_id]
    { if status == "expired":
        ~ bounty_status[bounty_id] = "archived"
        ~ active_cleanup = active_cleanup + 1
        Archive removes stale bounty {bounty_id}.
    }
    ~ clean_expired(index + 1)
}

== function print_board_status(label: string) => void ==
Board status: {label}
~ print_board_row(0)

== function print_board_status_day(day: int) => void ==
Board status: After day {day}
~ print_board_row(0)

== function print_board_row(index: int) => void ==
{ if index >= LEN(board_ids):
    ~ return
- else:
    ~ temp bounty_id: int = board_ids[index]
    ~ temp status: string = bounty_status[bounty_id]
    ~ temp claimant: string = bounty_claimant[bounty_id]
    ~ temp remaining: int = days_left[bounty_id]
    { if status == "open":
        ~ temp bounty: Bounty = bounties[bounty_index[bounty_id]]
        Bounty {bounty.id}: {bounty.title}
        Target: {bounty.target}
        Reward: {bounty.reward}
        State: {status}, remaining {remaining}
    - else:
        { if status == "archived":
            Bounty {bounty_id} is archived with state {status}.
            { if claimant != "":
                Last claimant: {claimant}
            - else:
                No claimant recorded.
            }
        - else:
            Bounty {bounty_id} is {status}.
        }
    }
    ~ print_board_row(index + 1)
}

== function print_bounty_summary() => void ==
Board payout summary.
Open bounties remaining: {open_bounties(0)}
Successful claims: {successful_claims}
Denied claims: {denied_claims}
Cleanup actions: {active_cleanup}
Gold ledger:
Player gold: {player_gold}
Guild gold: {guild_gold}
Player payouts: {payouts_to_player}
Non-player payouts: {payouts_to_npcs}

== function open_bounties(index: int) => int ==
{ if index >= LEN(board_ids):
    ~ return 0
- else:
    ~ temp bounty_id: int = board_ids[index]
    ~ temp status: string = bounty_status[bounty_id]
    ~ temp rest: int = open_bounties(index + 1)
    { if status == "open":
        ~ return rest + 1
    - else:
        ~ return rest
    }
}
