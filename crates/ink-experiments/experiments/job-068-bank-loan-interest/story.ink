=== module game ===

STRUCT BorrowerLoan {
    borrower: string
    principal: int
    rate: int
    collateral: int
    base_payment: int
    risk_buffer: int
    missed_payments: int
    status: string
}

VAR loans: BorrowerLoan[] = [
    %BorrowerLoan{
        borrower: "Nera",
        principal: 2500,
        rate: 8,
        collateral: 3600,
        base_payment: 380,
        risk_buffer: 20,
        missed_payments: 0,
        status: "active"
    },
    %BorrowerLoan{
        borrower: "Joss",
        principal: 6500,
        rate: 12,
        collateral: 5000,
        base_payment: 280,
        risk_buffer: 6,
        missed_payments: 0,
        status: "active"
    },
    %BorrowerLoan{
        borrower: "Hale",
        principal: 1800,
        rate: 10,
        collateral: 2500,
        base_payment: 290,
        risk_buffer: 24,
        missed_payments: 0,
        status: "active"
    },
    %BorrowerLoan{
        borrower: "Miro",
        principal: 2700,
        rate: 7,
        collateral: 3000,
        base_payment: 260,
        risk_buffer: 18,
        missed_payments: 0,
        status: "active"
    }
]

VAR policy_hardening: int = 0
VAR payment_adjustment_by_month: Dict<int, int> = %{1: 20, 2: 10, 3: 12, 4: 6}
VAR total_interest_collected: int = 0
VAR total_repaid: int = 0
VAR total_defaults: int = 0
VAR total_closed: int = 0
VAR reserve_buffer: int = 10000
VAR cycles: int = 3

== main ==
Bank loan portfolio stress test begins.
~ print_portfolio("Opening books")
~ process_cycle(1)
~ print_portfolio("Closing books")
~ print_final_review()
-> DONE

== function print_portfolio(label: string) => void ==
Borrower ledger: {label}
~ print_ledger_rows(0)

== function print_ledger_rows(index: int) => void ==
{ if index >= LEN(loans):
    ~ return
- else:
    ~ temp loan: BorrowerLoan = loans[index]
    {loan.borrower} | principal {loan.principal} | rate {loan.rate} | collateral {loan.collateral} | status {loan.status} | missed {loan.missed_payments}
    ~ print_ledger_rows(index + 1)
}

== function process_cycle(month: int) => void ==
{ if month > cycles:
    ~ return
- else:
    Cycle month {month}
    Policy hardening: {policy_hardening}
    ~ process_loan_set(0, month)
    ~ print_cycle_summary(month)
    { if month == 2:
        ~ policy_hardening = policy_hardening + 1
    }
    ~ process_cycle(month + 1)
}

== function process_loan_set(index: int, month: int) => void ==
{ if index >= LEN(loans):
    ~ return
- else:
    ~ process_loan(index, month)
    ~ process_loan_set(index + 1, month)
}

== function process_loan(index: int, month: int) => void ==
{ if index >= LEN(loans):
    ~ return
- else:
    ~ temp loan: BorrowerLoan = loans[index]
    { if loan.status == "defaulted" or loan.status == "closed":
        {loan.borrower} is no longer active.
    - else:
        ~ process_active_loan(index, month)
    }
}

== function process_active_loan(index: int, month: int) => void ==
~ temp loan: BorrowerLoan = loans[index]
~ temp effective_rate: int = loan.rate + policy_hardening
~ temp interest: int = (loan.principal * effective_rate) / 100
~ temp buffer_payment: int = loan.risk_buffer / 2
~ temp cyc_payment: int = payment_adjustment_by_month[month]
~ temp raw_payment: int = loan.base_payment + cyc_payment + buffer_payment + policy_hardening
~ temp principal_with_interest: int = loan.principal + interest
~ temp payment: int = raw_payment
~ temp missed: int = loan.missed_payments

~ total_interest_collected = total_interest_collected + interest
~ total_repaid = total_repaid + payment
~ reserve_buffer = reserve_buffer + payment

{ if payment > principal_with_interest:
    ~ payment = principal_with_interest
}

{ if payment < interest:
    ~ missed = missed + 1
}

~ loans[index].missed_payments = missed
~ temp next_principal: int = principal_with_interest - payment

{ if next_principal <= 0:
    ~ total_closed = total_closed + 1
    ~ loans[index].principal = 0
    ~ loans[index].status = "closed"
    {loan.borrower} paid all obligations. Status closed.
- else:
    ~ temp ratio: int = (next_principal * 100) / loan.collateral
    { if missed >= 2 and ratio >= 145:
        ~ total_defaults = total_defaults + 1
        ~ loans[index].principal = 0
        ~ loans[index].status = "defaulted"
        ~ reserve_buffer = reserve_buffer + loan.collateral / 2
        ~ print_defaulted_effect(loan.borrower, next_principal, ratio)
    - else:
        ~ loans[index].principal = next_principal
        { if missed > 0:
            { if ratio >= 135:
                ~ loans[index].status = "watch"
            - else:
                ~ loans[index].status = "active"
            }
        - else:
            { if ratio >= 140:
                ~ loans[index].status = "watch"
            - else:
                ~ loans[index].status = "active"
            }
        }
        ~ print_loan_status(loan.borrower, payment, interest, next_principal, ratio, loans[index].status)
    }
}

== function print_loan_status(borrower: string, payment: int, interest: int, principal: int, ratio: int, status: string) => void ==
{ borrower } payment {payment}, interest {interest}, remaining {principal}, ratio {ratio}%, status {status}.

== function print_defaulted_effect(borrower: string, principal: int, ratio: int) => void ==
{borrower} default triggered at ratio {ratio}% with leftover balance {principal}. Collateral seizure recovers partial value.

== function print_cycle_summary(month: int) => void ==
Cycle {month} review.
~ temp watch_count: int = count_status("watch", 0)
~ temp default_count: int = count_status("defaulted", 0)
~ temp closed_count: int = count_status("closed", 0)
Active loans: {LEN(loans) - watch_count - default_count - closed_count}
Watch: {watch_count}
Defaulted: {default_count}
Closed: {closed_count}
    ~ print_monthly_ledger(month)

== function count_status(target: string, index: int) => int ==
{ if index >= LEN(loans):
    ~ return 0
- else:
    { if loans[index].status == target:
        ~ return 1 + count_status(target, index + 1)
    - else:
        ~ return count_status(target, index + 1)
    }
}

== function print_monthly_ledger(month: int) => void ==
Month {month} ledger.
~ print_ledger_rows(0)

== function print_final_review() => void ==
Portfolio final review.
Total interest collected: {total_interest_collected}
Total principal repaid: {total_repaid}
Defaults recorded: {total_defaults}
Closed loans: {total_closed}
Reserve cushion: {reserve_buffer}
~ temp defaults: int = count_status("defaulted", 0)
~ temp closes: int = count_status("closed", 0)
~ temp watch: int = count_status("watch", 0)
~ temp active: int = LEN(loans) - defaults - closes - watch
Balance by status:
Active {active}
Watch {watch}
Defaulted {defaults}
Closed {closes}
{ if defaults > 0:
    Credit risk outcome is non-zero.
- else:
    Credit conditions stayed healthy for all borrowers.
}
