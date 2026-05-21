=== module game ===

ENUM Direction { Up Down Left Right }
ENUM GameState { Playing Won }

STRUCT Pos {
    x: int
    y: int
}

CONST width: int = 8
CONST height: int = 8
CONST rows: int[] = [0, 1, 2, 3, 4, 5, 6, 7]
CONST cols: int[] = [0, 1, 2, 3, 4, 5, 6, 7]
CONST initial_player: Pos = %Pos{ x: 1, y: 1 }
CONST initial_boxes: Pos[] = [
    %Pos{ x: 2, y: 2 },
    %Pos{ x: 3, y: 4 }
]
CONST goals: Pos[] = [
    %Pos{ x: 5, y: 2 },
    %Pos{ x: 5, y: 4 }
]
CONST walls: Pos[] = [
    %Pos{ x: 0, y: 0 }, %Pos{ x: 1, y: 0 }, %Pos{ x: 2, y: 0 }, %Pos{ x: 3, y: 0 }, %Pos{ x: 4, y: 0 }, %Pos{ x: 5, y: 0 }, %Pos{ x: 6, y: 0 }, %Pos{ x: 7, y: 0 },
    %Pos{ x: 0, y: 1 }, %Pos{ x: 7, y: 1 },
    %Pos{ x: 0, y: 2 }, %Pos{ x: 7, y: 2 },
    %Pos{ x: 0, y: 3 }, %Pos{ x: 2, y: 3 }, %Pos{ x: 7, y: 3 },
    %Pos{ x: 0, y: 4 }, %Pos{ x: 7, y: 4 },
    %Pos{ x: 0, y: 5 }, %Pos{ x: 4, y: 5 }, %Pos{ x: 7, y: 5 },
    %Pos{ x: 0, y: 6 }, %Pos{ x: 7, y: 6 },
    %Pos{ x: 0, y: 7 }, %Pos{ x: 1, y: 7 }, %Pos{ x: 2, y: 7 }, %Pos{ x: 3, y: 7 }, %Pos{ x: 4, y: 7 }, %Pos{ x: 5, y: 7 }, %Pos{ x: 6, y: 7 }, %Pos{ x: 7, y: 7 }
]

VAR player: Pos = initial_player
VAR boxes: Pos[] = initial_boxes
VAR moves: int = 0
VAR state: GameState = GameState.Playing
VAR last_message: string = "Push both boxes onto goals."

== main ==
Text sokoban 8x8.
-> board_prompt

== board_prompt ==
Move {moves}
{ for row in rows:
{board_row(row)}
}
Boxes on goals: {boxes_on_goals()}/{LEN(goals)}
{last_message}
{ if state == GameState.Won:
    Result: warehouse cleared.
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
~ player = initial_player
~ boxes = initial_boxes
~ moves = 0
~ state = GameState.Playing
~ last_message = "Restarted."
-> board_prompt

== move_player(direction: Direction, label: string) ==
~ moves += 1
~ temp dx: int = direction_dx(direction)
~ temp dy: int = direction_dy(direction)
~ temp next_x: int = player.x + dx
~ temp next_y: int = player.y + dy
~ temp box_index: int = box_index_at(next_x, next_y)
{ if is_wall(next_x, next_y):
    ~ last_message = "Move " + label + " hits a wall."
- else:
    { if box_index >= 0:
        ~ temp push_x: int = next_x + dx
        ~ temp push_y: int = next_y + dy
        { if is_wall(push_x, push_y) || box_index_at(push_x, push_y) >= 0:
            ~ last_message = "Move " + label + " cannot push the box."
        - else:
            ~ boxes[box_index].x = push_x
            ~ boxes[box_index].y = push_y
            ~ player.x = next_x
            ~ player.y = next_y
            ~ last_message = "Move " + label + " pushes a box."
        }
    - else:
        ~ player.x = next_x
        ~ player.y = next_y
        ~ last_message = "Move " + label + " walks."
    }
}
~ check_win()
-> board_prompt

== function check_win() => void ==
{ if state == GameState.Playing && boxes_on_goals() == LEN(goals):
    ~ state = GameState.Won
    ~ last_message = last_message + " All goals are covered."
}

== function board_row(row: int) => string ==
~ temp text: string = ""
{ for col in cols:
    ~ text = text + cell_char(col, row)
}
~ return text

== function cell_char(x: int, y: int) => string ==
~ temp box_index: int = box_index_at(x, y)
{ if is_wall(x, y):
    ~ return "#"
- else:
    { if player.x == x && player.y == y:
        ~ return "@"
    - else:
        { if box_index >= 0:
            { if is_goal(x, y):
                ~ return "*"
            - else:
                ~ return "$"
            }
        - else:
            { if is_goal(x, y):
                ~ return "G"
            - else:
                ~ return "."
            }
        }
    }
}

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

== function box_index_at(x: int, y: int) => int ==
~ temp found: int = -1
{ for index, box in boxes:
    { if box.x == x && box.y == y:
        ~ found = index
    }
}
~ return found

== function is_wall(x: int, y: int) => bool ==
~ temp found: bool = false
{ for wall in walls:
    { if wall.x == x && wall.y == y:
        ~ found = true
    }
}
~ return found

== function is_goal(x: int, y: int) => bool ==
~ temp found: bool = false
{ for goal in goals:
    { if goal.x == x && goal.y == y:
        ~ found = true
    }
}
~ return found

== function boxes_on_goals() => int ==
~ temp count: int = 0
{ for box in boxes:
    { if is_goal(box.x, box.y):
        ~ count += 1
    }
}
~ return count
