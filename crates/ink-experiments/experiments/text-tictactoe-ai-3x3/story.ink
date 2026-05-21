=== module game ===

ENUM Mark { Empty X O }
ENUM GameState { Playing XWon OWon Draw }

STRUCT Line {
    a: int
    b: int
    c: int
}

CONST rows: int[] = [0, 1, 2]
CONST cols: int[] = [0, 1, 2]
CONST lines: Line[] = [
    %Line{ a: 0, b: 1, c: 2 },
    %Line{ a: 3, b: 4, c: 5 },
    %Line{ a: 6, b: 7, c: 8 },
    %Line{ a: 0, b: 3, c: 6 },
    %Line{ a: 1, b: 4, c: 7 },
    %Line{ a: 2, b: 5, c: 8 },
    %Line{ a: 0, b: 4, c: 8 },
    %Line{ a: 2, b: 4, c: 6 }
]
CONST initial_board: Mark[] = [
    Mark.Empty, Mark.Empty, Mark.Empty,
    Mark.Empty, Mark.Empty, Mark.Empty,
    Mark.Empty, Mark.Empty, Mark.Empty
]

VAR board: Mark[] = initial_board
VAR turn: int = 0
VAR state: GameState = GameState.Playing
VAR last_message: string = "X moves first; O uses a simple blocking AI."

== main ==
Text tic-tac-toe AI 3x3.
-> board_prompt

== board_prompt ==
Turn {turn}
{ for row in rows:
{row_cells(row)}
}
{last_message}
{ if state != GameState.Playing:
    { if state == GameState.XWon:
        Result: X wins.
    - else:
        { if state == GameState.OWon:
            Result: O wins.
        - else:
            Result: draw.
        }
    }
    * Restart
        -> restart
- else:
    * Cell 1
        -> player_move(0)
    * Cell 2
        -> player_move(1)
    * Cell 3
        -> player_move(2)
    * Cell 4
        -> player_move(3)
    * Cell 5
        -> player_move(4)
    * Cell 6
        -> player_move(5)
    * Cell 7
        -> player_move(6)
    * Cell 8
        -> player_move(7)
    * Cell 9
        -> player_move(8)
    * Restart
        -> restart
}

== restart ==
~ board = initial_board
~ turn = 0
~ state = GameState.Playing
~ last_message = "Restarted."
-> board_prompt

== player_move(index: int) ==
{ if board[index] != Mark.Empty:
    ~ last_message = "Cell " + cell_label(index) + " is occupied."
- else:
    ~ board[index] = Mark.X
    ~ turn += 1
    ~ update_state()
    { if state == GameState.Playing:
        ~ ai_move()
    - else:
        ~ last_message = "X takes cell " + cell_label(index) + "."
    }
}
-> board_prompt

== function ai_move() => void ==
~ temp move: int = winning_move(Mark.O)
{ if move < 0:
    ~ move = winning_move(Mark.X)
}
{ if move < 0 && board[4] == Mark.Empty:
    ~ move = 4
}
{ if move < 0:
    ~ move = first_empty()
}
{ if move >= 0:
    ~ board[move] = Mark.O
    ~ last_message = "X moves; O takes cell " + cell_label(move) + "."
}
~ update_state()

== function update_state() => void ==
{ if has_winner(Mark.X):
    ~ state = GameState.XWon
- else:
    { if has_winner(Mark.O):
        ~ state = GameState.OWon
    - else:
        { if empty_count() == 0:
            ~ state = GameState.Draw
        }
    }
}

== function winning_move(mark: Mark) => int ==
~ temp found: int = -1
{ for line in lines:
    ~ temp a: Mark = board[line.a]
    ~ temp b: Mark = board[line.b]
    ~ temp c: Mark = board[line.c]
    { if a == Mark.Empty && b == mark && c == mark:
        ~ found = line.a
    }
    { if b == Mark.Empty && a == mark && c == mark:
        ~ found = line.b
    }
    { if c == Mark.Empty && a == mark && b == mark:
        ~ found = line.c
    }
}
~ return found

== function has_winner(mark: Mark) => bool ==
~ temp found: bool = false
{ for line in lines:
    { if board[line.a] == mark && board[line.b] == mark && board[line.c] == mark:
        ~ found = true
    }
}
~ return found

== function first_empty() => int ==
~ temp found: int = -1
{ for index, mark in board:
    { if found < 0 && mark == Mark.Empty:
        ~ found = index
    }
}
~ return found

== function empty_count() => int ==
~ temp count: int = 0
{ for mark in board:
    { if mark == Mark.Empty:
        ~ count += 1
    }
}
~ return count

== function row_cells(row: int) => string ==
~ temp text: string = ""
{ for col in cols:
    ~ temp index: int = row * LEN(cols) + col
    { if col == 0:
        ~ text = mark_label(board[index], index)
    - else:
        ~ text = text + "|" + mark_label(board[index], index)
    }
}
~ return text

== function mark_label(mark: Mark, index: int) => string ==
{ switch mark:
- Mark.X:
    ~ return "X"
- Mark.O:
    ~ return "O"
- else:
    ~ return cell_label(index)
}

== function cell_label(index: int) => string ==
{ switch index:
- 0:
    ~ return "1"
- 1:
    ~ return "2"
- 2:
    ~ return "3"
- 3:
    ~ return "4"
- 4:
    ~ return "5"
- 5:
    ~ return "6"
- 6:
    ~ return "7"
- 7:
    ~ return "8"
- else:
    ~ return "9"
}
