=== module masquerade ===

STRUCT Operative {
    alias: string
    house: string
    disguise: string
    disguise_quality: int
    composure: int
    access_tier: int
    revealed: bool
}

STRUCT RevealCheck {
    scene: string
    operative: int
    scrutiny: int
    rumor: int
    required_access: int
}

VAR operatives: Operative[] = [
    %Operative{
        alias: "Lady Veya",
        house: "House Vael",
        disguise: "Amber Conservator",
        disguise_quality: 15,
        composure: 10,
        access_tier: 2,
        revealed: false
    },
    %Operative{
        alias: "Master Corin",
        house: "House Rook",
        disguise: "Wine Factor",
        disguise_quality: 13,
        composure: 9,
        access_tier: 3,
        revealed: false
    },
    %Operative{
        alias: "Sister Iri",
        house: "House Sable",
        disguise: "Temple Auditor",
        disguise_quality: 11,
        composure: 8,
        access_tier: 3,
        revealed: false
    },
    %Operative{
        alias: "Captain Rohe",
        house: "House Varn",
        disguise: "Retired Cavalry",
        disguise_quality: 9,
        composure: 7,
        access_tier: 2,
        revealed: false
    }
]

VAR checks: RevealCheck[] = [
    %RevealCheck{scene: "Lantern Court", operative: 0, scrutiny: 7, rumor: 2, required_access: 1},
    %RevealCheck{scene: "Archive Gallery", operative: 1, scrutiny: 11, rumor: 4, required_access: 2},
    %RevealCheck{scene: "Amber Salon", operative: 2, scrutiny: 9, rumor: 5, required_access: 2},
    %RevealCheck{scene: "Cardinal Stair", operative: 3, scrutiny: 14, rumor: 6, required_access: 3},
    %RevealCheck{scene: "Whisper Balcony", operative: 0, scrutiny: 12, rumor: 7, required_access: 2},
    %RevealCheck{scene: "Signet Bridge", operative: 1, scrutiny: 15, rumor: 8, required_access: 3},
    %RevealCheck{scene: "Moon Vault Antechamber", operative: 2, scrutiny: 22, rumor: 11, required_access: 3},
    %RevealCheck{scene: "Final Toast Hall", operative: 3, scrutiny: 10, rumor: 3, required_access: 2}
]

VAR suspicion: Dict<string, int> = %{
    "Lady Veya": 2,
    "Master Corin": 3,
    "Sister Iri": 5,
    "Captain Rohe": 4
}

VAR successful_entries: int = 0
VAR blocked_entries: int = 0
VAR reveal_count: int = 0
VAR elite_entries: int = 0
VAR highest_suspicion_alias: string = "none"
VAR highest_suspicion_value: int = 0

== main ==
Masquerade identity tracking begins.
~ run_checks(0)
~ print_final_summary()
-> DONE

== function run_checks(index: int) => void ==
{ if index >= LEN(checks):
    Reveal checks complete.
- else:
    ~ temp check: RevealCheck = checks[index]
    ~ evaluate_check(check)
    ~ run_checks(index + 1)
}

== function evaluate_check(check: RevealCheck) => void ==
~ temp operative: Operative = operatives[check.operative]
~ temp current_suspicion: int = suspicion[operative.alias]

Scene: {check.scene}
Alias {operative.alias} appears as {operative.disguise}.
Required access tier: {check.required_access}

{ if operative.revealed:
    Already revealed. Access denied immediately.
    ~ blocked_entries = blocked_entries + 1
    ~ suspicion[operative.alias] = current_suspicion + 4
- else:
    ~ temp risk: int = reveal_risk(operative, check, current_suspicion)
    ~ temp suspicion_gain: int = (risk / 3) + (check.rumor / 2)
    ~ temp suspicion_next: int = current_suspicion + suspicion_gain
    ~ suspicion[operative.alias] = suspicion_next
    ~ operatives[check.operative].composure = clamp_composure(operative.composure - (risk / 8))

    Reveal risk: {risk}
    Suspicion climbs by {suspicion_gain} to {suspicion_next}.

    { if risk >= 18 || suspicion_next >= 24:
        Reveal check fails for {operative.alias}. True house exposed.
        ~ operatives[check.operative].revealed = true
        ~ blocked_entries = blocked_entries + 1
        ~ reveal_count = reveal_count + 1
    - else:
        { if operative.access_tier >= check.required_access && risk <= 12:
            Social access granted at {check.scene}.
            ~ successful_entries = successful_entries + 1
            { if check.required_access >= 3:
                ~ elite_entries = elite_entries + 1
            }
        - else:
            Access stalls under suspicion review.
            ~ blocked_entries = blocked_entries + 1
            ~ suspicion[operative.alias] = suspicion[operative.alias] + 3
        }
    }
}

== function reveal_risk(operative: Operative, check: RevealCheck, current_suspicion: int) => int ==
~ temp risk: int = check.scrutiny + check.rumor - operative.disguise_quality - (operative.composure / 2) + (current_suspicion / 2)
{ if risk < 0:
    ~ return 0
- else:
    ~ return risk
}

== function print_final_summary() => void ==
Masquerade summary.
Successful entries: {successful_entries}
Blocked entries: {blocked_entries}
Reveals: {reveal_count}
Elite access entries: {elite_entries}
~ highest_suspicion_alias = "none"
~ highest_suspicion_value = 0
~ print_operatives(0)
Peak suspicion: {highest_suspicion_alias} ({highest_suspicion_value})
~ print_outcome()

== function print_operatives(index: int) => void ==
{ if index >= LEN(operatives):
    ~ return
- else:
    ~ temp operative: Operative = operatives[index]
    ~ temp level: int = suspicion[operative.alias]
    { if level > highest_suspicion_value:
        ~ highest_suspicion_value = level
        ~ highest_suspicion_alias = operative.alias
    }
    { if operative.revealed:
        {operative.alias} of {operative.house} is revealed. Suspicion {level}, composure {operative.composure}
    - else:
        {operative.alias} stays covered. Suspicion {level}, composure {operative.composure}
    }
    ~ print_operatives(index + 1)
}

== function print_outcome() => void ==
{ if reveal_count >= 2 || blocked_entries > successful_entries:
    Outcome: masquerade network collapses under scrutiny.
- else:
    { if elite_entries >= 2 && reveal_count == 0:
        Outcome: social infiltration succeeds across elite circles.
    - else:
        Outcome: social access is partial and identities are strained.
    }
}

== function clamp_composure(value: int) => int ==
{ if value < 0:
    ~ return 0
- else:
    { if value > 12:
        ~ return 12
    - else:
        ~ return value
    }
}
