=== module game ===

STRUCT GuildMember {
    name: string
    rank: int
    reputation: int
    tasks_completed: int
    tasks_failed: int
    dues: int
    status: string
}

STRUCT GuildTask {
    member: string
    title: string
    result: int
    rep_delta: int
    dues_delta: int
    payout: int
}

VAR guild_reputation: int = 34
VAR guild_dues_pool: int = 0
VAR guild_treasury: int = 210
VAR promotion_candidates: int = 0
VAR demotion_candidates: int = 0
VAR tasks_passed: int = 0
VAR tasks_failed: int = 0

VAR members: GuildMember[] = [
    %GuildMember{ name: "Alaric", rank: 1, reputation: 12, tasks_completed: 0, tasks_failed: 0, dues: 4, status: "active" },
    %GuildMember{ name: "Nemi", rank: 2, reputation: 9, tasks_completed: 0, tasks_failed: 0, dues: 11, status: "active" },
    %GuildMember{ name: "Vex", rank: 1, reputation: 4, tasks_completed: 0, tasks_failed: 0, dues: 9, status: "active" }
]

VAR member_index: Dict<string, int> = %{
    "Alaric": 0,
    "Nemi": 1,
    "Vex": 2
}

VAR task_log: GuildTask[] = [
    %GuildTask{ member: "Alaric", title: "Ward Repair", result: 1, rep_delta: 3, dues_delta: -2, payout: 6 },
    %GuildTask{ member: "Nemi", title: "Gate Audit", result: 0, rep_delta: -1, dues_delta: 2, payout: 2 },
    %GuildTask{ member: "Vex", title: "Courier Route", result: 1, rep_delta: 4, dues_delta: -3, payout: 3 },
    %GuildTask{ member: "Nemi", title: "Silent Escort", result: -1, rep_delta: -5, dues_delta: 5, payout: -1 },
    %GuildTask{ member: "Vex", title: "Patrol Duty", result: -1, rep_delta: -4, dues_delta: 4, payout: 0 },
    %GuildTask{ member: "Alaric", title: "Rift Repair", result: 1, rep_delta: 2, dues_delta: -1, payout: 3 }
]

== main ==
Guild quarter review.
Current guild reputation: {guild_reputation}
Treasury reserve: {guild_treasury}
~ print_roster("Starting")
~ process_tasks(0)
~ evaluate_members()
~ print_roster("Final")
~ summary_report()
-> DONE

== function print_roster(label: string) => void ==
{ label } roster snapshot:
- Alaric: rank {members[0].rank}, rep {members[0].reputation}, dues {members[0].dues}, tasks ok {members[0].tasks_completed} fail {members[0].tasks_failed}, {members[0].status}
- Nemi: rank {members[1].rank}, rep {members[1].reputation}, dues {members[1].dues}, tasks ok {members[1].tasks_completed} fail {members[1].tasks_failed}, {members[1].status}
- Vex: rank {members[2].rank}, rep {members[2].reputation}, dues {members[2].dues}, tasks ok {members[2].tasks_completed} fail {members[2].tasks_failed}, {members[2].status}

== function process_tasks(index: int) => void ==
{ if index >= LEN(task_log):
    Task book processed.
- else:
    ~ temp task: GuildTask = task_log[index]
    Task {index + 1}: {task.title} assigned to {task.member}.
    ~ resolve_task(task)
    ~ process_tasks(index + 1)
}

== function resolve_task(task: GuildTask) => void ==
~ temp slot: int = member_index[task.member]
~ temp member: GuildMember = members[slot]
~ temp dues_before: int = member.dues
~ temp rep_before: int = member.reputation

{ if task.result > 0:
    ~ members[slot].tasks_completed = member.tasks_completed + 1
    ~ members[slot].reputation = member.reputation + task.rep_delta
    ~ tasks_passed = tasks_passed + 1
    Task outcome: success.
- else:
    { if task.result == 0:
        ~ members[slot].tasks_completed = member.tasks_completed + 1
        ~ members[slot].reputation = member.reputation + task.rep_delta
        ~ tasks_passed = tasks_passed + 1
        Task outcome: partial.
    - else:
        ~ members[slot].tasks_failed = member.tasks_failed + 1
        ~ members[slot].reputation = member.reputation + task.rep_delta
        ~ tasks_failed = tasks_failed + 1
        Task outcome: failed.
    }
}

~ members[slot].dues = dues_before + task.dues_delta
~ guild_dues_pool = guild_dues_pool + task.dues_delta
~ temp task_effective_payout: int = task.payout
~ guild_reputation = guild_reputation + task.rep_delta
~ guild_treasury = guild_treasury + task_effective_payout

~ temp member_after: GuildMember = members[slot]
~ temp rep_change: int = member_after.reputation - rep_before
Task {task.title} result for {task.member}: rep {rep_before} -> {member_after.reputation} (delta {rep_change}), dues {dues_before} -> {member_after.dues}.

== function evaluate_members() => void ==
Evaluating promotion and demotion.
~ evaluate_member(0)
~ evaluate_member(1)
~ evaluate_member(2)

== function evaluate_member(slot: int) => void ==
~ temp member: GuildMember = members[slot]
~ temp promote_ok: bool = member.reputation >= 12 && member.dues <= 2
~ temp demote_needed: bool = member.reputation <= 5 || member.dues >= 12
{ if promote_ok && member.rank < 3:
    ~ members[slot].rank = member.rank + 1
    ~ promotion_candidates = promotion_candidates + 1
    { member.name } earns a promotion.
- else:
    { if demote_needed && member.rank > 0:
        ~ members[slot].rank = member.rank - 1
        ~ demotion_candidates = demotion_candidates + 1
        { member.name } is demoted this quarter.
    - else:
        { if member.rank == 0 && demote_needed:
            ~ members[slot].status = "probation"
            ~ demotion_candidates = demotion_candidates + 1
            { member.name } is placed on probation.
        - else:
            { member.name } holds rank.
        }
    }
}

== function summary_report() => void ==
Guild reputation: {guild_reputation}
Completed tasks: {tasks_passed}
Failed tasks: {tasks_failed}
Dues pool delta: {guild_dues_pool}
Promotions issued: {promotion_candidates}
Demotions enforced: {demotion_candidates}
Treasury: {guild_treasury}
Current dues burden by member:
Alaric {members[0].dues}
Nemi {members[1].dues}
Vex {members[2].dues}
