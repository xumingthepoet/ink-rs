=== module party ===

STRUCT AllyDef {
    name: string
    role: string
    max_hp: int
    max_mp: int
}

CONST ACTOR_HERO: int = 1
CONST ACTOR_REN: int = 2

CONST allies: Dict<int, AllyDef> = %{
    1: %AllyDef{ name: "Lio", role: "blade", max_hp: 20, max_mp: 6 },
    2: %AllyDef{ name: "Ren", role: "guard", max_hp: 24, max_mp: 2 }
}

VAR active_party: int[] = [ACTOR_HERO]
VAR hp: Dict<int, int> = %{1: 20, 2: 24}
VAR mp: Dict<int, int> = %{1: 6, 2: 2}
VAR exp: Dict<int, int> = %{1: 0, 2: 0}
VAR level: Dict<int, int> = %{1: 1, 2: 1}

== function ally_name(actor_id: int) => string ==
~ return allies[actor_id].name

== function join_ren() => void ==
{ if !is_active(ACTOR_REN):
    ~ ARRAY_PUSH(active_party, ACTOR_REN)
}

== function leave_ren() => void ==
~ temp index: int = party_index(ACTOR_REN)
{ if index >= 0:
    ~ ARRAY_REMOVE(active_party, index)
}

== function rejoin_ren() => void ==
~ join_ren()

== function is_active(actor_id: int) => bool ==
~ return party_index(actor_id) >= 0

== function actor_hp(actor_id: int) => int ==
~ return hp[actor_id]

== function actor_mp(actor_id: int) => int ==
~ return mp[actor_id]

== function actor_exp(actor_id: int) => int ==
~ return exp[actor_id]

== function actor_level(actor_id: int) => int ==
~ return level[actor_id]

== function restore_roster(ren_active: bool) => void ==
~ active_party = [ACTOR_HERO]
{ if ren_active:
    ~ ARRAY_PUSH(active_party, ACTOR_REN)
}

== function set_actor_state(actor_id: int, hp_value: int, mp_value: int, exp_value: int, level_value: int) => void ==
~ hp[actor_id] = hp_value
~ mp[actor_id] = mp_value
~ exp[actor_id] = exp_value
~ level[actor_id] = level_value

== function party_index(actor_id: int) => int ==
~ temp found: int = -1
{ for index, active_id in active_party:
    { if active_id == actor_id:
        ~ found = index
    }
}
~ return found

== function damage_actor(actor_id: int, amount: int) => void ==
~ hp[actor_id] = hp[actor_id] - amount
{ if hp[actor_id] < 0:
    ~ hp[actor_id] = 0
}

== function heal_actor(actor_id: int, amount: int) => void ==
~ hp[actor_id] = hp[actor_id] + amount
{ if hp[actor_id] > allies[actor_id].max_hp:
    ~ hp[actor_id] = allies[actor_id].max_hp
}

== function spend_mp(actor_id: int, amount: int) => bool ==
{ if mp[actor_id] < amount:
    ~ return false
- else:
    ~ mp[actor_id] = mp[actor_id] - amount
    ~ return true
}

== function add_exp(amount: int) => string ==
~ temp text: string = ""
{ for actor_id in active_party:
    ~ exp[actor_id] = exp[actor_id] + amount
    { if exp[actor_id] >= 5 && level[actor_id] == 1:
        ~ level[actor_id] = 2
        { if text == "":
            ~ text = ally_name(actor_id) + " reaches level 2"
        - else:
            ~ text = text + ", " + ally_name(actor_id) + " reaches level 2"
        }
    }
}
{ if text == "":
    ~ return "no level change"
- else:
    ~ return text
}

== function party_summary() => string ==
~ temp text: string = ""
{ for actor_id in active_party:
    { if text == "":
        ~ text = actor_status(actor_id)
    - else:
        ~ text = text + ", " + actor_status(actor_id)
    }
}
~ return text

== function actor_status(actor_id: int) => string ==
~ return ally_name(actor_id) + " HP " + to_str(hp[actor_id]) + "/" + to_str(allies[actor_id].max_hp) + " MP " + to_str(mp[actor_id])
