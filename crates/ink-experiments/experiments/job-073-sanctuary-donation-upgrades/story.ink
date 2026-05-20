=== module game ===

STRUCT Donor {
    name: string
    gift: int
    note: string
}

STRUCT Upgrade {
    service: string
    unlock_threshold: int
    visitor_gain: int
    morale_gain: int
    unlocked: bool
}

VAR donor_queue: Donor[] = [
    %Donor{name: "Mara", gift: 12, note: "for the winter fire."},
    %Donor{name: "Bran", gift: 18, note: "small but sincere."},
    %Donor{name: "Alden", gift: 22, note: "for sick children."},
    %Donor{name: "Priest Halen", gift: 24, note: "for the chapel repair."},
    %Donor{name: "Captain Rook", gift: 30, note: "for extra shelter quilts."}
]

VAR upgrades: Upgrade[] = [
    %Upgrade{service: "Soup Queue", unlock_threshold: 10, visitor_gain: 4, morale_gain: 1, unlocked: false},
    %Upgrade{service: "Medicines", unlock_threshold: 30, visitor_gain: 6, morale_gain: 2, unlocked: false},
    %Upgrade{service: "Night Guard", unlock_threshold: 45, visitor_gain: 5, morale_gain: 1, unlocked: false},
    %Upgrade{service: "Quiet Chapel", unlock_threshold: 60, visitor_gain: 7, morale_gain: 2, unlocked: false},
    %Upgrade{service: "Hearth Library", unlock_threshold: 80, visitor_gain: 10, morale_gain: 3, unlocked: false}
]

VAR sanctuary_fund: int = 0
VAR visitor_base: int = 40
VAR visitor_bonus: int = 0
VAR morale_bonus: int = 0
VAR unlocked_services: int = 0
VAR donation_tally: int = 0

== main ==
The sanctuary hosts a quiet fundraiser.
~ process_donations(0)
~ print_upgrade_board(0)
~ report_impact()
-> DONE

== function process_donations(index: int) => void ==
{ if index >= LEN(donor_queue):
    ~ return
}

~ temp donor: Donor = donor_queue[index]
~ sanctuary_fund = sanctuary_fund + donor.gift
~ donation_tally = donation_tally + 1
Donor {donor.name} contributes {donor.gift} ({donor.note})
Funds now {sanctuary_fund}.
~ try_unlock_upgrade(0)
~ process_donations(index + 1)

== function try_unlock_upgrade(index: int) => void ==
{ if index >= LEN(upgrades):
    ~ return
}

~ temp upgrade: Upgrade = upgrades[index]
{ if upgrade.unlocked:
    ~ try_unlock_upgrade(index + 1)
- else:
    { if sanctuary_fund >= upgrade.unlock_threshold:
        ~ upgrades[index].unlocked = true
        ~ unlocked_services = unlocked_services + 1
        ~ visitor_bonus = visitor_bonus + upgrade.visitor_gain
        ~ morale_bonus = morale_bonus + upgrade.morale_gain
        New service opened: {upgrade.service}.
        {upgrade.visitor_gain} extra daily visitors.
        {upgrade.morale_gain} morale points.
        ~ try_unlock_upgrade(index + 1)
    - else:
        ~ try_unlock_upgrade(index + 1)
    }
}

== function print_upgrade_board(index: int) => void ==
{ if index >= LEN(upgrades):
    ~ return
}

~ temp upgrade: Upgrade = upgrades[index]
~ temp state: string = if_unlocked(upgrade.unlocked)
Service: {upgrade.service} ({state}) needs {upgrade.unlock_threshold}
~ print_upgrade_board(index + 1)

== function if_unlocked(unlocked: bool) => string ==
{ if unlocked:
    ~ return "active"
- else:
    ~ return "locked"
}

== function report_impact() => void ==
Expected visitors today: {visitor_base + visitor_bonus}
Visitor morale index: {8 + morale_bonus}
Sanctuary fund total: {sanctuary_fund}
Donations received: {donation_tally}
Active services: {unlocked_services}

{ if unlocked_services == 0:
    No upgrade unlocked yet.
- else:
    { if unlocked_services >= 4:
        Sanctuary now runs at high capacity and stable routine.
    - else:
        { if unlocked_services == 2:
            Two upgrades stabilize the sanctuary floor.
        - else:
            Sanctuary impact is still expanding.
        }
    }
}
