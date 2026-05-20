=== module game ===

STRUCT MealRecipe {
    name: string
    ingredient_a: string
    amount_a: int
    ingredient_b: string
    amount_b: int
    member_index: int
    duration_days: int
    attack_bonus: int
    stamina_bonus: int
}

STRUCT BuffInstance {
    meal: string
    member_index: int
    remaining_days: int
    attack_bonus: int
    stamina_bonus: int
}

VAR pantry: Dict<string, int> = %{
    "grain": 8,
    "meat": 3,
    "herb": 4,
    "water": 4,
    "oil": 2
}

VAR members: string[] = ["Rin", "Mara", "Joss"]

VAR base_attack: int[] = [9, 8, 7]
VAR base_stamina: int[] = [11, 10, 12]

VAR meal_recipes: MealRecipe[] = [
    %MealRecipe{
        name: "Hearty Stew",
        ingredient_a: "grain",
        amount_a: 3,
        ingredient_b: "meat",
        amount_b: 1,
        member_index: 0,
        duration_days: 2,
        attack_bonus: 2,
        stamina_bonus: 1
    },
    %MealRecipe{
        name: "Herbal Tea",
        ingredient_a: "herb",
        amount_a: 2,
        ingredient_b: "water",
        amount_b: 1,
        member_index: 1,
        duration_days: 3,
        attack_bonus: 1,
        stamina_bonus: 2
    },
    %MealRecipe{
        name: "Oil-Rubbed Grain Cakes",
        ingredient_a: "grain",
        amount_a: 2,
        ingredient_b: "oil",
        amount_b: 1,
        member_index: 2,
        duration_days: 2,
        attack_bonus: 3,
        stamina_bonus: 0
    }
]

VAR buff_slots: BuffInstance[] = [
    %BuffInstance{ meal: "", member_index: -1, remaining_days: 0, attack_bonus: 0, stamina_bonus: 0 },
    %BuffInstance{ meal: "", member_index: -1, remaining_days: 0, attack_bonus: 0, stamina_bonus: 0 },
    %BuffInstance{ meal: "", member_index: -1, remaining_days: 0, attack_bonus: 0, stamina_bonus: 0 }
]

VAR current_day: int = 0

== main ==
Camp fire cooking begins.
~ print_pantry("Kitchen stock")
~ prepare_meals(0)
~ print_pantry("After cooking")
~ run_days(1)
~ print_party_status("End of watch")
-> DONE

== function print_pantry(label: string) => void ==
-- {label} --
Grain: {pantry["grain"]}
Meat: {pantry["meat"]}
Herb: {pantry["herb"]}
Water: {pantry["water"]}
Oil: {pantry["oil"]}

== function can_cook(recipe: MealRecipe) => bool ==
~ return pantry[recipe.ingredient_a] >= recipe.amount_a and pantry[recipe.ingredient_b] >= recipe.amount_b

== function prepare_meals(index: int) => void ==
{ if index >= LEN(meal_recipes):
    ~ return
}
~ temp meal: MealRecipe = meal_recipes[index]
{ if can_cook(meal):
    Stir in ingredients for {meal.name}. {meal.name} is ready for {members[meal.member_index]}.
    ~ pantry[meal.ingredient_a] = pantry[meal.ingredient_a] - meal.amount_a
    ~ pantry[meal.ingredient_b] = pantry[meal.ingredient_b] - meal.amount_b
    ~ apply_buff(meal.member_index, meal.name, meal.duration_days, meal.attack_bonus, meal.stamina_bonus)
    - else:
    Missing ingredients for {meal.name}; it cannot be cooked.
}
~ prepare_meals(index + 1)

== function apply_buff(member_index: int, meal_name: string, duration: int, attack_bonus: int, stamina_bonus: int) => void ==
~ temp filled: bool = place_buff_slot(0, member_index, meal_name, duration, attack_bonus, stamina_bonus)
{ if filled:
    {members[member_index]} receives {meal_name} buff for {duration} days.
    - else:
    No buff slot for {members[member_index]}; {meal_name} fades unused.
}

== function place_buff_slot(slot_index: int, member_index: int, meal_name: string, duration: int, attack_bonus: int, stamina_bonus: int) => bool ==
{ if slot_index >= LEN(buff_slots):
    ~ return false
    - else:
    ~ temp slot: BuffInstance = buff_slots[slot_index]
    { if slot.member_index < 0 or slot.remaining_days <= 0:
        ~ buff_slots[slot_index].meal = meal_name
        ~ buff_slots[slot_index].member_index = member_index
        ~ buff_slots[slot_index].remaining_days = duration
        ~ buff_slots[slot_index].attack_bonus = attack_bonus
        ~ buff_slots[slot_index].stamina_bonus = stamina_bonus
        ~ return true
    - else:
        ~ return place_buff_slot(slot_index + 1, member_index, meal_name, duration, attack_bonus, stamina_bonus)
    }
}

== function run_days(day: int) => void ==
{ if day > 3:
    ~ return
}
Day {day}
~ current_day = day
~ print_party_status("Party status")
~ print_active_buffs("Active buffs")
~ advance_buffs()
~ run_days(day + 1)

== function advance_buffs() => void ==
~ update_buffs(0)

== function update_buffs(slot_index: int) => void ==
{ if slot_index >= LEN(buff_slots):
    ~ return
}
~ temp slot: BuffInstance = buff_slots[slot_index]
{ if slot.member_index >= 0 and slot.remaining_days > 0:
    ~ temp next_remaining: int = slot.remaining_days - 1
    { if next_remaining == 0:
        ~ slot.remaining_days = 0
        ~ buff_slots[slot_index].meal = ""
        ~ buff_slots[slot_index].member_index = -1
        ~ buff_slots[slot_index].remaining_days = 0
        ~ buff_slots[slot_index].attack_bonus = 0
        ~ buff_slots[slot_index].stamina_bonus = 0
        {members[slot.member_index]} loses {slot.meal} buff.
    - else:
        ~ buff_slots[slot_index].remaining_days = next_remaining
    }
}
~ update_buffs(slot_index + 1)

== function print_party_status(label: string) => void ==
-- {label} (Day {current_day}) --
~ print_member_status(0)
~ print_member_status(1)
~ print_member_status(2)

== function print_member_status(member_index: int) => void ==
~ temp attack: int = base_attack[member_index] + total_attack_bonus(member_index)
~ temp stamina: int = base_stamina[member_index] + total_stamina_bonus(member_index)
{members[member_index]}: Attack {attack}, Stamina {stamina}

== function total_attack_bonus(member_index: int) => int ==
~ return total_attack_bonus_from_slot(member_index, 0)

== function total_attack_bonus_from_slot(member_index: int, slot_index: int) => int ==
{ if slot_index >= LEN(buff_slots):
    ~ return 0
    - else:
    ~ temp slot: BuffInstance = buff_slots[slot_index]
    { if slot.member_index == member_index and slot.remaining_days > 0:
        ~ return slot.attack_bonus + total_attack_bonus_from_slot(member_index, slot_index + 1)
    - else:
        ~ return total_attack_bonus_from_slot(member_index, slot_index + 1)
    }
}

== function total_stamina_bonus(member_index: int) => int ==
~ return total_stamina_bonus_from_slot(member_index, 0)

== function total_stamina_bonus_from_slot(member_index: int, slot_index: int) => int ==
{ if slot_index >= LEN(buff_slots):
    ~ return 0
- else:
    ~ temp slot: BuffInstance = buff_slots[slot_index]
    { if slot.member_index == member_index and slot.remaining_days > 0:
        ~ return slot.stamina_bonus + total_stamina_bonus_from_slot(member_index, slot_index + 1)
    - else:
        ~ return total_stamina_bonus_from_slot(member_index, slot_index + 1)
    }
}

== function print_active_buffs(label: string) => void ==
-- {label} --
~ list_buffs(0)

== function list_buffs(slot_index: int) => void ==
{ if slot_index >= LEN(buff_slots):
    ~ return
- else:
    ~ temp slot: BuffInstance = buff_slots[slot_index]
    { if slot.member_index >= 0 and slot.remaining_days > 0:
        {members[slot.member_index]}: {slot.meal} ({slot.remaining_days} days left)
    - else:
        Empty slot.
    }
    ~ list_buffs(slot_index + 1)
}
