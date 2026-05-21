=== module game ===

ENUM Direction { Up Down Left Right }
ENUM GameState { Playing Dead Won }

VAR width: int = 10
VAR height: int = 10
VAR snake_x: int[] = [4, 3, 2]
VAR snake_y: int[] = [4, 4, 4]
VAR current_direction: Direction = Direction.Right
VAR apple_xs: int[] = [6, 6, 8]
VAR apple_ys: int[] = [4, 5, 5]
VAR apple_index: int = 0
VAR move_count: int = 0
VAR state: GameState = GameState.Playing
VAR last_message: string = "Use choices to steer the snake."

== main ==
Text snake 10x10.
-> board_prompt

== board_prompt ==
Move {move_count}
{border_row(0, "")}
{board_line(0)}
{board_line(1)}
{board_line(2)}
{board_line(3)}
{board_line(4)}
{board_line(5)}
{board_line(6)}
{board_line(7)}
{board_line(8)}
{board_line(9)}
{border_row(0, "")}
Length: {LEN(snake_x)}
{ if apple_index >= LEN(apple_xs):
    Apple: none
- else:
    Apple: ({apple_xs[apple_index]},{apple_ys[apple_index]})
}
{last_message}
{ if state != GameState.Playing:
    { if state == GameState.Won:
        Result: all apples collected.
    - else:
        Result: dead.
    }
    * 重新开始
        -> restart
- else:
    * 上
        -> move_snake(Direction.Up, "up")
    * 下
        -> move_snake(Direction.Down, "down")
    * 左
        -> move_snake(Direction.Left, "left")
    * 右
        -> move_snake(Direction.Right, "right")
    * 过
        -> move_snake(current_direction, "wait")
}

== restart ==
~ snake_x = [4, 3, 2]
~ snake_y = [4, 4, 4]
~ current_direction = Direction.Right
~ apple_index = 0
~ move_count = 0
~ state = GameState.Playing
~ last_message = "Restarted."
-> board_prompt

== move_snake(next_direction: Direction, label: string) ==
~ move_count = move_count + 1
~ temp dx: int = direction_dx(next_direction)
~ temp dy: int = direction_dy(next_direction)
~ temp next_x: int = snake_x[0] + dx
~ temp next_y: int = snake_y[0] + dy
{ if next_x < 0 || next_x >= width || next_y < 0 || next_y >= height:
    ~ state = GameState.Dead
    ~ last_message = "Move " + label + " hits the wall."
- else:
    ~ temp grows: bool = is_food_cell(next_x, next_y)
    { if snake_hits_self(next_x, next_y, grows, 0):
        ~ state = GameState.Dead
        ~ last_message = "Move " + label + " hits the snake body."
    - else:
        ~ current_direction = next_direction
        ~ ARRAY_INSERT(snake_x, 0, next_x)
        ~ ARRAY_INSERT(snake_y, 0, next_y)
        { if grows:
            ~ apple_index = apple_index + 1
            ~ last_message = "Move " + label + " eats an apple."
            { if apple_index >= LEN(apple_xs):
                ~ state = GameState.Won
                ~ last_message = "Move " + label + " eats the final apple."
            }
        - else:
            ~ ARRAY_REMOVE(snake_x, LEN(snake_x) - 1)
            ~ ARRAY_REMOVE(snake_y, LEN(snake_y) - 1)
            ~ last_message = "Move " + label + " advances."
        }
    }
}
-> board_prompt

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

== function border_row(x: int, text: string) => string ==
{ if x >= width + 2:
    ~ return text
- else:
    ~ return border_row(x + 1, text + "#")
}

== function board_line(y: int) => string ==
~ return "#" + board_row(y, 0, "") + "#"

== function board_row(y: int, x: int, text: string) => string ==
{ if x >= width:
    ~ return text
- else:
    ~ return board_row(y, x + 1, text + cell_char(x, y))
}

== function cell_char(x: int, y: int) => string ==
~ temp index: int = snake_index_at(x, y, 0)
{ if index == 0:
    ~ return "H"
- else:
    { if index > 0:
        ~ return "o"
    - else:
        { if is_food_cell(x, y):
            ~ return "A"
        - else:
            ~ return "."
        }
    }
}

== function snake_index_at(x: int, y: int, index: int) => int ==
{ if index >= LEN(snake_x):
    ~ return -1
- else:
    { if snake_x[index] == x && snake_y[index] == y:
        ~ return index
    - else:
        ~ return snake_index_at(x, y, index + 1)
    }
}

== function is_food_cell(x: int, y: int) => bool ==
{ if apple_index >= LEN(apple_xs):
    ~ return false
- else:
    ~ return x == apple_xs[apple_index] && y == apple_ys[apple_index]
}

== function snake_hits_self(x: int, y: int, grows: bool, index: int) => bool ==
{ if index >= LEN(snake_x):
    ~ return false
- else:
    { if !grows && index == LEN(snake_x) - 1:
        ~ return false
    - else:
        { if snake_x[index] == x && snake_y[index] == y:
            ~ return true
        - else:
            ~ return snake_hits_self(x, y, grows, index + 1)
        }
    }
}
