=== module game ===

STRUCT Student {
    name: string
    focus: int
    fatigue: int
    fatigue_limit: int
    progress: int
    mastery: int
    assignments: int
    status: string
}

STRUCT Lesson {
    name: string
    difficulty: int
    gain: int
    fatigue_cost: int
}

VAR students: Student[] = [
    %Student{
        name: "Aiden",
        focus: 12,
        fatigue: 0,
        fatigue_limit: 45,
        progress: 0,
        mastery: 0,
        assignments: 0,
        status: "unassigned"
    },
    %Student{
        name: "Brielle",
        focus: 16,
        fatigue: 0,
        fatigue_limit: 48,
        progress: 0,
        mastery: 0,
        assignments: 0,
        status: "unassigned"
    },
    %Student{
        name: "Chen",
        focus: 9,
        fatigue: 0,
        fatigue_limit: 42,
        progress: 0,
        mastery: 0,
        assignments: 0,
        status: "unassigned"
    },
    %Student{
        name: "Dara",
        focus: 14,
        fatigue: 0,
        fatigue_limit: 46,
        progress: 0,
        mastery: 0,
        assignments: 0,
        status: "unassigned"
    }
]

VAR lessons: Lesson[] = [
    %Lesson{name: "Rune Glyphs", difficulty: 3, gain: 26, fatigue_cost: 6},
    %Lesson{name: "Mentor Dialogue", difficulty: 5, gain: 18, fatigue_cost: 5},
    %Lesson{name: "Pattern Read", difficulty: 4, gain: 22, fatigue_cost: 7},
    %Lesson{name: "Live Drill", difficulty: 8, gain: 32, fatigue_cost: 10}
]

VAR session_plan: int[] = [0, 1, 2, 3, 1]
VAR session_load: Dict<int, int> = %{1: 2, 2: 1, 3: 3, 4: 2, 5: 4}
VAR session_count: int = 5
VAR total_mastery_points: int = 0
VAR total_lessons_taught: int = 0
VAR total_burnout: int = 0

== main ==
Mentor training cycle starts.
~ announce_roster("Initial")
~ train_session(1)
~ announce_roster("Final")
~ print_training_summary()
-> DONE

== function announce_roster(label: string) => void ==
Mentor roster: {label}
~ announce_student_rows(0)

== function announce_student_rows(index: int) => void ==
{ if index >= LEN(students):
    ~ return
- else:
    ~ temp student: Student = students[index]
    {student.name} progress {student.progress} fatigue {student.fatigue} mastery {student.mastery} status {student.status} assignments {student.assignments}
    ~ announce_student_rows(index + 1)
}

== function train_session(session: int) => void ==
{ if session > session_count:
    ~ return
- else:
    Training session {session}
    ~ conduct_student_batch(0, session)
    ~ print_session_summary(session)
    ~ train_session(session + 1)
}

== function conduct_student_batch(index: int, session: int) => void ==
{ if index >= LEN(students):
    ~ return
- else:
    ~ train_student(index, session)
    ~ conduct_student_batch(index + 1, session)
}

== function train_student(index: int, session: int) => void ==
~ temp student: Student = students[index]
{ if student.status == "burned_out":
    {student.name} cannot continue training this cycle.
- else:
    ~ temp lesson_slot: int = session_plan[session - 1] + student.focus / 6
    ~ temp lesson_index: int = lesson_slot
    { if lesson_index < 0:
        ~ lesson_index = 0
    - else:
        { if lesson_index > 3:
            ~ lesson_index = 3
        }
    }
    ~ temp assigned: Lesson = lessons[lesson_index]
    ~ temp load: int = session_load[session]
    ~ temp base_gain: int = assigned.gain + student.focus - (assigned.difficulty * 3) - load
    { if base_gain < 1:
        ~ base_gain = 1
    }
    ~ temp next_fatigue: int = student.fatigue + assigned.fatigue_cost + load

    ~ temp next_progress: int = student.progress + base_gain
    ~ total_lessons_taught = total_lessons_taught + 1
    ~ temp next_status: string = "unassigned"
    { if next_fatigue > student.fatigue_limit:
        ~ total_burnout = total_burnout + 1
        ~ students[index].fatigue = student.fatigue_limit
        ~ students[index].status = "burned_out"
        ~ students[index].assignments = student.assignments + 1
        Burnout prevented {student.name} from finishing {assigned.name} in this session.
    - else:
        ~ students[index].fatigue = next_fatigue
        ~ students[index].progress = next_progress
        ~ students[index].assignments = student.assignments + 1
        ~ students[index].status = lesson_level(next_progress)
        ~ temp next_mastery: int = mastery_from_progress(next_progress)
        { if next_mastery > student.mastery:
            ~ total_mastery_points = total_mastery_points + (next_mastery - student.mastery)
            ~ students[index].mastery = next_mastery
            ~ next_status = "promoted"
        - else:
            ~ students[index].mastery = student.mastery
        }
        { if next_status == "promoted":
            {student.name} unlocks {students[index].status}.
        - else:
            {student.name} studies {assigned.name}.
        }
        {student.name} gains {base_gain} progress (now {next_progress}), fatigue {next_fatigue}.
    }
}

== function lesson_level(progress: int) => string ==
{ if progress >= 260:
    ~ return "master"
- else:
    { if progress >= 170:
        ~ return "advanced"
    - else:
        { if progress >= 90:
            ~ return "intermediate"
        - else:
            ~ return "novice"
        }
    }
}

== function mastery_from_progress(progress: int) => int ==
{ if progress >= 260:
    ~ return 3
- else:
    { if progress >= 170:
        ~ return 2
    - else:
        { if progress >= 90:
            ~ return 1
        - else:
            ~ return 0
        }
    }
}

== function print_session_summary(session: int) => void ==
Session {session} check.
~ temp mentor_novice: int = count_status("novice", 0)
~ temp mentor_intermediate: int = count_status("intermediate", 0)
~ temp mentor_advanced: int = count_status("advanced", 0)
~ temp mentor_master: int = count_status("master", 0)
~ temp mentor_burned: int = count_status("burned_out", 0)
Current standings:
Novice {mentor_novice}
Intermediate {mentor_intermediate}
Advanced {mentor_advanced}
Master {mentor_master}
Burned out {mentor_burned}

== function count_status(target: string, index: int) => int ==
{ if index >= LEN(students):
    ~ return 0
- else:
    { if students[index].status == target:
        ~ return 1 + count_status(target, index + 1)
    - else:
        ~ return count_status(target, index + 1)
    }
}

== function print_training_summary() => void ==
Final training outcome.
~ temp master_count: int = count_status("master", 0)
~ temp advanced_count: int = count_status("advanced", 0)
~ temp novice_count: int = count_status("novice", 0)
~ temp burned_count: int = count_status("burned_out", 0)
Total lessons taught: {total_lessons_taught}
Total mastery points unlocked: {total_mastery_points}
Burnout events: {burned_count}
Masters: {master_count}
Advanced: {advanced_count}
Novices: {novice_count}
Lesson log:
{ if burned_count > 0:
    Burnout risk must be managed before planning additional cohorts.
- else:
    No student burned out during this training track.
}
