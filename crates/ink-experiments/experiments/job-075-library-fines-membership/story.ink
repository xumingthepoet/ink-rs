=== module library ===

STRUCT BorrowedBook {
    title: string
    due_in: int
    overdue_days: int
    active: bool
}

STRUCT BorrowRequest {
    day: int
    title: string
    due_days: int
    min_tier: int
    reward_points: int
}

STRUCT ReturnEvent {
    day: int
    slot: int
}

VAR borrowed_books: BorrowedBook[] = [
    %BorrowedBook{title: "Introductory Herbology", due_in: 3, overdue_days: 0, active: true},
    %BorrowedBook{title: "Civic Charters", due_in: 1, overdue_days: 0, active: true},
    %BorrowedBook{title: "", due_in: 0, overdue_days: 0, active: false},
    %BorrowedBook{title: "", due_in: 0, overdue_days: 0, active: false}
]

VAR requests: BorrowRequest[] = [
    %BorrowRequest{day: 2, title: "Guild Ledger", due_days: 2, min_tier: 1, reward_points: 5},
    %BorrowRequest{day: 3, title: "Archive Cartography", due_days: 1, min_tier: 2, reward_points: 8},
    %BorrowRequest{day: 4, title: "Folk Tales", due_days: 3, min_tier: 0, reward_points: 2}
]

VAR returns: ReturnEvent[] = [
    %ReturnEvent{day: 2, slot: 1},
    %ReturnEvent{day: 5, slot: 0}
]

VAR borrowed_count: int = 2
VAR overdue_fines: int = 0
VAR membership_score: int = 34
VAR denied_requests: int = 0
VAR request_cursor: int = 0
VAR return_cursor: int = 0
VAR total_days: int = 6

== main ==
Library branch daybook starts.
~ process_days(1)
~ print_final_membership_report()
-> DONE

== function process_days(day: int) => void ==
{ if day > total_days:
    All six-day checks complete.
- else:
    Day {day} begins.
    ~ update_overdues(0)
    ~ process_returns(day)
    ~ process_requests(day)
    ~ print_daily_state(day)
    ~ process_days(day + 1)
}

== function update_overdues(index: int) => void ==
{ if index >= LEN(borrowed_books):
    ~ return
}

~ temp book: BorrowedBook = borrowed_books[index]
{ if book.active:
    { if book.due_in > 0:
        ~ borrowed_books[index].due_in = book.due_in - 1
        { if borrowed_books[index].due_in <= 0:
            Borrowed book "{book.title}" reaches due point.
        }
    - else:
        ~ borrowed_books[index].overdue_days = book.overdue_days + 1
        ~ overdue_fines = overdue_fines + 3
        ~ membership_score = membership_score - 3
        { if borrowed_books[index].overdue_days > 0:
            Late fee accumulates on "{book.title}".
        }
    }
}
~ update_overdues(index + 1)

== function process_returns(day: int) => void ==
{ if return_cursor >= LEN(returns):
    ~ return
}

~ temp event: ReturnEvent = returns[return_cursor]
{ if event.day == day:
    ~ return_book(event.slot)
    ~ return_cursor = return_cursor + 1
    ~ process_returns(day)
- else:
    ~ return
}

== function return_book(slot: int) => void ==
~ temp book: BorrowedBook = borrowed_books[slot]
{ if !book.active:
    Return event for empty slot {slot}.
- else:
    ~ borrowed_books[slot].active = false
    ~ borrowed_count = borrowed_count - 1
    { if book.overdue_days == 0:
        ~ membership_score = membership_score + 4
        {book.title} returned on time.
    - else:
        ~ membership_score = membership_score - 2
        {book.title} returned late by {book.overdue_days} day(s).
    }
    ~ overdue_fines = overdue_fines - 1
    ~ if_overdue_negative()
}

== function if_overdue_negative() => void ==
{ if overdue_fines < 0:
    ~ overdue_fines = 0
}

== function process_requests(day: int) => void ==
{ if request_cursor >= LEN(requests):
    ~ return
}

~ temp request: BorrowRequest = requests[request_cursor]
{ if request.day == day:
    ~ attempt_borrow(request)
    ~ request_cursor = request_cursor + 1
    ~ process_requests(day)
- else:
    ~ return
}

== function attempt_borrow(request: BorrowRequest) => void ==
~ temp level: int = tier_level(membership_score)
~ temp free_slot: int = find_free_slot(0)
~ temp limit: int = borrow_limit(level)
~ temp tier_name: string = tier_name(level)
~ print_request_intro(request, level, tier_name)
{ if level < request.min_tier:
    ~ denied_requests = denied_requests + 1
    Request denied: {request.title} requires higher tier.
- else:
    { if borrowed_count >= limit:
        ~ denied_requests = denied_requests + 1
        Request denied: {request.title} exceeds concurrent borrowing limit.
    - else:
        { if free_slot < 0:
            ~ denied_requests = denied_requests + 1
            Request denied: no shelf slot currently available.
        - else:
            ~ borrowed_books[free_slot].active = true
            ~ borrowed_books[free_slot].title = request.title
            ~ borrowed_books[free_slot].due_in = request.due_days
            ~ borrowed_books[free_slot].overdue_days = 0
            ~ borrowed_count = borrowed_count + 1
            ~ membership_score = membership_score + request.reward_points
            Borrowing approved.
        }
    }
}

== function print_request_intro(request: BorrowRequest, level: int, tier: string) => void ==
Request {request.title} arrives on day {request.day} for {tier} member.

== function find_free_slot(index: int) => int ==
{ if index >= LEN(borrowed_books):
    ~ return -1
}
~ temp book: BorrowedBook = borrowed_books[index]
{ if !book.active:
    ~ return index
- else:
    ~ return find_free_slot(index + 1)
}

== function borrow_limit(level: int) => int ==
{ if level == 2:
    ~ return 3
- else:
    { if level == 1:
        ~ return 2
    - else:
        ~ return 1
    }
}

== function tier_level(score: int) => int ==
{ if score >= 40:
    ~ return 2
- else:
    { if score >= 25:
        ~ return 1
    - else:
        ~ return 0
    }
}

== function tier_name(level: int) => string ==
{ if level == 2:
    ~ return "Gold"
- else:
    { if level == 1:
        ~ return "Silver"
    - else:
        ~ return "Bronze"
    }
}

== function print_daily_state(day: int) => void ==
Day {day} snapshot
Membership: {tier_name(tier_level(membership_score))} (score {membership_score})
Active borrowed books: {borrowed_count}
Outstanding fines: {overdue_fines}
Privileges: {privilege_summary(tier_level(membership_score))}
~ print_borrowed_books(0)

== function print_borrowed_books(index: int) => void ==
{ if index >= LEN(borrowed_books):
    ~ return
}

~ temp book: BorrowedBook = borrowed_books[index]
{ if book.active:
    {book.title} due in {book.due_in}, overdue {book.overdue_days}
- else:
    Empty slot.
}
~ print_borrowed_books(index + 1)

== function privilege_summary(level: int) => string ==
{ if level == 2:
    ~ return "rare + archive access, up to 3 books"
- else:
    { if level == 1:
        ~ return "standard + rare access, up to 2 books"
    - else:
        ~ return "standard access, up to 1 book"
    }
}

== function print_final_membership_report() => void ==
Library month-end report:
Final tier: {tier_name(tier_level(membership_score))}
Membership score: {membership_score}
Overdue fines accrued: {overdue_fines}
Requests denied: {denied_requests}
Outstanding books:
~ print_borrowed_books(0)
Access status: {privilege_summary(tier_level(membership_score))}
