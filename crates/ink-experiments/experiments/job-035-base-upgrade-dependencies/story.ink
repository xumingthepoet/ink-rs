=== module game ===

STRUCT Upgrade {
    id: string
    prereq_a: string
    prereq_b: string
    gold_cost: int
    wood_cost: int
    stone_cost: int
    ore_cost: int
    attack_bonus: int
    defense_bonus: int
    scouting_bonus: int
    acquired: bool
}

VAR gold: int = 2
VAR wood: int = 0
VAR stone: int = 0
VAR ore: int = 0

VAR base_attack: int = 5
VAR base_defense: int = 5
VAR base_scouting: int = 0
VAR current_day: int = 0

VAR upgrades: Upgrade[] = [
    %Upgrade{
        id: "Foundations",
        prereq_a: "",
        prereq_b: "",
        gold_cost: 6,
        wood_cost: 4,
        stone_cost: 2,
        ore_cost: 1,
        attack_bonus: 0,
        defense_bonus: 3,
        scouting_bonus: 0,
        acquired: false
    },
    %Upgrade{
        id: "Watchtower",
        prereq_a: "Foundations",
        prereq_b: "",
        gold_cost: 5,
        wood_cost: 2,
        stone_cost: 2,
        ore_cost: 0,
        attack_bonus: 0,
        defense_bonus: 1,
        scouting_bonus: 4,
        acquired: false
    },
    %Upgrade{
        id: "Barracks",
        prereq_a: "Foundations",
        prereq_b: "",
        gold_cost: 7,
        wood_cost: 4,
        stone_cost: 2,
        ore_cost: 1,
        attack_bonus: 5,
        defense_bonus: 1,
        scouting_bonus: 0,
        acquired: false
    },
    %Upgrade{
        id: "Fortified Gate",
        prereq_a: "Foundations",
        prereq_b: "Watchtower",
        gold_cost: 4,
        wood_cost: 2,
        stone_cost: 0,
        ore_cost: 1,
        attack_bonus: 1,
        defense_bonus: 4,
        scouting_bonus: 1,
        acquired: false
    },
    %Upgrade{
        id: "Armory",
        prereq_a: "Barracks",
        prereq_b: "Watchtower",
        gold_cost: 5,
        wood_cost: 2,
        stone_cost: 1,
        ore_cost: 2,
        attack_bonus: 4,
        defense_bonus: 0,
        scouting_bonus: 0,
        acquired: false
    }
]

== main ==
Fortress upgrade dependency simulation.
~ print_resources("Day 0")
~ print_upgrades_status("Initial upgrade status")
~ run_day(1)
~ print_base_stats("Final base stats")
-> DONE

== function run_day(day: int) => void ==
~ current_day = day
{ if day <= 7:
    ~ print_resources("Day {day}")
    ~ collect_supplies(day)
    ~ print_resources("After day {day} logistics")
    ~ print_upgrades_status("Evaluating upgrades on day {day}")
    ~ apply_passes_until_locked()
    ~ print_base_stats("End day {day}")
    ~ run_day(day + 1)
- else:
    Construction period ended.
}

== function collect_supplies(day: int) => void ==
~ gold = gold + 4
~ wood = wood + 3
~ stone = stone + 1
~ ore = ore + 1
-- Supply report --
Day {day} delivered:
4 gold
3 wood
1 stone
1 ore

== function apply_passes_until_locked() => void ==
~ temp changed: bool = apply_upgrades_pass(0)
{ if changed:
    A construction wave completed.
    ~ apply_passes_until_locked()
}

== function apply_upgrades_pass(index: int) => bool ==
{ if index >= LEN(upgrades):
    ~ return false
- else:
    ~ temp changed_rest: bool = apply_upgrade(index)
    ~ temp changed_next: bool = apply_upgrades_pass(index + 1)
    { if changed_rest:
        ~ return true
    - else:
        ~ return changed_next
    }
}

== function apply_upgrade(index: int) => bool ==
~ temp upgrade: Upgrade = upgrades[index]
{ if upgrade.acquired:
    Already acquired: {upgrade.id}.
    ~ return false
- else:
    { if !prerequisites_met(index):
        Locked: {upgrade.id} waits for prerequisites.
        ~ return false
    - else:
        { if !has_budget(index):
            Blocked: {upgrade.id} lacks funds ({upgrade.gold_cost} gold, {upgrade.wood_cost} wood, {upgrade.stone_cost} stone, {upgrade.ore_cost} ore).
            ~ return false
        - else:
            ~ buy_upgrade(index)
            ~ return true
        }
    }
}

== function prerequisites_met(index: int) => bool ==
~ temp upgrade: Upgrade = upgrades[index]
{ if upgrade.prereq_a == "":
    ~ return true
- else:
    { if upgrade.prereq_b == "":
        ~ return upgrade_owned(upgrade.prereq_a)
    - else:
        ~ return upgrade_owned(upgrade.prereq_a) && upgrade_owned(upgrade.prereq_b)
    }
}

== function upgrade_owned(name: string) => bool ==
~ temp index: int = upgrade_index(name, 0)
{ if index < 0:
    ~ return false
- else:
    ~ return upgrades[index].acquired
}

== function upgrade_index(name: string, index: int) => int ==
{ if index >= LEN(upgrades):
    ~ return -1
- else:
    { if upgrades[index].id == name:
        ~ return index
    - else:
        ~ return upgrade_index(name, index + 1)
    }
}

== function has_budget(index: int) => bool ==
~ temp upgrade: Upgrade = upgrades[index]
{ if gold >= upgrade.gold_cost && wood >= upgrade.wood_cost && stone >= upgrade.stone_cost && ore >= upgrade.ore_cost:
    ~ return true
- else:
    ~ return false
}

== function buy_upgrade(index: int) => void ==
~ temp upgrade: Upgrade = upgrades[index]
~ gold = gold - upgrade.gold_cost
~ wood = wood - upgrade.wood_cost
~ stone = stone - upgrade.stone_cost
~ ore = ore - upgrade.ore_cost
~ upgrades[index].acquired = true
~ base_attack = base_attack + upgrade.attack_bonus
~ base_defense = base_defense + upgrade.defense_bonus
~ base_scouting = base_scouting + upgrade.scouting_bonus
Acquired {upgrade.id} this cycle.
Stats: atk {base_attack}, def {base_defense}, scouting {base_scouting}.

== function print_resources(label: string) => void ==
-- {label} --
Current stock: {gold} gold, {wood} wood, {stone} stone, {ore} ore

== function print_base_stats(label: string) => void ==
-- {label} --
Attack: {base_attack}.
Defense: {base_defense}.
Scouting: {base_scouting}.

== function print_upgrades_status(label: string) => void ==
-- {label} --
~ print_upgrade_entry(0)

== function print_upgrade_entry(index: int) => void ==
{ if index < LEN(upgrades):
    ~ temp upgrade: Upgrade = upgrades[index]
    { if upgrade.acquired:
        {upgrade.id}: active
    - else:
        {upgrade.id}: unavailable
    }
    ~ print_upgrade_entry(index + 1)
}
