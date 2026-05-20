=== module game ===

STRUCT CrewMember {
    name: string
    lockpick: int
    driving: int
    stealth: int
    social: int
    assigned_role: string
}

VAR crew: CrewMember[] = [
    %CrewMember{
        name: "Aria",
        lockpick: 9,
        driving: 4,
        stealth: 5,
        social: 4,
        assigned_role: "Unassigned"
    },
    %CrewMember{
        name: "Bran",
        lockpick: 5,
        driving: 9,
        stealth: 3,
        social: 6,
        assigned_role: "Unassigned"
    },
    %CrewMember{
        name: "Nika",
        lockpick: 6,
        driving: 4,
        stealth: 8,
        social: 5,
        assigned_role: "Unassigned"
    },
    %CrewMember{
        name: "Sloane",
        lockpick: 7,
        driving: 5,
        stealth: 4,
        social: 2,
        assigned_role: "Unassigned"
    }
]

VAR role_lock_filled: bool = false
VAR role_driver_filled: bool = false
VAR role_lookout_filled: bool = false
VAR role_infiltrator_filled: bool = false
VAR plan_risk: int = 0

== main ==
Heist planning begins.
~ print_roster()
~ assign_roles(0)
~ review_role_assignments()
~ evaluate_plan_risk(0)
~ resolve_heist_plan()
-> DONE

== function print_roster() => void ==
Crew arrives for the operation.
~ print_crew_member(0)

== function print_crew_member(index: int) => void ==
{ if index < LEN(crew):
    ~ temp agent: CrewMember = crew[index]
    {agent.name}: lockpick {agent.lockpick}, driving {agent.driving}, stealth {agent.stealth}, social {agent.social}.
    ~ print_crew_member(index + 1)
- else:
    End roster.
}

== function assign_roles(index: int) => void ==
{ if index < LEN(crew):
    { if index == 0:
        ~ crew[index].assigned_role = "Lock"
    - else:
        { if index == 1:
            ~ crew[index].assigned_role = "Driver"
        - else:
            { if index == 2:
                ~ crew[index].assigned_role = "Lookout"
            - else:
                ~ crew[index].assigned_role = "Infiltrator"
            }
        }
    }
    ~ temp agent: CrewMember = crew[index]
    Assigned {agent.name} to {agent.assigned_role}.
    ~ assign_roles(index + 1)
- else:
    Assignments complete.
}

== function review_role_assignments() => void ==
Role list after planning:
~ print_role_list(0)

== function print_role_list(index: int) => void ==
{ if index < LEN(crew):
    ~ temp agent: CrewMember = crew[index]
    {agent.name}: {agent.assigned_role}
    ~ print_role_list(index + 1)
- else:
    End role list.
}

== function evaluate_plan_risk(index: int) => void ==
{ if index < LEN(crew):
    ~ temp agent: CrewMember = crew[index]
    { if agent.assigned_role == "Lock":
        ~ role_lock_filled = true
        { if agent.lockpick >= 8:
            Skill check passed for lock.
        - else:
            ~ plan_risk = plan_risk + (8 - agent.lockpick)
            {agent.name} lacks confidence on lockwork.
        }
    - else:
        { if agent.assigned_role == "Driver":
            ~ role_driver_filled = true
            { if agent.driving >= 8:
                Drive lane readiness confirmed.
            - else:
                ~ plan_risk = plan_risk + (8 - agent.driving)
                {agent.name} is not dependable behind the wheel.
            }
        - else:
            { if agent.assigned_role == "Lookout":
                ~ role_lookout_filled = true
                { if agent.stealth >= 7:
                    Lookout posture is strong.
                - else:
                    ~ plan_risk = plan_risk + (7 - agent.stealth)
                    {agent.name} might be seen near the approach lane.
                }
            - else:
                { if agent.assigned_role == "Infiltrator":
                    ~ role_infiltrator_filled = true
                    { if agent.social >= 7:
                        Infiltration deception score is adequate.
                    - else:
                        ~ plan_risk = plan_risk + (7 - agent.social)
                        {agent.name} does not read people well.
                    }
                }
            }
        }
    }
    ~ evaluate_plan_risk(index + 1)
- else:
    All assignments reviewed.
}

== function resolve_heist_plan() => void ==
-- Plan resolution --
Role coverage:
{ if role_lock_filled:
    Lock assigned.
- else:
    Lock role missing.
}
{ if role_driver_filled:
    Driver assigned.
- else:
    Driver role missing.
}
{ if role_lookout_filled:
    Lookout assigned.
- else:
    Lookout role missing.
}
{ if role_infiltrator_filled:
    Infiltrator assigned.
- else:
    Infiltrator role missing.
}

Plan risk score: {plan_risk}.

{ if role_lock_filled && role_driver_filled && role_lookout_filled && role_infiltrator_filled:
    { if plan_risk <= 3:
        Entry route confirmed. The team proceeds with synchronized timing.
    - else:
        Coverage is complete, but risk is too high.
        Heist scrubbed before execution.
    }
- else:
    Coverage is incomplete. Team cannot be deployed safely.
}
