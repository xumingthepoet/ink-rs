=== module game ===

ENUM Direction { Up Down Left Right }
ENUM GameState { Playing Dead Won }
ENUM RoomKind { Key Chest Trap Monster Exit }

STRUCT RoomDef {
kind: RoomKind
amount: int
}

CONST width: int = 5
CONST height: int = 5
CONST rows: int[] = [0, 1, 2, 3, 4]
CONST cols: int[] = [0, 1, 2, 3, 4]
CONST initial_visited: bool[] = [
    true, false, false, false, false,
    false, false, false, false, false,
    false, false, false, false, false,
    false, false, false, false, false,
    false, false, false, false, false
]
CONST rooms: Dict<int, RoomDef> = %{
    1: %RoomDef{ kind: RoomKind.Key, amount: 1 },
    2: %RoomDef{ kind: RoomKind.Chest, amount: 2 },
    7: %RoomDef{ kind: RoomKind.Trap, amount: 1 },
    8: %RoomDef{ kind: RoomKind.Monster, amount: 2 },
    9: %RoomDef{ kind: RoomKind.Exit, amount: 0 }
}

VAR player_x: int = 0
VAR player_y: int = 0
VAR hp: int = 6
VAR gold: int = 0
VAR has_key: bool = false
VAR visited: bool[] = initial_visited
VAR moves: int = 0
VAR state: GameState = GameState.Playing
VAR last_message: string = "Find the key, loot safely, and reach the exit."

== main ==
Text roguelike room crawl 5x5.
-> board_prompt

== board_prompt ==
Move {moves}
HP: {hp}
Gold: {gold}
Key: {key_label()}
{ for row in rows:
{map_row(row)}
}
{room_report()}
{last_message}
{ if state != GameState.Playing:
    { if state == GameState.Won:
        Result: escaped.
    - else:
        Result: defeated.
    }
    * Restart
        -> restart
- else:
    * Up
        -> move_player(Direction.Up, "up")
    * Down
        -> move_player(Direction.Down, "down")
    * Left
        -> move_player(Direction.Left, "left")
    * Right
        -> move_player(Direction.Right, "right")
    * Restart
        -> restart
}

== restart ==
~ player_x = 0
~ player_y = 0
~ hp = 6
~ gold = 0
~ has_key = false
~ visited = initial_visited
~ moves = 0
~ state = GameState.Playing
~ last_message = "Restarted."
-> board_prompt

== move_player(direction: Direction, label: string) ==
~ moves += 1
~ temp next_x: int = player_x + direction_dx(direction)
~ temp next_y: int = player_y + direction_dy(direction)
{ if next_x < 0 || next_x >= width || next_y < 0 || next_y >= height:
    ~ last_message = "Move " + label + " hits the boundary."
- else:
    ~ player_x = next_x
    ~ player_y = next_y
    ~ enter_room(cell_index(player_y, player_x), label)
}
-> board_prompt

== function enter_room(index: int, label: string) => void ==
~ temp first_visit: bool = !visited[index]
~ visited[index] = true
{ if DICT_HAS(rooms, index):
    ~ temp room: RoomDef = rooms[index]
    { switch room.kind:
    - RoomKind.Key:
        { if first_visit:
            ~ has_key = true
            ~ last_message = "Move " + label + " finds the iron key."
        - else:
            ~ last_message = "Move " + label + " revisits the empty key room."
        }
    - RoomKind.Chest:
        { if first_visit:
            ~ gold += room.amount
            ~ last_message = "Move " + label + " opens a chest for " + amount_label(room.amount) + " gold."
        - else:
            ~ last_message = "Move " + label + " finds an empty chest."
        }
    - RoomKind.Trap:
        { if first_visit:
            ~ hp -= room.amount
            ~ last_message = "Move " + label + " triggers a trap for " + amount_label(room.amount) + " damage."
        - else:
            ~ last_message = "Move " + label + " steps over a spent trap."
        }
    - RoomKind.Monster:
        { if first_visit:
            ~ hp -= room.amount
            ~ gold += 1
            ~ last_message = "Move " + label + " defeats a monster and takes 1 gold."
        - else:
            ~ last_message = "Move " + label + " crosses an empty lair."
        }
    - else:
        { if has_key:
            ~ state = GameState.Won
            ~ last_message = "Move " + label + " unlocks the exit."
        - else:
            ~ last_message = "Move " + label + " reaches a locked exit."
        }
    }
- else:
    ~ last_message = "Move " + label + " explores a quiet room."
}
{ if hp <= 0:
    ~ state = GameState.Dead
    ~ last_message = last_message + " HP falls to zero."
}

== function map_row(row: int) => string ==
~ temp text: string = ""
{ for col in cols:
    ~ text = text + map_char(row, col)
}
~ return text

== function map_char(row: int, col: int) => string ==
~ temp index: int = cell_index(row, col)
{ if player_x == col && player_y == row:
    ~ return "@"
- else:
    { if DICT_HAS(rooms, index):
        ~ temp room: RoomDef = rooms[index]
        ~ return room_char(room.kind)
    - else:
        { if visited[index]:
            ~ return "."
        - else:
            ~ return "?"
        }
    }
}

== function room_report() => string ==
~ temp text: string = "Special rooms: "
{ for index, room in rooms:
    ~ text = text + cell_name(index) + "=" + room_char(room.kind) + " "
}
~ return text

== function direction_dx(direction: Direction) => int ==
{ switch direction:
- Direction.Left:
    ~ return -1
- Direction.Right:
    ~ return 1
- else:
    ~ return 0
}

== function direction_dy(direction: Direction) => int ==
{ switch direction:
- Direction.Up:
    ~ return -1
- Direction.Down:
    ~ return 1
- else:
    ~ return 0
}

== function cell_index(row: int, col: int) => int ==
~ return row * width + col

== function cell_name(index: int) => string ==
~ return col_label(index - (index / width) * width) + row_label(index / width)

== function room_char(kind: RoomKind) => string ==
{ switch kind:
- RoomKind.Key:
    ~ return "K"
- RoomKind.Chest:
    ~ return "C"
- RoomKind.Trap:
    ~ return "T"
- RoomKind.Monster:
    ~ return "M"
- else:
    ~ return "E"
}

== function key_label() => string ==
{ if has_key:
    ~ return "yes"
- else:
    ~ return "no"
}

== function amount_label(value: int) => string ==
{ switch value:
- 0:
    ~ return "0"
- 1:
    ~ return "1"
- 2:
    ~ return "2"
- else:
    ~ return "3"
}

== function col_label(col: int) => string ==
{ switch col:
- 0:
    ~ return "A"
- 1:
    ~ return "B"
- 2:
    ~ return "C"
- 3:
    ~ return "D"
- else:
    ~ return "E"
}

== function row_label(row: int) => string ==
{ switch row:
- 0:
    ~ return "1"
- 1:
    ~ return "2"
- 2:
    ~ return "3"
- 3:
    ~ return "4"
- else:
    ~ return "5"
}
