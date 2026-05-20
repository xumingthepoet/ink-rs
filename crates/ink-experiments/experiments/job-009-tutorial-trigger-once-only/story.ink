=== module game ===
VAR tutorials_seen: Dict<string, bool> = %{
    "movement": false,
    "combat": false,
    "inventory": false,
    "magic": false,
    "dialog": false
}

VAR tutorial_text: Dict<string, string> = %{
    "movement": "You move with IJKL or WASD.",
    "combat": "Press the Attack key when an enemy faces you.",
    "inventory": "Inventory toggles with I and uses with number keys.",
    "magic": "Double-tap to chain spells.",
    "dialog": "Esc saves and opens a pause menu."
}

VAR event_order: string[] = ["movement", "combat", "movement", "inventory", "dialog", "magic", "combat"]

== main ==
Tutorial once-only registry:
-> trigger_tutorials(0) ->
-> DONE

== trigger_tutorials(index: int) ==
{ if index >= LEN(event_order):
    Tutorial flow complete.
    ->->
- else:
    ~ temp topic: string = event_order[index]
    { if tutorials_seen[topic]:
        {tutorial_text[topic]} (already seen)
    - else:
        {tutorial_text[topic]}
        ~ tutorials_seen[topic] = true
    }
    -> trigger_tutorials(index + 1)
}
