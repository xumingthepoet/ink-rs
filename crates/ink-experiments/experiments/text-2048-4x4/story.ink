=== module game ===

ENUM Direction { Up Down Left Right }
ENUM GameState { Playing Failed Won }

STRUCT Line {
    i0: int
    i1: int
    i2: int
    i3: int
}

STRUCT Pair {
    left: int
    right: int
}

CONST initial_tiles: int[] = [
    2, 4, 8, 16,
    32, 64, 128, 256,
    512, 1024, 2, 4,
    8, 16, 32, 0
]
CONST row_starts: int[] = [0, 4, 8, 12]
CONST left_lines: Line[] = [
    %Line{ i0: 0, i1: 1, i2: 2, i3: 3 },
    %Line{ i0: 4, i1: 5, i2: 6, i3: 7 },
    %Line{ i0: 8, i1: 9, i2: 10, i3: 11 },
    %Line{ i0: 12, i1: 13, i2: 14, i3: 15 }
]
CONST right_lines: Line[] = [
    %Line{ i0: 3, i1: 2, i2: 1, i3: 0 },
    %Line{ i0: 7, i1: 6, i2: 5, i3: 4 },
    %Line{ i0: 11, i1: 10, i2: 9, i3: 8 },
    %Line{ i0: 15, i1: 14, i2: 13, i3: 12 }
]
CONST up_lines: Line[] = [
    %Line{ i0: 0, i1: 4, i2: 8, i3: 12 },
    %Line{ i0: 1, i1: 5, i2: 9, i3: 13 },
    %Line{ i0: 2, i1: 6, i2: 10, i3: 14 },
    %Line{ i0: 3, i1: 7, i2: 11, i3: 15 }
]
CONST down_lines: Line[] = [
    %Line{ i0: 12, i1: 8, i2: 4, i3: 0 },
    %Line{ i0: 13, i1: 9, i2: 5, i3: 1 },
    %Line{ i0: 14, i1: 10, i2: 6, i3: 2 },
    %Line{ i0: 15, i1: 11, i2: 7, i3: 3 }
]
CONST merge_pairs: Pair[] = [
    %Pair{ left: 0, right: 1 },
    %Pair{ left: 1, right: 2 },
    %Pair{ left: 2, right: 3 },
    %Pair{ left: 4, right: 5 },
    %Pair{ left: 5, right: 6 },
    %Pair{ left: 6, right: 7 },
    %Pair{ left: 8, right: 9 },
    %Pair{ left: 9, right: 10 },
    %Pair{ left: 10, right: 11 },
    %Pair{ left: 12, right: 13 },
    %Pair{ left: 13, right: 14 },
    %Pair{ left: 14, right: 15 },
    %Pair{ left: 0, right: 4 },
    %Pair{ left: 4, right: 8 },
    %Pair{ left: 8, right: 12 },
    %Pair{ left: 1, right: 5 },
    %Pair{ left: 5, right: 9 },
    %Pair{ left: 9, right: 13 },
    %Pair{ left: 2, right: 6 },
    %Pair{ left: 6, right: 10 },
    %Pair{ left: 10, right: 14 },
    %Pair{ left: 3, right: 7 },
    %Pair{ left: 7, right: 11 },
    %Pair{ left: 11, right: 15 }
]
VAR tiles: int[] = initial_tiles
VAR scratch: int[] = []
VAR merged: int[] = []
CONST spawn_values: int[] = [2, 4, 2, 2]
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
{ for row_start in row_starts:
|{tile_label(tiles[row_start])}|{tile_label(tiles[row_start + 1])}|{tile_label(tiles[row_start + 2])}|{tile_label(tiles[row_start + 3])}|
|----|----|----|----|
}
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
~ tiles = initial_tiles
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
    ~ changed = apply_lines(left_lines)
- Direction.Right:
    ~ changed = apply_lines(right_lines)
- Direction.Up:
    ~ changed = apply_lines(up_lines)
- else:
    ~ changed = apply_lines(down_lines)
}
{ if changed:
    ~ spawn_first_empty(0)
    ~ last_message = "Move " + label + " changed the board; spawned a tile."
- else:
    ~ last_message = "Move " + label + " has no effect."
}
~ update_game_state()
-> board_prompt

== function apply_lines(lines: Line[]) => bool ==
~ temp changed: bool = false
{ for line in lines:
    { if apply_line(line.i0, line.i1, line.i2, line.i3):
        ~ changed = true
    }
}
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
{ if has_tile_at_least(2048):
    ~ state = GameState.Won
- else:
    { if !can_move():
        ~ state = GameState.Failed
        ~ last_message = last_message + " No moves left."
    }
}

== function can_move() => bool ==
~ return has_empty() || has_any_merge()

== function has_empty() => bool ==
~ temp found: bool = false
{ for tile in tiles:
    { if tile == 0:
        ~ found = true
    }
}
~ return found

== function has_any_merge() => bool ==
~ temp found: bool = false
{ for pair in merge_pairs:
    { if same_nonzero(pair.left, pair.right):
        ~ found = true
    }
}
~ return found

== function same_nonzero(left: int, right: int) => bool ==
~ return tiles[left] != 0 && tiles[left] == tiles[right]

== function has_tile_at_least(target: int) => bool ==
~ temp found: bool = false
{ for tile in tiles:
    { if tile >= target:
        ~ found = true
    }
}
~ return found

== function tile_label(value: int) => string ==
{ if value == 0:
    ~ return "."
- else:
    ~ return to_str(value)
}
