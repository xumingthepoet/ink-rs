=== module game ===
VAR ability_ids: int[] = [101, 102, 103]
VAR ability_names: Dict<int, string> = %{
    101: "Pierce",
    102: "Healing Rhyme",
    103: "Overwatch"
}
VAR ability_cooldowns: Dict<int, int> = %{
    101: 0,
    102: 0,
    103: 0
}
VAR ability_base_cd: Dict<int, int> = %{
    101: 2,
    102: 3,
    103: 4
}
VAR command_queue: int[] = [101, 101, 102, 103, 102, 101, 103]

== main ==
Cooldown scheduler by ability id.
-> execute_commands(0)
-> DONE

== execute_commands(command_index: int) ==
{ if command_index >= LEN(command_queue):
    All commands processed.
    -> DONE
- else:
    ~ temp ability_id: int = command_queue[command_index]
    Attempting {ability_names[ability_id]}.
    -> cast_if_ready(ability_id) ->
    -> show_cooldowns(0) ->
    -> tick_cooldowns(0) ->
    -> execute_commands(command_index + 1)
}

== cast_if_ready(ability_id: int) ==
{ if ability_cooldowns[ability_id] > 0:
    {ability_names[ability_id]} is on cooldown for {ability_cooldowns[ability_id]} more turns.
- else:
    {ability_names[ability_id]} cast.
    ~ ability_cooldowns[ability_id] = ability_cooldowns[ability_id] + ability_base_cd[ability_id]
    {ability_names[ability_id]} starts a {ability_base_cd[ability_id]}-turn cooldown.
}
->->

== tick_cooldowns(index: int) ==
{ if index >= LEN(ability_ids):
    ->->
- else:
    ~ temp id: int = ability_ids[index]
    { if ability_cooldowns[id] > 0:
        ~ ability_cooldowns[id] = ability_cooldowns[id] - 1
    }
    -> tick_cooldowns(index + 1)
}

== show_cooldowns(index: int) ==
{ if index >= LEN(ability_ids):
    ->->
- else:
    ~ temp id: int = ability_ids[index]
    {ability_names[id]} cooldown now {ability_cooldowns[id]}.
    -> show_cooldowns(index + 1)
}
