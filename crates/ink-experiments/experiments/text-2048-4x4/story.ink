=== module game ===

ENUM Direction { Up Down Left Right }
ENUM GameState { Playing Failed Won }

VAR tiles: int[] = [
    2, 4, 8, 16,
    32, 64, 128, 256,
    512, 1024, 2, 4,
    8, 16, 32, 0
]
VAR scratch: int[] = []
VAR merged: int[] = []
VAR spawn_values: int[] = [2, 4, 2, 2]
VAR spawn_cursor: int = 0
VAR move_count: int = 0
VAR score: int = 0
VAR state: GameState = GameState.Playing
VAR last_message: string = "Use choices to slide tiles."

== main ==
Text 2048 4x4.
-> board_prompt

== board_prompt ==
Move {move_count}
Score: {score}
|----|----|----|----|
|{tile_label(tiles[0])}|{tile_label(tiles[1])}|{tile_label(tiles[2])}|{tile_label(tiles[3])}|
|----|----|----|----|
|{tile_label(tiles[4])}|{tile_label(tiles[5])}|{tile_label(tiles[6])}|{tile_label(tiles[7])}|
|----|----|----|----|
|{tile_label(tiles[8])}|{tile_label(tiles[9])}|{tile_label(tiles[10])}|{tile_label(tiles[11])}|
|----|----|----|----|
|{tile_label(tiles[12])}|{tile_label(tiles[13])}|{tile_label(tiles[14])}|{tile_label(tiles[15])}|
|----|----|----|----|
{last_message}
{ if state != GameState.Playing:
    { if state == GameState.Won:
        Result: 2048 reached.
    - else:
        Result: failed; no moves left.
    }
    * 重新开始
        -> restart
- else:
    * 上
        -> move_board(Direction.Up, "up")
    * 下
        -> move_board(Direction.Down, "down")
    * 左
        -> move_board(Direction.Left, "left")
    * 右
        -> move_board(Direction.Right, "right")
}

== restart ==
~ tiles = [2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2, 4, 8, 16, 32, 0]
~ scratch = []
~ merged = []
~ spawn_cursor = 0
~ move_count = 0
~ score = 0
~ state = GameState.Playing
~ last_message = "Restarted."
-> board_prompt

== move_board(direction: Direction, label: string) ==
~ move_count = move_count + 1
~ temp changed: bool = false
{ switch direction:
- Direction.Left:
    ~ changed = apply_lines_left()
- Direction.Right:
    ~ changed = apply_lines_right()
- Direction.Up:
    ~ changed = apply_lines_up()
- else:
    ~ changed = apply_lines_down()
}
{ if changed:
    ~ spawn_first_empty(0)
    ~ last_message = "Move " + label + " changed the board; spawned a tile."
- else:
    ~ last_message = "Move " + label + " has no effect."
}
~ update_game_state()
-> board_prompt

== function apply_lines_left() => bool ==
~ temp changed: bool = false
~ changed = apply_line(0, 1, 2, 3) || changed
~ changed = apply_line(4, 5, 6, 7) || changed
~ changed = apply_line(8, 9, 10, 11) || changed
~ changed = apply_line(12, 13, 14, 15) || changed
~ return changed

== function apply_lines_right() => bool ==
~ temp changed: bool = false
~ changed = apply_line(3, 2, 1, 0) || changed
~ changed = apply_line(7, 6, 5, 4) || changed
~ changed = apply_line(11, 10, 9, 8) || changed
~ changed = apply_line(15, 14, 13, 12) || changed
~ return changed

== function apply_lines_up() => bool ==
~ temp changed: bool = false
~ changed = apply_line(0, 4, 8, 12) || changed
~ changed = apply_line(1, 5, 9, 13) || changed
~ changed = apply_line(2, 6, 10, 14) || changed
~ changed = apply_line(3, 7, 11, 15) || changed
~ return changed

== function apply_lines_down() => bool ==
~ temp changed: bool = false
~ changed = apply_line(12, 8, 4, 0) || changed
~ changed = apply_line(13, 9, 5, 1) || changed
~ changed = apply_line(14, 10, 6, 2) || changed
~ changed = apply_line(15, 11, 7, 3) || changed
~ return changed

== function apply_line(i0: int, i1: int, i2: int, i3: int) => bool ==
~ temp old0: int = tiles[i0]
~ temp old1: int = tiles[i1]
~ temp old2: int = tiles[i2]
~ temp old3: int = tiles[i3]
~ scratch = []
~ merged = []
~ collect_nonzero(i0)
~ collect_nonzero(i1)
~ collect_nonzero(i2)
~ collect_nonzero(i3)
~ merge_scratch(0)
~ pad_merged()
~ tiles[i0] = merged[0]
~ tiles[i1] = merged[1]
~ tiles[i2] = merged[2]
~ tiles[i3] = merged[3]
~ return old0 != tiles[i0] || old1 != tiles[i1] || old2 != tiles[i2] || old3 != tiles[i3]

== function collect_nonzero(tile_index: int) => void ==
{ if tiles[tile_index] != 0:
    ~ ARRAY_PUSH(scratch, tiles[tile_index])
}

== function merge_scratch(index: int) => void ==
{ if index >= LEN(scratch):
    ~ return
- else:
    ~ temp value: int = scratch[index]
    { if index + 1 < LEN(scratch) && scratch[index + 1] == value:
        ~ temp doubled: int = value * 2
        ~ ARRAY_PUSH(merged, doubled)
        ~ score = score + doubled
        ~ merge_scratch(index + 2)
    - else:
        ~ ARRAY_PUSH(merged, value)
        ~ merge_scratch(index + 1)
    }
}

== function pad_merged() => void ==
{ if LEN(merged) < 4:
    ~ ARRAY_PUSH(merged, 0)
    ~ pad_merged()
}

== function spawn_first_empty(index: int) => void ==
{ if index >= LEN(tiles):
    ~ return
- else:
    { if tiles[index] == 0:
        ~ tiles[index] = next_spawn_value()
        ~ spawn_cursor = spawn_cursor + 1
    - else:
        ~ spawn_first_empty(index + 1)
    }
}

== function next_spawn_value() => int ==
{ if spawn_cursor >= LEN(spawn_values):
    ~ spawn_cursor = 0
}
~ return spawn_values[spawn_cursor]

== function update_game_state() => void ==
{ if has_tile_at_least(2048, 0):
    ~ state = GameState.Won
- else:
    { if !can_move():
        ~ state = GameState.Failed
        ~ last_message = last_message + " No moves left."
    }
}

== function can_move() => bool ==
~ return has_empty(0) || has_any_horizontal_merge() || has_any_vertical_merge()

== function has_empty(index: int) => bool ==
{ if index >= LEN(tiles):
    ~ return false
- else:
    { if tiles[index] == 0:
        ~ return true
    - else:
        ~ return has_empty(index + 1)
    }
}

== function has_any_horizontal_merge() => bool ==
~ return same_nonzero(0, 1) || same_nonzero(1, 2) || same_nonzero(2, 3) || same_nonzero(4, 5) || same_nonzero(5, 6) || same_nonzero(6, 7) || same_nonzero(8, 9) || same_nonzero(9, 10) || same_nonzero(10, 11) || same_nonzero(12, 13) || same_nonzero(13, 14) || same_nonzero(14, 15)

== function has_any_vertical_merge() => bool ==
~ return same_nonzero(0, 4) || same_nonzero(4, 8) || same_nonzero(8, 12) || same_nonzero(1, 5) || same_nonzero(5, 9) || same_nonzero(9, 13) || same_nonzero(2, 6) || same_nonzero(6, 10) || same_nonzero(10, 14) || same_nonzero(3, 7) || same_nonzero(7, 11) || same_nonzero(11, 15)

== function same_nonzero(left: int, right: int) => bool ==
~ return tiles[left] != 0 && tiles[left] == tiles[right]

== function has_tile_at_least(target: int, index: int) => bool ==
{ if index >= LEN(tiles):
    ~ return false
- else:
    { if tiles[index] >= target:
        ~ return true
    - else:
        ~ return has_tile_at_least(target, index + 1)
    }
}

== function tile_label(value: int) => string ==
{ switch value:
- 0:
    ~ return "."
- 2:
    ~ return "2"
- 4:
    ~ return "4"
- 8:
    ~ return "8"
- 16:
    ~ return "16"
- 32:
    ~ return "32"
- 64:
    ~ return "64"
- 128:
    ~ return "128"
- 256:
    ~ return "256"
- 512:
    ~ return "512"
- 1024:
    ~ return "1024"
- else:
    ~ return "2048"
}
