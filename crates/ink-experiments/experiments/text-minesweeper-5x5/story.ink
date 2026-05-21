=== module game ===

ENUM ActionMode { Reveal Flag }
ENUM GameState { Playing Dead Won }

STRUCT NeighborOffset {
    dr: int
    dc: int
}

CONST width: int = 5
CONST height: int = 5
CONST mine_count: int = 3
CONST safe_count: int = 22
CONST row_indices: int[] = [0, 1, 2, 3, 4]
CONST columns: int[] = [0, 1, 2, 3, 4]
CONST neighbor_offsets: NeighborOffset[] = [
    %NeighborOffset{ dr: -1, dc: -1 },
    %NeighborOffset{ dr: -1, dc: 0 },
    %NeighborOffset{ dr: -1, dc: 1 },
    %NeighborOffset{ dr: 0, dc: -1 },
    %NeighborOffset{ dr: 0, dc: 1 },
    %NeighborOffset{ dr: 1, dc: -1 },
    %NeighborOffset{ dr: 1, dc: 0 },
    %NeighborOffset{ dr: 1, dc: 1 }
]
CONST mines: bool[] = [
    false, false, false, false, false,
    false, true, false, false, false,
    false, false, false, true, false,
    false, false, false, false, false,
    false, false, true, false, false
]
VAR revealed: bool[] = [
    false, false, false, false, false,
    false, false, false, false, false,
    false, false, false, false, false,
    false, false, false, false, false,
    false, false, false, false, false
]
VAR flagged: bool[] = [
    false, false, false, false, false,
    false, false, false, false, false,
    false, false, false, false, false,
    false, false, false, false, false,
    false, false, false, false, false
]
VAR mode: ActionMode = ActionMode.Reveal
VAR state: GameState = GameState.Playing
VAR moves: int = 0
VAR last_message: string = "Choose a row, then a column."

== main ==
Text minesweeper 5x5.
-> board_prompt

== board_prompt ==
Mode: {mode_label(mode)}
Mines: {mine_count}
    A B C D E
{ for row in row_indices:
{row_label(row)} {row_cells(row)}
}
Moves: {moves}
{last_message}
{ if state != GameState.Playing:
    { if state == GameState.Won:
        Result: cleared.
    - else:
        Result: mine exploded.
    }
    * 重新开始
        -> restart
- else:
    * 选择行 1
        -> choose_column(0)
    * 选择行 2
        -> choose_column(1)
    * 选择行 3
        -> choose_column(2)
    * 选择行 4
        -> choose_column(3)
    * 选择行 5
        -> choose_column(4)
    * 切换标记模式
        -> toggle_mode
    * 重新开始
        -> restart
}

== choose_column(row: int) ==
选择第 {row_label(row)} 行.
* A{row_label(row)}
    -> act(row, 0)
* B{row_label(row)}
    -> act(row, 1)
* C{row_label(row)}
    -> act(row, 2)
* D{row_label(row)}
    -> act(row, 3)
* E{row_label(row)}
    -> act(row, 4)
* 返回
    -> board_prompt

== toggle_mode ==
{ if mode == ActionMode.Reveal:
    ~ mode = ActionMode.Flag
    ~ last_message = "Mode changed to flag."
- else:
    ~ mode = ActionMode.Reveal
    ~ last_message = "Mode changed to reveal."
}
-> board_prompt

== restart ==
~ revealed = [false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false]
~ flagged = [false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false]
~ mode = ActionMode.Reveal
~ state = GameState.Playing
~ moves = 0
~ last_message = "Restarted."
-> board_prompt

== act(row: int, col: int) ==
~ moves = moves + 1
~ temp index: int = cell_index(row, col)
{ switch mode:
- ActionMode.Flag:
    ~ toggle_flag(row, col, index)
- else:
    ~ reveal_at(row, col, index)
}
~ check_win()
-> board_prompt

== function toggle_flag(row: int, col: int, index: int) => void ==
{ if revealed[index]:
    ~ last_message = cell_name(row, col) + " is already revealed."
- else:
    { if flagged[index]:
        ~ flagged[index] = false
        ~ last_message = "Unflagged " + cell_name(row, col) + "."
    - else:
        ~ flagged[index] = true
        ~ last_message = "Flagged " + cell_name(row, col) + "."
    }
}

== function reveal_at(row: int, col: int, index: int) => void ==
{ if flagged[index]:
    ~ last_message = cell_name(row, col) + " is flagged."
- else:
    { if revealed[index]:
        ~ last_message = cell_name(row, col) + " is already revealed."
    - else:
        { if mines[index]:
            ~ revealed[index] = true
            ~ state = GameState.Dead
            ~ last_message = "Mine hit at " + cell_name(row, col) + "."
        - else:
            ~ reveal_safe(row, col)
            ~ last_message = "Revealed " + cell_name(row, col) + "."
        }
    }
}

== function reveal_safe(row: int, col: int) => void ==
{ if row < 0 || row >= height || col < 0 || col >= width:
    ~ return
- else:
    ~ temp index: int = cell_index(row, col)
    { if revealed[index] || flagged[index] || mines[index]:
        ~ return
    - else:
        ~ revealed[index] = true
        { if adjacent_mines(row, col) == 0:
            ~ reveal_safe(row - 1, col - 1)
            ~ reveal_safe(row - 1, col)
            ~ reveal_safe(row - 1, col + 1)
            ~ reveal_safe(row, col - 1)
            ~ reveal_safe(row, col + 1)
            ~ reveal_safe(row + 1, col - 1)
            ~ reveal_safe(row + 1, col)
            ~ reveal_safe(row + 1, col + 1)
        }
    }
}

== function check_win() => void ==
{ if state == GameState.Playing:
    { if revealed_safe_count() >= safe_count:
        ~ state = GameState.Won
        ~ last_message = last_message + " All safe cells are clear."
    }
}

== function revealed_safe_count() => int ==
~ temp count: int = 0
{ for index, is_revealed in revealed:
    { if is_revealed && !mines[index]:
        ~ count += 1
    }
}
~ return count

== function row_cells(row: int) => string ==
~ temp text: string = ""
{ for col in columns:
    { if col == 0:
        ~ text = cell_char(row, col)
    - else:
        ~ text = text + " " + cell_char(row, col)
    }
}
~ return text

== function cell_char(row: int, col: int) => string ==
~ temp index: int = cell_index(row, col)
{ if state == GameState.Dead && mines[index]:
    ~ return "*"
- else:
    { if flagged[index]:
        ~ return "F"
    - else:
        { if !revealed[index]:
            ~ return "#"
        - else:
            ~ return adjacent_label(adjacent_mines(row, col))
        }
    }
}

== function adjacent_label(count: int) => string ==
{ if count == 0:
    ~ return "."
- else:
    ~ return to_str(count)
}

== function adjacent_mines(row: int, col: int) => int ==
~ temp count: int = 0
{ for offset in neighbor_offsets:
    ~ count += mine_at(row + offset.dr, col + offset.dc)
}
~ return count

== function mine_at(row: int, col: int) => int ==
{ if row < 0 || row >= height || col < 0 || col >= width:
    ~ return 0
- else:
    { if mines[cell_index(row, col)]:
        ~ return 1
    - else:
        ~ return 0
    }
}

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

== function mode_label(value: ActionMode) => string ==
{ switch value:
- ActionMode.Flag:
    ~ return "Flag"
- else:
    ~ return "Reveal"
}
