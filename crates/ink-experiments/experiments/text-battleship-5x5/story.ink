=== module game ===

ENUM GameState { Playing Won }

STRUCT ShipCell {
x: int
y: int
ship_id: int
}

CONST width: int = 5
CONST height: int = 5
CONST rows: int[] = [0, 1, 2, 3, 4]
CONST cols: int[] = [0, 1, 2, 3, 4]
CONST fleet: ShipCell[] = [
    %ShipCell{ x: 0, y: 0, ship_id: 1 },
    %ShipCell{ x: 1, y: 0, ship_id: 1 },
    %ShipCell{ x: 3, y: 2, ship_id: 2 },
    %ShipCell{ x: 3, y: 3, ship_id: 2 },
    %ShipCell{ x: 3, y: 4, ship_id: 2 }
]
CONST ship_names: Dict<int, string> = %{1: "skiff", 2: "barge"}
CONST ship_sizes: Dict<int, int> = %{1: 2, 2: 3}
CONST initial_shots: int[] = [
    0, 0, 0, 0, 0,
    0, 0, 0, 0, 0,
    0, 0, 0, 0, 0,
    0, 0, 0, 0, 0,
    0, 0, 0, 0, 0
]

VAR shots: int[] = initial_shots
VAR shot_count: int = 0
VAR state: GameState = GameState.Playing
VAR last_message: string = "Choose a coordinate to fire."

== main ==
Text battleship 5x5.
-> board_prompt

== board_prompt ==
Shots: {shot_count}
  A B C D E
{ for row in rows:
{row_label(row)} {row_cells(row)}
}
{ for ship_id, size in ship_sizes:
Ship {ship_names[ship_id]}: {ship_status(ship_id, size)}
}
{last_message}
{ if state == GameState.Won:
    Result: fleet sunk.
    * Restart
        -> restart
- else:
    * Row 1
        -> choose_column(0)
    * Row 2
        -> choose_column(1)
    * Row 3
        -> choose_column(2)
    * Row 4
        -> choose_column(3)
    * Row 5
        -> choose_column(4)
    * Restart
        -> restart
}

== choose_column(row: int) ==
Choose column for row {row_label(row)}.
* A{row_label(row)}
    -> fire(row, 0)
* B{row_label(row)}
    -> fire(row, 1)
* C{row_label(row)}
    -> fire(row, 2)
* D{row_label(row)}
    -> fire(row, 3)
* E{row_label(row)}
    -> fire(row, 4)
* Back
    -> board_prompt

== restart ==
~ shots = initial_shots
~ shot_count = 0
~ state = GameState.Playing
~ last_message = "Restarted."
-> board_prompt

== fire(row: int, col: int) ==
~ temp index: int = cell_index(row, col)
{ if shots[index] != 0:
    ~ last_message = cell_name(row, col) + " was already targeted."
- else:
    ~ shot_count += 1
    ~ temp ship_id: int = ship_id_at(col, row)
    { if ship_id == 0:
        ~ shots[index] = 1
        ~ last_message = "Miss at " + cell_name(row, col) + "."
    - else:
        ~ shots[index] = 2
        { if is_ship_sunk(ship_id):
            ~ last_message = "Hit at " + cell_name(row, col) + "; " + ship_names[ship_id] + " sunk."
        - else:
            ~ last_message = "Hit at " + cell_name(row, col) + "."
        }
    }
}
~ check_win()
-> board_prompt

== function check_win() => void ==
{ if state == GameState.Playing && all_ships_sunk():
    ~ state = GameState.Won
    ~ last_message = last_message + " Enemy fleet destroyed."
}

== function row_cells(row: int) => string ==
~ temp text: string = ""
{ for col in cols:
    { if col == 0:
        ~ text = shot_char(row, col)
    - else:
        ~ text = text + " " + shot_char(row, col)
    }
}
~ return text

== function shot_char(row: int, col: int) => string ==
{ switch shots[cell_index(row, col)]:
- 1:
    ~ return "x"
- 2:
    ~ return "H"
- else:
    ~ return "?"
}

== function ship_status(ship_id: int, size: int) => string ==
{ if is_ship_sunk(ship_id):
    ~ return "sunk"
- else:
    ~ return "afloat " + count_label(hit_count(ship_id)) + "/" + count_label(size)
}

== function ship_id_at(x: int, y: int) => int ==
~ temp found: int = 0
{ for cell in fleet:
    { if cell.x == x && cell.y == y:
        ~ found = cell.ship_id
    }
}
~ return found

== function hit_count(ship_id: int) => int ==
~ temp count: int = 0
{ for cell in fleet:
    { if cell.ship_id == ship_id && shots[cell_index(cell.y, cell.x)] == 2:
        ~ count += 1
    }
}
~ return count

== function is_ship_sunk(ship_id: int) => bool ==
~ return hit_count(ship_id) >= ship_sizes[ship_id]

== function all_ships_sunk() => bool ==
~ temp sunk: int = 0
{ for ship_id, size in ship_sizes:
    { if hit_count(ship_id) >= size:
        ~ sunk += 1
    }
}
~ return sunk == DICT_SIZE(ship_sizes)

== function cell_index(row: int, col: int) => int ==
~ return row * width + col

== function cell_name(row: int, col: int) => string ==
~ return col_label(col) + row_label(row)

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

== function count_label(value: int) => string ==
{ switch value:
- 0:
    ~ return "0"
- 1:
    ~ return "1"
- 2:
    ~ return "2"
- 3:
    ~ return "3"
- 4:
    ~ return "4"
- else:
    ~ return "5"
}
