=== module game ===

STRUCT CommandMemory {
    command: string
    confidence: int
    attempts: int
    learned: bool
    last_response: string
}

VAR pet_name: string = "Mara"

VAR command_profile: CommandMemory[] = [
    %CommandMemory{
        command: "sit",
        confidence: 12,
        attempts: 0,
        learned: false,
        last_response: "Looks away, then back at you."
    },
    %CommandMemory{
        command: "stay",
        confidence: 25,
        attempts: 0,
        learned: false,
        last_response: "Shifts weight and glances at the trail."
    },
    %CommandMemory{
        command: "fetch",
        confidence: 6,
        attempts: 0,
        learned: false,
        last_response: "Confused but attentive."
    },
    %CommandMemory{
        command: "roll",
        confidence: 2,
        attempts: 0,
        learned: false,
        last_response: "Needs more cues."
    }
]

VAR command_difficulty: Dict<string, int> = %{
    "sit": 1,
    "stay": 2,
    "fetch": 3,
    "roll": 1
}

VAR trainer_focus: int = 2

== main ==
Pet command memory with response tracking.
~ show_commands("Start")
~ train("sit", 1)
~ show_commands("After sit attempt")
~ train("stay", 2)
~ train("sit", 2)
~ train("fetch", 1)
~ train("roll", 2)
~ train("fetch", 3)
~ show_commands("After training sessions")
~ train("sit", 1)
~ show_commands("After final recall")
-> DONE

== function train(command_name: string, reps: int) => void ==
{ if command_name == "sit" || command_name == "stay" || command_name == "fetch" || command_name == "roll":
    ~ temp index: int = command_index(command_name, 0)
    ~ temp entry: CommandMemory = command_profile[index]
    ~ temp gain: int = reps + trainer_focus + command_difficulty[command_name]
    ~ command_profile[index].attempts = entry.attempts + 1
    ~ temp next_confidence: int = entry.confidence + gain
    { if next_confidence > 100:
        ~ next_confidence = 100
    }
    ~ command_profile[index].confidence = next_confidence
    { if !command_profile[index].learned && next_confidence >= 20:
        ~ command_profile[index].learned = true
    }
    ~ command_profile[index].last_response = response_text(command_name, next_confidence, command_profile[index].learned)
    {pet_name} trains command {command_name} (attempt {command_profile[index].attempts}).
    Confidence: {entry.confidence} to {next_confidence}.
    Learned: {if command_profile[index].learned: yes - else: no}.
    Response: {command_profile[index].last_response}
- else:
    {pet_name} has no routine for command {command_name}.
}

== function response_text(command_name: string, confidence: int, is_learned: bool) => string ==
{ if is_learned:
    { if confidence >= 80:
        ~ return command_name + " is executed instantly."
    - else:
        { if confidence >= 40:
            ~ return command_name + " is executed after a brief pause."
        - else:
            ~ return command_name + " is remembered when prompted."
        }
    }
- else:
    { if confidence >= 60:
        ~ return command_name + " is starting to stick."
    - else:
        { if confidence >= 40:
            ~ return command_name + " is recognized with a cue."
        - else:
            ~ return command_name + " is still unclear."
        }
    }
}

== function command_index(name: string, index: int) => int ==
{ if index >= LEN(command_profile):
    ~ return -1
    - else:
        { if command_profile[index].command == name:
            ~ return index
        - else:
            ~ return command_index(name, index + 1)
        }
}

== function show_commands(label: string) => void ==
-- {label} --
~ show_command_entry(0)

== function show_command_entry(index: int) => void ==
{ if index < LEN(command_profile):
    ~ temp memory: CommandMemory = command_profile[index]
    { if memory.learned:
        {memory.command} | attempts {memory.attempts} | conf {memory.confidence} | learned | {memory.last_response}
    - else:
        {memory.command} | attempts {memory.attempts} | conf {memory.confidence} | not learned | {memory.last_response}
    }
    ~ show_command_entry(index + 1)
}
